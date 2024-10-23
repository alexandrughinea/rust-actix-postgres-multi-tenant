use crate::models::AppState;
use actix_web::web;
use chrono::Utc;
use std::time::Duration;

#[tracing::instrument(
    name = "Running cleanup idle tenant pools",
    skip(state, idle_duration_in_seconds)
)]
pub async fn cleanup_idle_tenant_pools(state: &web::Data<AppState>, idle_duration_in_seconds: u64) {
    let mut pools = state.pools.lock().await;
    let now = Utc::now();
    let idle_duration = Duration::from_secs(idle_duration_in_seconds);
    let mut pools_to_retain = Vec::new();

    // Check each pool
    for (key, tenant_pool) in pools.iter() {
        let tenant_pool = tenant_pool.lock().await;

        if now
            .signed_duration_since(tenant_pool.last_accessed)
            .num_seconds()
            < idle_duration.as_secs() as i64
        {
            pools_to_retain.push(*key);
        }
    }

    // Now use retain with a synchronous closure
    pools.retain(|key, _| pools_to_retain.contains(key));
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::TenantPool;
    use chrono::{DateTime, Duration as ChronoDuration, Utc};
    use sqlx::postgres::PgPoolOptions;
    use sqlx::PgPool;
    use std::{collections::HashMap, sync::Arc};
    use tokio::sync::Mutex;
    use uuid::Uuid;

    // Mock pool that doesn't try to connect to a database
    #[derive(Clone)]
    struct MockPool;
    impl MockPool {
        fn into_pg_pool() -> PgPool {
            // Create a pool with options that won't actually connect
            PgPoolOptions::new()
                .max_connections(1)
                .connect_lazy("postgres://mock:mock@localhost/mock")
                .unwrap()
        }
    }

    async fn create_test_state(pools_data: Vec<(Uuid, DateTime<Utc>)>) -> web::Data<AppState> {
        let mut pools = HashMap::new();

        for (id, last_accessed) in pools_data {
            let tenant_pool = Arc::new(Mutex::new(TenantPool {
                pool: Arc::new(MockPool::into_pg_pool()),
                last_accessed,
            }));
            pools.insert(id, tenant_pool);
        }

        web::Data::new(AppState {
            pools: Arc::new(Mutex::new(pools)),
        })
    }

    #[tokio::test]
    async fn test_cleanup_removes_idle_pools() {
        // Create test data
        let now = Utc::now();
        let old_time = now - ChronoDuration::seconds(100);
        let recent_time = now - ChronoDuration::seconds(10);

        let tenant1 = Uuid::new_v4();
        let tenant2 = Uuid::new_v4();
        let tenant3 = Uuid::new_v4();

        let pools_data = vec![
            (tenant1, old_time),    // Should be removed
            (tenant2, recent_time), // Should be kept
            (tenant3, old_time),    // Should be removed
        ];

        let state = create_test_state(pools_data).await;

        // Run cleanup with 30 second idle duration
        cleanup_idle_tenant_pools(&state, 30).await;

        // Verify results
        let pools = state.pools.lock().await;
        assert_eq!(pools.len(), 1, "Should only retain one pool");
        assert!(
            pools.contains_key(&tenant2),
            "Recent pool should be retained"
        );
        assert!(!pools.contains_key(&tenant1), "Old pool should be removed");
        assert!(!pools.contains_key(&tenant3), "Old pool should be removed");
    }

    #[tokio::test]
    async fn test_cleanup_keeps_all_active_pools() {
        let now = Utc::now();
        let recent_time = now - ChronoDuration::seconds(5);

        let tenant1 = Uuid::new_v4();
        let tenant2 = Uuid::new_v4();

        let pools_data = vec![(tenant1, recent_time), (tenant2, recent_time)];

        let state = create_test_state(pools_data).await;

        // Run cleanup with 30 second idle duration
        cleanup_idle_tenant_pools(&state, 30).await;

        // Verify all pools are kept
        let pools = state.pools.lock().await;
        assert_eq!(pools.len(), 2, "Should retain all pools");
        assert!(pools.contains_key(&tenant1));
        assert!(pools.contains_key(&tenant2));
    }

    #[tokio::test]
    async fn test_cleanup_removes_all_idle_pools() {
        let now = Utc::now();
        let old_time = now - ChronoDuration::seconds(100);

        let tenant1 = Uuid::new_v4();
        let tenant2 = Uuid::new_v4();

        let pools_data = vec![(tenant1, old_time), (tenant2, old_time)];

        let state = create_test_state(pools_data).await;

        // Run cleanup with 30 second idle duration
        cleanup_idle_tenant_pools(&state, 30).await;

        // Verify all pools are removed
        let pools = state.pools.lock().await;
        assert_eq!(pools.len(), 0, "Should remove all pools");
    }

    #[tokio::test]
    async fn test_cleanup_with_empty_pools() {
        let state = create_test_state(vec![]).await;

        // Run cleanup
        cleanup_idle_tenant_pools(&state, 30).await;

        // Verify state
        let pools = state.pools.lock().await;
        assert_eq!(pools.len(), 0, "Empty pools should remain empty");
    }
}
