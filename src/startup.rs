use crate::configurations::Configuration;
use crate::models::AppState;
use crate::routes::{health_check, internal};
use crate::utils::cleanup_idle_tenant_pools;
use actix_cors::Cors;
use actix_session::config::PersistentSession;
use actix_session::storage::CookieSessionStore;
use actix_session::SessionMiddleware;
use actix_web::cookie::time::Duration;
use actix_web::cookie::Key;
use actix_web::dev::Server;
use actix_web::web::Data;
use actix_web::{http, web, HttpServer};
use secrecy::ExposeSecret;
use sqlx::PgPool;
use std::collections::HashMap;
use std::net::TcpListener;
use std::sync::Arc;
use tokio::sync::Mutex;
use tokio::time;
use tracing::event;
use tracing_actix_web::TracingLogger;

pub struct Application {
    port: u16,
    server: Server,
}

impl Application {
    pub async fn build(
        configuration: Configuration,
        pool: Option<PgPool>,
    ) -> Result<Self, std::io::Error> {
        let connection_pool = if let Some(pool) = pool {
            pool
        } else {
            let database_connect_options = configuration.database.with_db();

            match sqlx::postgres::PgPoolOptions::new()
                .max_connections(configuration.database.max_connections)
                .connect_with(database_connect_options)
                .await
            {
                Ok(pool) => pool,
                Err(e) => {
                    event!(target: "sqlx",tracing::Level::ERROR, "Couldn't establish DB connection!: {:#?}", e);
                    panic!("Couldn't establish DB connection!")
                }
            }
        };

        sqlx::migrate!()
            .run(&connection_pool)
            .await
            .expect("Failed to migrate the database.");

        let address = format!(
            "{}:{}",
            configuration.application.host, configuration.application.port
        );

        let listener = TcpListener::bind(&address)?;
        let port = listener.local_addr()?.port();
        let server = run(listener, connection_pool, configuration).await?;

        Ok(Self { port, server })
    }

    pub fn port(&self) -> u16 {
        self.port
    }

    pub async fn run_until_stopped(self) -> Result<(), std::io::Error> {
        self.server.await
    }
}

async fn run(
    listener: TcpListener,
    db_pool: PgPool,
    settings: Configuration,
) -> Result<Server, std::io::Error> {
    let settings_data = Data::new(settings.clone());
    let app_state_data = Data::new(AppState {
        pools: Arc::new(Mutex::new(HashMap::new())),
    });
    // Database connection pool application state
    let database_pool_data = Data::new(db_pool);

    let server = HttpServer::new(move || {
        let hmac_secret_key = Key::from(settings.secrets.hmac.expose_secret().as_bytes());

        let application_host_origin = settings.application.host.clone();
        let application_cookie = settings.application.cookie.clone();
        let actix_compress_middleware = actix_web::middleware::Compress::default();
        let actix_cors_middleware = Cors::default()
            .allowed_origin(&application_host_origin)
            .allowed_origin_fn(move |origin, _req_head| {
                origin
                    .as_bytes()
                    .ends_with(application_host_origin.as_bytes())
            })
            .allowed_methods(vec!["GET", "POST", "PUT"])
            .allowed_headers(vec![http::header::AUTHORIZATION, http::header::ACCEPT])
            .allowed_header(http::header::CONTENT_TYPE)
            .max_age(3600);

        let actix_cookie_session_middleware =
            SessionMiddleware::builder(CookieSessionStore::default(), hmac_secret_key)
                .cookie_http_only(application_cookie.http_only)
                .cookie_same_site(application_cookie.same_site.try_into().unwrap())
                .cookie_secure(application_cookie.secure)
                .cookie_name(application_cookie.name)
                .cookie_content_security(application_cookie.content_security.try_into().unwrap())
                .cookie_domain(application_cookie.domain)
                .cookie_path(application_cookie.path)
                .session_lifecycle(
                    PersistentSession::default()
                        .session_ttl(Duration::hours(application_cookie.session_ttl)),
                )
                .build();

        // Spawn a background task to clean up idle pools
        let state_clone = app_state_data.clone();

        tokio::spawn(async move {
            let cleanup_interval = tokio::time::Duration::from_secs(5 * 60); // 5 minutes
            let idle_duration_in_seconds = 10 * 60; // 10 minutes

            loop {
                time::sleep(cleanup_interval).await;
                cleanup_idle_tenant_pools(&state_clone, idle_duration_in_seconds).await;
            }
        });

        actix_web::App::new()
            .app_data(app_state_data.clone())
            .app_data(database_pool_data.clone())
            .app_data(settings_data.clone())
            .wrap(actix_cookie_session_middleware)
            .wrap(actix_compress_middleware)
            .wrap(actix_cors_middleware)
            .route("/health_check", web::get().to(health_check))
            .service(
                web::scope("/v1").service(web::scope("/internal").configure(internal::configure)),
            )
            .wrap(TracingLogger::default())
    })
    .listen(listener)?
    .run();

    Ok(server)
}
