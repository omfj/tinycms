use rand::Rng;
use sha2::{Digest, Sha256};

pub fn hash_token(token: &str) -> String {
    let result = Sha256::digest(token.as_bytes());
    result.iter().map(|b| format!("{b:02x}")).collect()
}

pub fn generate_token() -> String {
    let mut bytes = [0u8; 32];
    rand::rng().fill_bytes(&mut bytes);
    bytes.iter().map(|b| format!("{b:02x}")).collect()
}
