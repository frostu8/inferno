//! Password hashing utilities.

use rand::{distributions::Alphanumeric, rngs::StdRng, Rng, SeedableRng};

use sha2::{Digest, Sha256};

use std::cell::RefCell;

use base16::encode_lower;

/// Default length for salt.
pub const SALT_LENGTH: usize = 32;

thread_local! {
    static CRYPTO_RNG: RefCell<StdRng> = RefCell::new(StdRng::from_entropy());
}

/// Generates a random sequence of characters.
///
/// Each character is sampled from the [`Alphanumeric`] distribution, so each
/// character carries almost 6-bits of entropy.
pub fn generate_salt(length: usize) -> String {
    CRYPTO_RNG.with(|rng| {
        let mut rng = rng.borrow_mut();
        generate_salt_with(length, &mut *rng)
    })
}

/// Generates a random sequence of characters with a given [`Rng`].
pub fn generate_salt_with<R>(length: usize, rng: &mut R) -> String
where
    R: Rng,
{
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

/// Hashes a string and returns a 64-character hash.
pub fn hash(password: &str) -> String {
    let mut hasher = Sha256::new();

    hasher.update(password);

    let result = hasher.finalize();

    encode_lower(&result)
}
