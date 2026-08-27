use crate::models::AppState;
use actix_web::web;

#[tracing::instrument(
    name = "Running cleanup idle tenant pools",
    skip(state, idle_duration_in_seconds)
)]
pub async fn cleanup_idle_tenant_pools(state: &web::Data<AppState>, idle_duration_in_seconds: u64) {
    state
        .pools
        .retain(|_, tenant_pool| !tenant_pool.is_idle(idle_duration_in_seconds as i64));
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::TenantPool;
    use chrono::{DateTime, Duration as ChronoDuration, Utc};
    use dashmap::DashMap;
    use sqlx::postgres::PgPoolOptions;
    use sqlx::PgPool;
    use uuid::Uuid;

    fn mock_pg_pool() -> PgPool {
        PgPoolOptions::new()
            .max_connections(1)
            .connect_lazy("postgres://mock:mock@localhost/mock")
            .unwrap()
    }

    async fn create_test_state(pools_data: Vec<(Uuid, DateTime<Utc>)>) -> web::Data<AppState> {
        let pools = DashMap::new();

        for (id, last_accessed) in pools_data {
            pools.insert(
                id,
                TenantPool::with_last_accessed(mock_pg_pool(), last_accessed),
            );
        }

        web::Data::new(AppState { pools })
    }

    #[tokio::test]
    async fn test_cleanup_removes_idle_pools() {
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

        cleanup_idle_tenant_pools(&state, 30).await;

        assert_eq!(state.pools.len(), 1, "Should only retain one pool");
        assert!(
            state.pools.contains_key(&tenant2),
            "Recent pool should be retained"
        );
        assert!(
            !state.pools.contains_key(&tenant1),
            "Old pool should be removed"
        );
        assert!(
            !state.pools.contains_key(&tenant3),
            "Old pool should be removed"
        );
    }

    #[tokio::test]
    async fn test_cleanup_keeps_all_active_pools() {
        let now = Utc::now();
        let recent_time = now - ChronoDuration::seconds(5);

        let tenant1 = Uuid::new_v4();
        let tenant2 = Uuid::new_v4();

        let pools_data = vec![(tenant1, recent_time), (tenant2, recent_time)];

        let state = create_test_state(pools_data).await;

        cleanup_idle_tenant_pools(&state, 30).await;

        assert_eq!(state.pools.len(), 2, "Should retain all pools");
        assert!(state.pools.contains_key(&tenant1));
        assert!(state.pools.contains_key(&tenant2));
    }

    #[tokio::test]
    async fn test_cleanup_removes_all_idle_pools() {
        let now = Utc::now();
        let old_time = now - ChronoDuration::seconds(100);

        let tenant1 = Uuid::new_v4();
        let tenant2 = Uuid::new_v4();

        let pools_data = vec![(tenant1, old_time), (tenant2, old_time)];

        let state = create_test_state(pools_data).await;

        cleanup_idle_tenant_pools(&state, 30).await;

        assert_eq!(state.pools.len(), 0, "Should remove all pools");
    }

    #[tokio::test]
    async fn test_cleanup_with_empty_pools() {
        let state = create_test_state(vec![]).await;

        cleanup_idle_tenant_pools(&state, 30).await;

        assert_eq!(state.pools.len(), 0, "Empty pools should remain empty");
    }
}
