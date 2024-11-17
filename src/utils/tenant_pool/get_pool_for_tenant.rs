use crate::configurations::{Configuration, DatabaseConfiguration};
use crate::models::{AppState, TenantPool};
use crate::utils::fetch_tenant_db_credentials;
use actix_web::HttpResponse;
use chrono::Utc;
use sqlx::postgres::PgPoolOptions;
use sqlx::PgPool;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::Mutex;
use uuid::Uuid;

#[tracing::instrument(
    name = "Fetching connection pool for tenant.",
    fields(tenant_id = %tenant_id),
    skip(state, pool, configuration)
)]
pub async fn get_pool_for_tenant(
    tenant_id: &Uuid,
    state: &AppState,
    pool: &PgPool,
    configuration: &Configuration,
) -> Result<Arc<PgPool>, HttpResponse> {
    let mut pools = state.pools.lock().await;
    // Check if the tenant's pool already exists
    if let Some(tenant_pool) = pools.get_mut(tenant_id) {
        // Lock the Mutex to mutate the TenantPool
        let mut tenant_pool = tenant_pool.lock().await;

        // Mutate the last accessed (visited) time
        tenant_pool.last_accessed = Utc::now();

        return Ok(Arc::clone(&tenant_pool.pool));
    }

    // Fetch credentials for the tenant's database
    let credentials = fetch_tenant_db_credentials(tenant_id, pool, configuration)
        .await
        .map_err(|_| {
            HttpResponse::InternalServerError().body("Failed to fetch tenant credentials")
        })?;

    let database_configuration = DatabaseConfiguration {
        username: credentials.db_user.clone(),
        password: credentials.db_password,
        port: configuration.database.port,
        host: configuration.database.host.clone(),
        database_name: configuration.database.database_name.clone(),
        require_ssl: configuration.database.require_ssl,
        min_connections: None,
        max_connections: None,
        acquire_timeout: None,
        max_lifetime: None,
        idle_timeout: None,
    };
    let connect_options = database_configuration.with_db();
    let mut pool_options = PgPoolOptions::new();

    // Only set min_connections if provided
    if let Some(min_connections) = configuration.database.min_connections {
        pool_options = pool_options.min_connections(min_connections);
    }

    // Only set max_connections if provided
    if let Some(max_connections) = configuration.database.max_connections {
        pool_options = pool_options.max_connections(max_connections);
    }

    // Only set acquire_timeout if provided
    if let Some(acquire_timeout) = configuration.database.acquire_timeout {
        pool_options = pool_options.acquire_timeout(Duration::from_millis(acquire_timeout));
    }

    // Only set max_lifetime if provided
    if let Some(max_lifetime) = configuration.database.max_lifetime {
        pool_options = pool_options.max_lifetime(Duration::from_secs(max_lifetime));
    }

    // Only set idle_timeout if provided
    if let Some(idle_timeout) = configuration.database.idle_timeout {
        pool_options = pool_options.idle_timeout(Duration::from_secs(idle_timeout));
    }

    let pool = pool_options
        .connect_with(connect_options)
        .await
        .map_err(|_| {
            HttpResponse::InternalServerError().body("Failed to create dedicated tenant pool.")
        })?;

    // Create a new `TenantPool` struct with the newly created pool (wrapped in an Arc)
    let tenant_pool = Arc::new(Mutex::new(TenantPool {
        pool: Arc::new(pool),
        last_accessed: Utc::now(),
    }));

    pools.insert(*tenant_id, Arc::clone(&tenant_pool));

    // Lock the Mutex to access the pool
    let tenant_pool = tenant_pool.lock().await;

    Ok(Arc::clone(&tenant_pool.pool)) // Clone the Arc to return
}
