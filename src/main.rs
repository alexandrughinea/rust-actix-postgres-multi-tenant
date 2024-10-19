mod utils;

use crate::utils::{cleanup_idle_tenant_pools, create_user, get_users};
use actix_web::{web, App, HttpServer};
use chrono::{DateTime, Utc};
use dotenv::dotenv;
use serde::{Deserialize, Serialize};
use sqlx::postgres::PgPoolOptions;
use sqlx::{FromRow, PgPool};
use std::collections::HashMap;
use std::env;
use std::sync::{Arc, Mutex};
use tokio::time::{self, Duration};
use uuid::Uuid;

#[derive(Serialize, Deserialize, FromRow)]
struct Tenant {
    pub id: Option<Uuid>,
    pub name: String,
    pub db_user: String,
    #[serde(skip)]
    pub db_password_encrypted: Option<String>,
    pub created_at: Option<DateTime<Utc>>,
    pub updated_at: Option<DateTime<Utc>>,
}

#[derive(Serialize, Deserialize, FromRow)]
struct TenantCredentials {
    pub db_user: String,
    #[serde(skip)]
    pub db_password: String,
}

#[derive(Serialize, Deserialize, FromRow)]
struct User {
    pub id: Option<Uuid>,
    #[serde(skip)]
    pub tenant_id: Option<Uuid>,
    pub name: String,
    pub created_at: Option<DateTime<Utc>>,
    pub updated_at: Option<DateTime<Utc>>,
}

struct TenantPool {
    pool: Arc<PgPool>,
    last_accessed: DateTime<Utc>,
}

struct AppState {
    pools: Arc<Mutex<HashMap<Uuid, Arc<Mutex<TenantPool>>>>>,
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    dotenv().ok();

    let database_url = match env::var("DATABASE_URL") {
        Ok(url) => url,
        Err(_) => {
            eprintln!("Error: DATABASE_URL environment variable not set.");
            return Err(std::io::Error::new(
                std::io::ErrorKind::Other,
                "DATABASE_URL not set",
            ));
        }
    };

    let global_pool = web::Data::new(
        PgPoolOptions::new()
            .max_connections(5)
            .connect(&database_url)
            .await
            .expect("Failed to create global pool."),
    );

    let state = web::Data::new(AppState {
        pools: Arc::new(Mutex::new(HashMap::new())),
    });

    // Spawn a background task to clean up idle pools
    let state_clone = state.clone();

    tokio::spawn(async move {
        let cleanup_interval = Duration::from_secs(5 * 60); // 5 minutes
        let idle_duration_in_seconds = 10 * 60; // 10 minutes

        loop {
            time::sleep(cleanup_interval).await;
            cleanup_idle_tenant_pools(&state_clone, idle_duration_in_seconds).await;
        }
    });

    HttpServer::new(move || {
        App::new()
            .app_data(state.clone())
            .app_data(global_pool.clone())
            .route("/users", web::get().to(get_users))
            .route("/users", web::post().to(create_user))
    })
    .bind("127.0.0.1:8080")?
    .run()
    .await
}
