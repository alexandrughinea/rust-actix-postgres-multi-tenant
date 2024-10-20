use chrono::{DateTime, Utc};
use sqlx::PgPool;
use std::sync::Arc;

pub struct TenantPool {
    pub pool: Arc<PgPool>,
    pub last_accessed: DateTime<Utc>,
}
