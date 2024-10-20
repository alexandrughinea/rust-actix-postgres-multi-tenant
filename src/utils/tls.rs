use crate::models::AppError;
use openssl::pkey::{PKey, Private};
use openssl::ssl::{SslAcceptor, SslAcceptorBuilder, SslFiletype, SslMethod};
use std::fs::File;
use std::io::Read;

#[tracing::instrument(name = "Loading encrypted private key", skip(key_path))]
fn load_encrypted_private_key(key_path: &str) -> Result<PKey<Private>, AppError> {
    tracing::info!("Loading encrypted private key from: {}", key_path);
    let mut file = File::open(key_path)?;
    let mut key_data_buffer = Vec::new();
    file.read_to_end(&mut key_data_buffer)?;

    tracing::error!("Encrypted key decryption not implemented");

    match PKey::private_key_from_pem_passphrase(&key_data_buffer, b"password") {
        Ok(result) => Ok(result),
        Err(_) => Err(AppError::EncryptedKeyError(
            "Decryption not implemented".to_string(),
        )),
    }
}

pub fn build_tls_config(
    cert_path: &str,
    key_path: &str,
    is_key_encrypted: &bool,
) -> Result<SslAcceptorBuilder, AppError> {
    tracing::info!("Building TLS configuration");
    let mut builder = SslAcceptor::mozilla_intermediate(SslMethod::tls())?;

    // Set the certs chain file
    tracing::info!("Setting certs chain file: {}", cert_path);
    builder.set_certificate_chain_file(cert_path)?;

    // Set the private key
    if *is_key_encrypted {
        tracing::info!("Using encrypted private key");
        let encrypted_key = load_encrypted_private_key(key_path)?;
        builder.set_private_key(&encrypted_key)?;
    } else {
        tracing::info!("Using unencrypted private key file: {}", key_path);
        builder.set_private_key_file(key_path, SslFiletype::PEM)?;
    }

    tracing::info!("TLS configuration built successfully");

    Ok(builder)
}
