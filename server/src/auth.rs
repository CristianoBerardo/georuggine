use argon2::password_hash::{PasswordHash, PasswordVerifier, SaltString, rand_core::OsRng};
use argon2::{Argon2, PasswordHasher};

pub fn hash_password(password: &str) -> Result<String, Box<dyn std::error::Error + Send + Sync>> {
    let salt = SaltString::generate(&mut OsRng);
    let argon2 = Argon2::default();
    let hash = argon2.hash_password(password.as_bytes(), &salt)?;
    Ok(hash.to_string())
}

pub fn verify_password(hash: &str, password: &str) -> bool {
    if let Ok(parsed_hash) = PasswordHash::new(hash) {
        Argon2::default()
            .verify_password(password.as_bytes(), &parsed_hash)
            .is_ok()
    } else {
        false
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn password_corretta_viene_verificata() {
        let hash = hash_password("supersegreta").unwrap();
        assert!(verify_password(&hash, "supersegreta"));
    }

    #[test]
    fn password_sbagliata_viene_rifiutata() {
        let hash = hash_password("supersegreta").unwrap();
        assert!(!verify_password(&hash, "altra-password"));
    }

    #[test]
    fn hash_non_valido_non_va_in_panico() {
        assert!(!verify_password("non-un-hash", "qualsiasi"));
    }

    #[test]
    fn stessa_password_produce_hash_diversi() {
        // Il salt è casuale a ogni chiamata: due hash della stessa password
        // non devono mai coincidere, altrimenti vorrebbe dire che il salt
        // non viene davvero rigenerato.
        let hash1 = hash_password("supersegreta").unwrap();
        let hash2 = hash_password("supersegreta").unwrap();
        assert_ne!(hash1, hash2);
        assert!(verify_password(&hash1, "supersegreta"));
        assert!(verify_password(&hash2, "supersegreta"));
    }
}
