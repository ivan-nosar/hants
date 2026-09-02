use rand::SeedableRng;
use rand::prelude::StdRng;
use rand::rngs::SysRng;

pub fn generate_password(length: usize, alphabet_chars: Vec<char>, seed: Option<u64>) -> String {
    // TODO: The current implementation is naive and does not provide a sufficient level of security.
    // TODO: In particular, it does not ensure that all classes of symbols are represented in the output.
    // TODO: This must be fixed. Suggested implementation is Chromium's `GenerateMaxEntropyPassword`:
    // TODO: https://github.com/chromium/chromium/blob/d4fb2e185f2e984d03200fd0b49086201ac71478/components/password_manager/core/browser/generation/password_generator.cc#L94
    // TODO: Add minimum one symbol of every class, fill the remaining space with random symbols
    // TODO: from all classes, then shuffle the resulting string to ensure high entropy.

    // If seed is provided - use `seed_from_u64` with this seed. If not - use `SysRng` generator that
    // retrieves cryptographically secure entropy from the host OS core - it's not bound directly to
    // any externally-visible data (such as timestamps) and ensure cryptographic resistance.
    let mut rng = match seed {
        Some(seed) => { StdRng::seed_from_u64(seed) }
        None => { StdRng::try_from_rng(&mut SysRng).unwrap() }
    };

    (0..length)
        .map(|_| {
            use rand::RngExt;
            alphabet_chars[rng.random_range(0..alphabet_chars.len())]
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::generate_password;
    use std::collections::HashSet;

    #[test]
    fn same_seed_produces_same_password() {
        let alphabet: Vec<char> = "abcdefghijklmnopqrstuvwxyz".chars().collect();
        let length: usize = 64;
        let seed = Some(12_345);

        let first = generate_password(length, alphabet.clone(), seed);
        let second = generate_password(length, alphabet, seed);

        assert_eq!(first, second);
    }

    #[test]
    fn produces_password_of_requested_length_using_only_alphabet_chars() {
        let alphabet: Vec<char> = "abcXYZ0189".chars().collect();
        let allowed: HashSet<char> = alphabet.iter().copied().collect();
        let length: usize = 128;
        let seed = Some(2_024);

        let password = generate_password(length, alphabet, seed);

        assert_eq!(password.chars().count(), length);
        for ch in password.chars() {
            assert!(
                allowed.contains(&ch),
                "unexpected char {ch:?} not in alphabet"
            );
        }
    }
}
