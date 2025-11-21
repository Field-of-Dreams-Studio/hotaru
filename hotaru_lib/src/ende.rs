pub mod aes {
    use aes_gcm::{
        Aes256Gcm, Nonce,
        aead::{Aead, KeyInit},
    };
    use base64::{Engine as _, engine::general_purpose::STANDARD as BASE64};
    use hkdf::Hkdf; 
    use rand::rngs::OsRng;
    use rand::TryRngCore; 
    use sha2::Sha256;

    pub struct EncryptedData {
        salt: [u8; 16],
        nonce: [u8; 12],     // AES-GCM uses a 12-byte nonce
        ciphertext: Vec<u8>, // Includes authentication tag
    }

    // Generate secure key from password
    // pub fn derive_key(password: &str, salt: &[u8]) -> [u8; 32] {
    //     let mut key = [0u8; 32];
    //     pbkdf2::pbkdf2::<hmac::Hmac<Sha256>>(
    //         password.as_bytes(),
    //         salt,
    //         100_000, // Increased iteration count for 2024 security standards
    //         &mut key,
    //     )
    //     .expect("PBKDF2 should not fail");
    //     key
    // } 

    pub fn derive_key(password: &str, salt: &[u8]) -> [u8; 32] {
        let mut key = [0u8; 32];
        let info = password.as_bytes();
        Hkdf::<Sha256>::new(None, salt)
            .expand(info, &mut key)
            .unwrap();
        key
    } 

    // Encrypt data with AES-256-GCM
    pub fn encrypt_struct(plaintext: &[u8], password: &str) -> Result<EncryptedData, String> {
        let mut salt = [0u8; 16];
        let mut nonce = [0u8; 12];
        let mut rng = OsRng;

        rng.try_fill_bytes(&mut salt)
            .map_err(|e| format!("Failed to fill salt: {}", e))?;
        rng.try_fill_bytes(&mut nonce)
            .map_err(|e| format!("Failed to fill nonce: {}", e))?;

        // Derive encryption key
        let key = derive_key(password, &salt);

        // Create cipher instance
        let cipher = Aes256Gcm::new_from_slice(&key).map_err(|e| format!("Key error: {}", e))?;

        // Encrypt the data
        let nonce_ref = Nonce::from_slice(&nonce);
        let ciphertext = cipher
            .encrypt(nonce_ref, plaintext.as_ref())
            .map_err(|e| format!("Encryption failed: {}", e))?;

        Ok(EncryptedData {
            salt,
            nonce,
            ciphertext,
        })
    }

    pub fn encrypt(plaintext: &str, password: &str) -> Result<String, String> {
        encrypt_struct(plaintext.as_bytes(), password)
            .map(|data| serialize_encrypted_data(&data))
    }

    // Decrypt data with AES-256-GCM
    pub fn decrypt_struct(
        encrypted: &EncryptedData,
        password: &str,
    ) -> Result<Vec<u8>, String> {
        // Derive the same key using the stored salt
        let key = derive_key(password, &encrypted.salt);

        // Create cipher instance
        let cipher = Aes256Gcm::new_from_slice(&key).map_err(|e| format!("Key error: {}", e))?;

        // Decrypt the data
        let nonce = Nonce::from_slice(&encrypted.nonce);
        let plaintext = cipher
            .decrypt(nonce, encrypted.ciphertext.as_ref())
            .map_err(|e| {
                format!(
                    "Decryption failed (likely wrong password or tampered data): {}",
                    e
                )
            })?;

        Ok(plaintext)
    }

    pub fn decrypt(serialized: &str, password: &str) -> Result<String, String> {
        let encrypted_data = deserialize_encrypted_data(serialized)?;
        decrypt_struct(&encrypted_data, password)
            .map(|plaintext| String::from_utf8(plaintext).map_err(|e| e.to_string()))
            .and_then(|s| s.map_err(|e| format!("Decryption resulted in invalid UTF-8: {}", e)))
    }

    // Serialize encrypted data to string (for storage or transmission)
    pub fn serialize_encrypted_data(data: &EncryptedData) -> String {
        let mut serialized = Vec::new();
        serialized.extend_from_slice(&data.salt);
        serialized.extend_from_slice(&data.nonce);
        serialized.extend_from_slice(&data.ciphertext);
        BASE64.encode(serialized)
    }

    // Deserialize from string back to EncryptedData
    pub fn deserialize_encrypted_data(serialized: &str) -> Result<EncryptedData, String> {
        let decoded = BASE64
            .decode(serialized)
            .map_err(|e| format!("Base64 decoding failed: {}", e))?;

        if decoded.len() < 16 + 12 {
            return Err("Data too short to contain salt and nonce".to_string());
        }

        let mut salt = [0u8; 16];
        let mut nonce = [0u8; 12];

        salt.copy_from_slice(&decoded[0..16]);
        nonce.copy_from_slice(&decoded[16..28]);

        let ciphertext = decoded[28..].to_vec();

        Ok(EncryptedData {
            salt,
            nonce,
            ciphertext,
        }) 
    }
}

#[cfg(test)]
mod test {
    #[test]
    fn test1() {
        let password = "test_password";
        let plaintext = "Hello, World!";

        // Encrypt
        let encrypted = super::aes::encrypt(plaintext, password).expect("Encryption failed");

        println!("Encrypted text: {}", encrypted);

        // Decrypt
        let decrypted = super::aes::decrypt(&encrypted, password).expect("Decryption failed");

        println!("Decrypted text: {}", decrypted);

        assert_eq!(decrypted, plaintext);
    }
}
