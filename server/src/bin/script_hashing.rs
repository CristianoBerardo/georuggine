use argon2::password_hash::{SaltString, rand_core::OsRng};
use argon2::{Argon2, PasswordHasher};

fn hash_password(password: &str) -> Result<String, argon2::password_hash::Error> {
    let salt = SaltString::generate(&mut OsRng);
    let argon2 = Argon2::default();
    let hash = argon2.hash_password(password.as_bytes(), &salt)?;
    Ok(hash.to_string())
}

fn main() {
    let users = vec![("mario", "supersegreta"), ("anna", "password")];

    for (username, password) in users {
        let hash = hash_password(password).expect("errore hashing");
        println!("username: {}, password hash: {}", username, hash);
    }
}
