use std::time::{SystemTime, UNIX_EPOCH};
use rand::prelude::StdRng;
use rand::SeedableRng;

pub fn generate_password(length: usize, alphabet_chars: Vec<char>, seed: Option<u64>) -> String {
    // TODO: The current implementation is naive and does not provide a sufficient level of security.
    // TODO: In particular, it does not ensure that all classes of symbols are represented in the output.
    // TODO: This must be fixed.
    let seed = seed.unwrap_or_else(|| {
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map(|d| d.as_nanos() as u64)
            .unwrap_or(0)
    });
    let mut rng = StdRng::seed_from_u64(seed);

    (0..length)
        .map(|_| {
            use rand::RngExt;
            alphabet_chars[rng.random_range(0..alphabet_chars.len())]
        })
        .collect()
}