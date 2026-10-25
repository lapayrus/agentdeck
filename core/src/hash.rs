use std::fs;
use std::path::Path;

use sha2::{Digest, Sha256};

pub fn hash_bytes(bytes: &[u8]) -> String {
    let mut hasher = Sha256::new();
    hasher.update(bytes);
    hex::encode(hasher.finalize())
}

pub fn hash_text(text: &str) -> String {
    hash_bytes(text.as_bytes())
}

pub fn hash_file(path: &Path) -> Option<String> {
    fs::read(path).ok().map(|bytes| hash_bytes(&bytes))
}
