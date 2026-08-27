use chrono::{DateTime, Utc};
use sqlx::PgPool;
use std::sync::atomic::{AtomicI64, Ordering};

pub struct TenantPool {
    pub pool: PgPool,
    last_accessed: AtomicI64,
}

impl TenantPool {
    pub fn new(pool: PgPool) -> Self {
        Self::with_last_accessed(pool, Utc::now())
    }

    pub fn with_last_accessed(pool: PgPool, last_accessed: DateTime<Utc>) -> Self {
        Self {
            pool,
            last_accessed: AtomicI64::new(last_accessed.timestamp()),
        }
    }

    pub fn touch(&self) {
        self.last_accessed
            .store(Utc::now().timestamp(), Ordering::Relaxed);
    }

    pub fn is_idle(&self, idle_secs: i64) -> bool {
        let last = self.last_accessed.load(Ordering::Relaxed);
        Utc::now().timestamp().saturating_sub(last) >= idle_secs
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Duration;
    use sqlx::postgres::PgPoolOptions;

    fn mock_pool() -> PgPool {
        PgPoolOptions::new()
            .max_connections(1)
            .connect_lazy("postgres://mock:mock@localhost/mock")
            .unwrap()
    }

    #[tokio::test]
    async fn is_idle_when_last_accessed_exceeds_threshold() {
        let last = Utc::now() - Duration::seconds(100);
        let tenant_pool = TenantPool::with_last_accessed(mock_pool(), last);
        assert!(tenant_pool.is_idle(30));
    }

    #[tokio::test]
    async fn is_not_idle_when_recently_accessed() {
        let last = Utc::now() - Duration::seconds(5);
        let tenant_pool = TenantPool::with_last_accessed(mock_pool(), last);
        assert!(!tenant_pool.is_idle(30));
    }

    #[tokio::test]
    async fn is_idle_at_exact_threshold() {
        let last = Utc::now() - Duration::seconds(30);
        let tenant_pool = TenantPool::with_last_accessed(mock_pool(), last);
        assert!(tenant_pool.is_idle(30));
    }

    #[tokio::test]
    async fn touch_clears_idle_state() {
        let last = Utc::now() - Duration::seconds(100);
        let tenant_pool = TenantPool::with_last_accessed(mock_pool(), last);
        tenant_pool.touch();
        assert!(!tenant_pool.is_idle(30));
    }
}
