use secrecy::SecretString;
use serde::{Deserialize, Serialize};
use sqlx::FromRow;

#[derive(Serialize, Deserialize, FromRow)]
pub struct TenantCredentials {
    pub db_user: String,
    #[serde(skip)]
    pub db_password: SecretString,
}
