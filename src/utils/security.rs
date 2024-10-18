use aes_gcm::{
    aead::{Aead, KeyInit},
    Aes256Gcm, Nonce,
};
use hex;
use rand::Rng;

fn decode_aes_hex_key(key_hex: &str) -> Result<Vec<u8>, String> {
    match hex::decode(key_hex) {
        Ok(key) => {
            println!("Decoded AES hex key: {:?}", key);
            Ok(key)
        }
        Err(e) => {
            let message = format!("Failed to decode AES hex key: {:?}", e);
            eprintln!("{}", message);
            Err(message)
        }
    }
}

pub fn encrypt_aes_gcm(
    key_hex: &str,
    plaintext: &str,
) -> Result<String, Box<dyn std::error::Error>> {
    let decoded_aes_hex_key = decode_aes_hex_key(key_hex)?;
    let cipher = Aes256Gcm::new_from_slice(&*decoded_aes_hex_key)
        .map_err(|e| format!("Failed to create cipher: {:?}", e))?;

    let nonce = generate_nonce();
    let ciphertext = cipher
        .encrypt(Nonce::from_slice(&nonce), plaintext.as_bytes())
        .map_err(|e| format!("Encryption failed: {:?}", e))?;

    let mut encrypted = nonce.to_vec();
    encrypted.extend_from_slice(&ciphertext);

    Ok(hex::encode(encrypted))
}

pub fn decrypt_aes_gcm(
    key_hex: &str,
    encrypted_data: &str,
) -> Result<String, Box<dyn std::error::Error>> {
    let decoded_aes_hex_key = decode_aes_hex_key(key_hex)?;
    let encrypted_bytes = hex::decode(encrypted_data)?;

    if encrypted_bytes.len() < 12 {
        return Err("Encrypted data is too short".into());
    }

    let (nonce, ciphertext) = encrypted_bytes.split_at(12);

    let cipher = Aes256Gcm::new_from_slice(&*decoded_aes_hex_key)
        .map_err(|e| format!("Failed to create cipher: {:?}", e))?;

    let plaintext = cipher
        .decrypt(Nonce::from_slice(nonce), ciphertext)
        .map_err(|e| format!("Decryption failed: {:?}", e))?;

    String::from_utf8(plaintext).map_err(|e| e.into())
}

fn generate_nonce() -> [u8; 12] {
    let mut nonce = [0u8; 12];
    rand::thread_rng().fill(&mut nonce);
    nonce
}

#[cfg(test)]
mod tests {
    use super::*;

    const AES_KEY_HEX: &str = "4b5d623f8a9b2dc3e78f5c6a1d3b9f0e2a1c4b7d5e8f0a3c6b9d2e5f8a1c4d7b";
    const ORIGINAL_MESSAGE: &str = "secure_password_test_1";
    const ENCRYPTED_ORIGINAL_MESSAGE: &str = "3d0353bd1f90f4e2d6b001d0c5a9cc23fd65a4712f1d8f9452750d46fe36aaeec120ac11889138bb156e731194eca0d9ff59";

    #[test]
    fn test_encrypt_aes_gcm() {
        let encrypted = encrypt_aes_gcm(&AES_KEY_HEX, ORIGINAL_MESSAGE).unwrap();

        println!("Encrypted: {}", encrypted);

        // Verify that the encrypted result is a valid hex string and has the correct length
        assert!(hex::decode(&encrypted).is_ok());
        assert!(encrypted.len() > 24); // At least 12 bytes for nonce + some ciphertext
    }

    #[test]
    fn test_decrypt_aes_gcm() {
        let decrypted = decrypt_aes_gcm(&AES_KEY_HEX, ENCRYPTED_ORIGINAL_MESSAGE).unwrap();

        assert_eq!(decrypted, ORIGINAL_MESSAGE);
    }
}
