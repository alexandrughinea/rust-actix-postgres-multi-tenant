use actix_web::{http::StatusCode, HttpResponse, ResponseError};
use anyhow::Error as AnyhowError;
use config::ConfigError;
use sqlx::error::Error as SqlxError;
use sqlx::migrate::MigrateError;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum AppError {
    #[error("An OpenSSL error occurred.")]
    OpenSslError(#[from] openssl::error::ErrorStack),

    #[error("An IO error occurred. {0}")]
    IOError(#[from] std::io::Error),

    #[error("An encrypted key (SSL Certificate) error occurred. : {0}")]
    EncryptedKeyError(String),

    #[error("A database error occurred: {0}")]
    DatabaseError(#[from] SqlxError),

    #[error("A database migration error occurred: {0}")]
    DatabaseMigrationError(#[from] MigrateError),

    #[error("An internal error occurred. Please try again later.")]
    InternalError(#[from] AnyhowError),

    #[error("Authentication failed: {0}")]
    AuthenticationFailed(String),

    #[error("Authorization failed: {0}")]
    AuthorizationFailed(String),

    #[error("Not found")]
    NotFoundError,

    #[error("Bad request: {0}")]
    BadRequestError(String),

    #[error("Not content: {0}")]
    NoContentError(String),

    #[error("An unexpected auth error occurred: {0}")]
    Other(String),

    #[error("Config failed: {0}")]
    ConfigError(#[from] ConfigError),
}

impl ResponseError for AppError {
    fn status_code(&self) -> StatusCode {
        match *self {
            AppError::DatabaseError(_) | AppError::InternalError(_) => {
                StatusCode::INTERNAL_SERVER_ERROR
            }
            AppError::NotFoundError => StatusCode::NOT_FOUND,
            AppError::BadRequestError(_) => StatusCode::BAD_REQUEST,
            AppError::NoContentError(_) => StatusCode::NO_CONTENT,
            // Handle other error types
            _ => StatusCode::NOT_FOUND,
        }
    }

    fn error_response(&self) -> HttpResponse {
        match self {
            AppError::DatabaseError(_) => {
                HttpResponse::InternalServerError().body("Database error.")
            }
            AppError::InternalError(_) | AppError::OpenSslError(_) => {
                HttpResponse::InternalServerError().body("Something went terribly wrong.")
            }
            AppError::NotFoundError => HttpResponse::NotFound().body("Resource not found."),
            AppError::NoContentError(_) => HttpResponse::NoContent().body("No content available."),
            AppError::BadRequestError(_) => HttpResponse::BadRequest().body("Bad request"),
            _ => HttpResponse::InternalServerError().body("Something went terribly wrong."),
        }
    }
}
