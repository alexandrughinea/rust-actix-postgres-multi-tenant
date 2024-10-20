use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use uuid::Uuid;

#[derive(Serialize, Deserialize, FromRow)]
pub struct Tenant {
    pub id: Option<Uuid>,
    pub name: String,

    #[serde(skip)]
    pub db_user: String,
    #[serde(skip)]
    pub db_password_encrypted: Option<String>,
    pub created_at: Option<DateTime<Utc>>,
    pub updated_at: Option<DateTime<Utc>>,
}
