use crate::utils::decrypt_aes_gcm;
use crate::{AppState, Tenant, TenantCredentials, TenantPool};
use actix_web::{web, HttpRequest, HttpResponse};
use aes_gcm::aead::KeyInit;
use chrono::Utc;
use sqlx::postgres::{PgPool, PgPoolOptions};
use std::env;
use std::time::Duration;
use uuid::Uuid;

pub async fn fetch_tenant_db_credentials(
    pool: &PgPool,
    tenant_id: &Uuid,
) -> Result<TenantCredentials, Box<dyn std::error::Error>> {
    let tenant = sqlx::query_as!(Tenant, "SELECT * FROM tenants WHERE id = $1", tenant_id)
        .fetch_one(&*pool)
        .await?;

    let decryption_key = env::var("DATABASE_AES_KEY").expect("DATABASE_AES_KEY must be set");
    let db_password_encrypted = tenant.db_password_encrypted.unwrap();
    let db_password_plaintext = decrypt_aes_gcm(&decryption_key, db_password_encrypted.as_str())?;

    Ok(TenantCredentials {
        db_user: tenant.db_user,
        db_password: db_password_plaintext,
    })
}

pub async fn get_pool_for_tenant(
    tenant_id: &Uuid,
    state: &AppState,
    pool: &PgPool,
) -> Result<PgPool, HttpResponse> {
    let mut pools = state.pools.lock().unwrap();
    if let Some(tenant_pool) = pools.get_mut(&tenant_id) {
        tenant_pool.last_accessed = Utc::now();
        return Ok(tenant_pool.pool.clone());
    }
    let credentials = fetch_tenant_db_credentials(&pool, &tenant_id)
        .await
        .map_err(|_| HttpResponse::InternalServerError().body("Failed to create pool"))?;

    let connection_str = format!(
        "postgres://{}:{}@localhost:5630/db_7090a5",
        credentials.db_user, credentials.db_password
    );

    let pool = PgPoolOptions::new()
        .max_connections(5)
        .after_connect(move |conn, _| {
            Box::pin({
                let role = credentials.db_user.clone();

                async move {
                    // Set the role for this connection
                    sqlx::query(&format!("SET ROLE {}", &role))
                        .execute(conn)
                        .await?;
                    Ok(())
                }
            })
        })
        .idle_timeout(Some(Duration::from_secs(600))) // Set idle timeout for connections
        .connect(&connection_str)
        .await
        .map_err(|_| HttpResponse::InternalServerError().body("Failed to create pool"))?;

    pools.insert(
        *tenant_id,
        TenantPool {
            pool: pool.clone(),
            last_accessed: Utc::now(),
        },
    );

    Ok(pool)
}

pub async fn cleanup_idle_tenant_pools(state: &web::Data<AppState>, idle_duration_in_seconds: u64) {
    let mut pools = state.pools.lock().unwrap();
    let now = Utc::now();
    let idle_duration = Duration::from_secs(idle_duration_in_seconds);

    pools.retain(|_, tenant_pool| {
        now.signed_duration_since(tenant_pool.last_accessed)
            .num_seconds()
            < idle_duration.as_secs() as i64
    });
}

pub fn get_tenant_id_from_request(req: &HttpRequest) -> Result<Uuid, HttpResponse> {
    match req.headers().get("x-tenant-id") {
        Some(tenant_id) => {
            match tenant_id.to_str() {
                Ok(tenant_id_str) => match Uuid::parse_str(tenant_id_str) {
                    Ok(uuid) => Ok(uuid),
                    Err(_) => Err(HttpResponse::BadRequest()
                        .body("Invalid UUID format in x-tenant-id header")),
                },
                Err(_) => Err(HttpResponse::BadRequest().body("Invalid x-tenant-id header")),
            }
        }
        None => Err(HttpResponse::BadRequest().body("x-tenant-id header missing")),
    }
}
