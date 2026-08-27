use crate::configurations::{Configuration, DatabaseConfiguration};
use crate::models::{AppState, TenantPool};
use crate::utils::fetch_tenant_db_credentials;
use actix_web::HttpResponse;
use dashmap::mapref::entry::Entry::{Occupied, Vacant};
use sqlx::postgres::PgPoolOptions;
use sqlx::PgPool;
use std::time::Duration;
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
) -> Result<PgPool, HttpResponse> {
    if let Some(existing) = state.pools.get(tenant_id) {
        existing.touch();
        return Ok(existing.pool.clone());
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

    let new_pool = pool_options
        .connect_with(connect_options)
        .await
        .map_err(|_| {
            HttpResponse::InternalServerError().body("Failed to create dedicated tenant pool.")
        })?;

    let lost_race = {
        match state.pools.entry(*tenant_id) {
            Occupied(e) => {
                e.get().touch();
                Some(e.get().pool.clone())
            }
            Vacant(e) => {
                e.insert(TenantPool::new(new_pool.clone()));
                None
            }
        }
    };

    if let Some(existing) = lost_race {
        new_pool.close().await;
        Ok(existing)
    } else {
        Ok(new_pool)
    }
}
