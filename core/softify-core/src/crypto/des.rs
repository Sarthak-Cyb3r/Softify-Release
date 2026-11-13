use base64::engine::general_purpose::STANDARD as BASE64_STANDARD;
use base64::Engine;
use cipher::{Block, BlockDecrypt, KeyInit};
#[cfg(test)]
use cipher::BlockEncrypt;
use des::Des;

/// DES-ECB Key for JioSaavn encrypted media URLs.
/// ASCII bytes for "38346591": [51, 56, 51, 52, 54, 53, 57, 49]
pub const JIOSAAVN_DES_KEY: &[u8; 8] = b"38346591";

/// Decrypts JioSaavn encrypted_media_url string using DES-ECB and upgrades stream to 320 Kbps.
pub fn decrypt_jiosaavn_url(encrypted_b64: &str) -> Result<String, String> {
    let raw_decrypted = decrypt_des_ecb(encrypted_b64)?;
    Ok(upgrade_to_320k(&raw_decrypted))
}

/// Decrypts base64 encoded ciphertext using DES-ECB with JioSaavn key and strips PKCS5/PKCS7 padding.
pub fn decrypt_des_ecb(encrypted_b64: &str) -> Result<String, String> {
    let trimmed = encrypted_b64.trim();
    if trimmed.is_empty() {
        return Err("Encrypted media URL is empty".to_string());
    }

    let ciphertext = BASE64_STANDARD
        .decode(trimmed)
        .map_err(|e| format!("Base64 decode error: {e}"))?;

    if ciphertext.is_empty() || ciphertext.len() % 8 != 0 {
        return Err(format!(
            "Ciphertext length {} is not a multiple of 8",
            ciphertext.len()
        ));
    }

    let cipher = Des::new_from_slice(JIOSAAVN_DES_KEY)
        .map_err(|e| format!("DES cipher init failed: {e}"))?;

    let mut decrypted = ciphertext;
    for chunk in decrypted.chunks_exact_mut(8) {
        let block = Block::<Des>::from_mut_slice(chunk);
        cipher.decrypt_block(block);
    }

    // PKCS5/PKCS7 unpadding
    let pad_len = *decrypted.last().ok_or_else(|| "Empty buffer".to_string())? as usize;
    if pad_len == 0 || pad_len > 8 || pad_len > decrypted.len() {
        return Err(format!("Invalid PKCS padding length: {pad_len}"));
    }

    for &byte in &decrypted[decrypted.len() - pad_len..] {
        if byte as usize != pad_len {
            return Err("Corrupted PKCS padding bytes".to_string());
        }
    }

    decrypted.truncate(decrypted.len() - pad_len);

    String::from_utf8(decrypted).map_err(|e| format!("Decrypted string is not valid UTF-8: {e}"))
}

/// Replaces lower bitrates (_96.mp4, _160.mp4) with 320 Kbps CD-quality studio master (_320.mp4).
pub fn upgrade_to_320k(raw_url: &str) -> String {
    raw_url
        .replace("_96.mp4", "_320.mp4")
        .replace("_160.mp4", "_320.mp4")
}

/// Test helper to encrypt raw plaintext using DES-ECB with PKCS7 padding and return Base64.
#[cfg(test)]
pub fn encrypt_des_ecb(plaintext: &str) -> Result<String, String> {
    let cipher = Des::new_from_slice(JIOSAAVN_DES_KEY)
        .map_err(|e| format!("DES cipher init failed: {e}"))?;

    let bytes = plaintext.as_bytes();
    let pad_len = 8 - (bytes.len() % 8);
    let mut padded = bytes.to_vec();
    padded.extend(std::iter::repeat(pad_len as u8).take(pad_len));

    for chunk in padded.chunks_exact_mut(8) {
        let block = Block::<Des>::from_mut_slice(chunk);
        cipher.encrypt_block(block);
    }

    Ok(BASE64_STANDARD.encode(&padded))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_des_key_integrity() {
        assert_eq!(JIOSAAVN_DES_KEY, &[51, 56, 51, 52, 54, 53, 57, 49]);
        assert_eq!(JIOSAAVN_DES_KEY, b"38346591");
    }

    #[test]
    fn test_320k_upgrade_rewrite() {
        let url_96 = "https://aac.saavncdn.com/100/sample_96.mp4";
        assert_eq!(
            upgrade_to_320k(url_96),
            "https://aac.saavncdn.com/100/sample_320.mp4"
        );

        let url_160 = "https://aac.saavncdn.com/100/sample_160.mp4";
        assert_eq!(
            upgrade_to_320k(url_160),
            "https://aac.saavncdn.com/100/sample_320.mp4"
        );

        let url_320 = "https://aac.saavncdn.com/100/sample_320.mp4";
        assert_eq!(
            upgrade_to_320k(url_320),
            "https://aac.saavncdn.com/100/sample_320.mp4"
        );
    }

    #[test]
    fn test_encrypt_decrypt_roundtrip_with_320k_upgrade() {
        let original_url = "https://aac.saavncdn.com/284/d41d8cd98f00b204e9800998ecf8427e_96.mp4";
        let encrypted_b64 = encrypt_des_ecb(original_url).expect("Encryption failed");

        let resolved_320 =
            decrypt_jiosaavn_url(&encrypted_b64).expect("Decryption and upgrade failed");
        assert_eq!(
            resolved_320,
            "https://aac.saavncdn.com/284/d41d8cd98f00b204e9800998ecf8427e_320.mp4"
        );
    }

    #[test]
    fn test_corrupted_ciphertext_fails() {
        assert!(decrypt_jiosaavn_url("invalid-base64!").is_err());
        assert!(decrypt_jiosaavn_url("").is_err());
        let odd_b64 = BASE64_STANDARD.encode(b"short");
        assert!(decrypt_jiosaavn_url(&odd_b64).is_err());
    }
}
