use actix_session::config::CookieContentSecurity;
use actix_web::cookie::SameSite;
use config::{Config, ConfigError, File};
use secrecy::{ExposeSecret, SecretString};
use serde::Deserialize;
use sqlx::postgres::{PgConnectOptions, PgSslMode};
use sqlx::ConnectOptions;
use std::env;

#[derive(Deserialize, Clone)]
pub struct Settings {
    pub debug: bool,
    pub database: DatabaseSettings,
    pub application: ApplicationSettings,
    pub redis: RedisSettings,
    pub secret: SecretSettings,
    pub frontend_url: Option<String>,
}

#[derive(Deserialize, Clone)]
pub struct ApplicationSettings {
    pub port: u16,
    pub host: String,
    pub cookie: CookieSettings,
    pub certificate: CertificateSettings,
}

#[derive(Deserialize, Debug, Clone)]
pub struct CookieContentSecurityWrapper(String);

impl TryFrom<CookieContentSecurityWrapper> for CookieContentSecurity {
    type Error = String;

    fn try_from(wrapper: CookieContentSecurityWrapper) -> Result<Self, Self::Error> {
        match wrapper.0.to_lowercase().as_str() {
            "private" => Ok(CookieContentSecurity::Private),
            "signed" => Ok(CookieContentSecurity::Signed),
            _ => Err(format!(
                "Invalid CookieContentSecurity value: {}",
                wrapper.0
            )),
        }
    }
}

#[derive(Deserialize, Debug, Clone)]
pub struct CookieSameSiteWrapper(String);

impl TryFrom<CookieSameSiteWrapper> for SameSite {
    type Error = String;

    fn try_from(wrapper: CookieSameSiteWrapper) -> Result<Self, Self::Error> {
        match wrapper.0.to_lowercase().as_str() {
            "lax" => Ok(SameSite::Lax),
            "signed" => Ok(SameSite::Strict),
            "none" => Ok(SameSite::None),
            _ => Err(format!("Invalid SameSite value: {}", wrapper.0)),
        }
    }
}

#[derive(Deserialize, Clone)]
pub struct CertificateSettings {
    pub cert_path: String,
    pub key_path: String,
    pub is_key_encrypted: bool,
}

#[derive(Deserialize, Clone)]
pub struct CookieSettings {
    pub secure: bool,
    pub content_security: CookieContentSecurityWrapper,
    pub domain: Option<String>,
    pub name: String,
    pub path: String,
    pub http_only: bool,
    pub same_site: CookieSameSiteWrapper,
    pub session_ttl: i64,
}

#[derive(Deserialize, Clone)]
pub struct DatabaseSettings {
    pub username: String,
    pub password: SecretString,
    pub port: u16,
    pub host: String,
    pub database_name: String,
    pub require_ssl: bool,
    pub max_connections: u32,
}

#[derive(Deserialize, Clone, Debug)]
pub struct RedisSettings {
    pub username: Option<String>,
    pub password: Option<String>,
    pub port: u16,
    pub host: String,
    pub database_index: Option<u8>,
    pub tls: bool,
    pub pool_max_size: usize,
    pub pool_timeout_in_seconds: u64,
}

#[derive(Deserialize, Clone)]
pub struct SecretSettings {
    pub argon2_salt: SecretString,
    pub argon2_secret_key: SecretString,

    pub hmac_secret: SecretString,
    pub aes256_gcm_secret_key: SecretString,
}

pub enum Environment {
    Development,
    Production,
}

impl Environment {
    pub fn as_str(&self) -> &'static str {
        match self {
            Environment::Development => "development",
            Environment::Production => "production",
        }
    }
}

impl TryFrom<String> for Environment {
    type Error = String;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        match value.to_lowercase().as_str() {
            "development" => Ok(Self::Development),
            "production" => Ok(Self::Production),
            other => Err(format!(
                "{} is not a supported environment. Use either `local` or `production`.",
                other
            )),
        }
    }
}

pub fn get_configuration() -> Result<Settings, ConfigError> {
    let base_path = env::current_dir().expect("Failed to determine the current directory");
    let configuration_directory = base_path.join("configuration");

    // Detect the running environment.
    // Default to `development` if unspecified.
    let environment: Environment = env::var("APP_ENVIRONMENT")
        .unwrap_or_else(|_| "development".into())
        .try_into()
        .expect("Failed to parse APP_ENVIRONMENT.");
    let environment_filename = format!("{}.yaml", environment.as_str());
    let settings = Config::builder()
        .add_source(File::from(configuration_directory.join("base.yaml")))
        .add_source(File::from(
            configuration_directory.join(environment_filename),
        ))
        // Add in settings from environment variables (with a prefix of APP and '__' as separator)
        // E.g. `APP_APPLICATION__PORT=5001 would set `Settings.application.port`
        .add_source(config::Environment::with_prefix("app").separator("__"))
        .build()?;

    settings.try_deserialize::<Settings>()
}

impl DatabaseSettings {
    pub fn with_db(&self) -> PgConnectOptions {
        let options = self.without_db().database(&self.database_name);

        options
            .clone()
            .log_statements(tracing::log::LevelFilter::Trace);

        options
    }

    pub fn without_db(&self) -> PgConnectOptions {
        let ssl_mode = if self.require_ssl {
            PgSslMode::Require
        } else {
            PgSslMode::Prefer
        };
        PgConnectOptions::new()
            .host(&self.host)
            .username(&self.username)
            .password(self.password.expose_secret())
            .port(self.port)
            .ssl_mode(ssl_mode)
    }
}

impl RedisSettings {
    pub fn redis_url(&self) -> String {
        let protocol = if self.tls { "rediss" } else { "redis" };

        match (&self.username, &self.password, &self.database_index) {
            (Some(username), Some(password), Some(database_index)) => {
                format!(
                    "{}://{}:{}@{}:{}/{}",
                    protocol, username, password, self.host, self.port, database_index
                )
            }
            (Some(username), Some(password), None) => {
                format!(
                    "{}://{}:{}@{}:{}",
                    protocol, username, password, self.host, self.port
                )
            }
            (None, None, Some(database_index)) => {
                format!(
                    "{}://{}:{}/{}",
                    protocol, self.host, self.port, database_index
                )
            }
            _ => format!("{}://{}:{}", protocol, self.host, self.port),
        }
    }
}
