mod utils;

use crate::utils::{cleanup_idle_tenant_pools, get_pool_for_tenant, get_tenant_id_from_request};
use actix_web::{web, App, HttpRequest, HttpResponse, HttpServer, Responder};
use chrono::{DateTime, Utc};
use dotenv::dotenv;
use serde::{Deserialize, Serialize};
use serde_json::json;
use sqlx::postgres::PgPoolOptions;
use sqlx::{FromRow, PgPool};
use std::collections::HashMap;
use std::env;
use std::str::FromStr;
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
    pool: PgPool,
    last_accessed: DateTime<Utc>,
}

struct AppState {
    pools: Arc<Mutex<HashMap<Uuid, TenantPool>>>,
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
        let cleanup_interval = Duration::from_secs(60 * 5); // 5 minutes

        loop {
            time::sleep(cleanup_interval).await;
            cleanup_idle_tenant_pools(&state_clone).await;
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

async fn get_users(
    req: HttpRequest,
    state: web::Data<AppState>,
    pool: web::Data<PgPool>,
) -> impl Responder {
    let tenant_id = match get_tenant_id_from_request(&req) {
        Ok(id) => id,
        Err(e) => return e,
    };
    let pool = match get_pool_for_tenant(&tenant_id, &state, &pool).await {
        Ok(pool) => pool,
        Err(e) => return e,
    };

    let users = match sqlx::query_as!(User, "SELECT * FROM users")
        .fetch_all(&pool)
        .await
    {
        Ok(users) => users,
        Err(_) => return HttpResponse::InternalServerError().body("Error fetching users"),
    };

    HttpResponse::Ok().json(json!(users))
}

async fn create_user(
    req: HttpRequest,
    user: web::Json<User>,
    state: web::Data<AppState>,
    pool: web::Data<PgPool>,
) -> impl Responder {
    let tenant_id = match get_tenant_id_from_request(&req) {
        Ok(id) => id,
        Err(e) => return e,
    };
    let pool = match get_pool_for_tenant(&tenant_id, &state, &pool).await {
        Ok(pool) => pool,
        Err(e) => return e,
    };

    let user = sqlx::query_as::<_, User>(
        r#"
        INSERT INTO users (tenant_id, name)
        VALUES ($1, $2)
        RETURNING *
        "#,
    )
    .bind(tenant_id)
    .bind(&user.name)
    .fetch_one(&pool)
    .await
    .unwrap();

    HttpResponse::Ok().json(json!(user))
}
