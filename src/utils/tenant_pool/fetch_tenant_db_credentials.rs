use crate::configurations::Configuration;
use crate::models::TenantCredentials;
use crate::utils::decrypt_aes_gcm;
use secrecy::{ExposeSecret, SecretString};
use sqlx::PgPool;
use uuid::Uuid;

#[tracing::instrument(
    name = "Fetching tenant db credentials for tenant.",
    fields(tenant_id = %tenant_id),
    skip(pool, configuration)
)]
pub async fn fetch_tenant_db_credentials(
    tenant_id: &Uuid,
    pool: &PgPool,
    configuration: &Configuration,
) -> Result<TenantCredentials, Box<dyn std::error::Error>> {
    let tenant = sqlx::query_as!(Tenant, "SELECT * FROM tenants WHERE id = $1", tenant_id)
        .fetch_one(pool)
        .await?;

    let decryption_key = configuration.secrets.aes256_gcm_key.expose_secret();
    let db_password_encrypted = tenant.db_password_encrypted.unwrap();
    let db_password_plaintext = decrypt_aes_gcm(decryption_key, db_password_encrypted.as_str())?;

    Ok(TenantCredentials {
        db_user: tenant.db_user,
        db_password: SecretString::from(db_password_plaintext),
    })
}
