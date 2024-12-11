//! Password management utilities.

use rand::{distributions::Alphanumeric, rngs::StdRng, Rng, SeedableRng};

use sha2::{Digest, Sha256};

use base16::encode_lower;

/// Default length for salt.
pub const SALT_LENGTH: usize = 32;

/// Generates a random sequence of characters.
pub fn generate_salt(length: usize) -> String {
    let rng = StdRng::from_entropy();
    rng.sample_iter(Alphanumeric)
        .map(|u| u as char)
        .take(length)
        .collect()
}

/// Hashes a password with a given salt and returns a 64-character hash.
pub fn hash_password(password: &str, salt: &str) -> String {
    let mut hasher = Sha256::new();

    hasher.update(password);
    hasher.update(salt);

    let result = hasher.finalize();

    encode_lower(&result)
}
