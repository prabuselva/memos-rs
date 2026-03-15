use bcrypt::{hash, verify, DEFAULT_COST};
use ring::rand::{SecureRandom, SystemRandom};
use std::time::{SystemTime, UNIX_EPOCH};

pub fn generate_secure_token() -> String {
    let mut bytes = [0u8; 32];
    let rng = SystemRandom::new();
    rng.fill(&mut bytes)
        .expect("Failed to generate random bytes");
    hex::encode(bytes)
}

pub fn hash_password(password: &str) -> Result<String, anyhow::Error> {
    let hashed = hash(password, DEFAULT_COST)?;
    Ok(hashed)
}

pub fn verify_password(password: &str, hashed: &str) -> bool {
    verify(password, hashed).unwrap_or(false)
}

pub fn generate_session_token() -> String {
    let mut bytes = [0u8; 48];
    let rng = SystemRandom::new();
    rng.fill(&mut bytes)
        .expect("Failed to generate session token");
    hex::encode(bytes)
}

pub fn get_current_timestamp() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("Time went backwards")
        .as_secs()
}

pub fn generate_password_reset_token() -> String {
    let mut bytes = [0u8; 32];
    let rng = SystemRandom::new();
    rng.fill(&mut bytes)
        .expect("Failed to generate reset token");
    hex::encode(bytes)
}
