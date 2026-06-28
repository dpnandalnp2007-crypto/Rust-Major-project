use rand::{thread_rng, RngCore};
use sha2::{Digest, Sha256};

pub fn generate_api_key() -> (String, String, String) {
    let mut bytes = [0u8; 32];
    thread_rng().fill_bytes(&mut bytes);
    
    let encoded = hex::encode(bytes);
    let raw_key = format!("iam_live_{}", encoded);
    
    let key_prefix = raw_key.chars().take(16).collect::<String>();
    let key_hash = hash_api_key(&raw_key);
    
    (raw_key, key_prefix, key_hash)
}

pub fn hash_api_key(raw_key: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(raw_key.as_bytes());
    hex::encode(hasher.finalize())
}
