use std::{collections::{HashSet}};

pub const DEFAULT_ALPHABET: &str = "ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";

pub fn validate_alphabet(alphabet: Option<String>, complementary_symbols: Option<String>) -> Result<String, String> {
    // Return error if both alphabet and complementary symbols are provided
    if alphabet.is_some() && complementary_symbols.is_some() {
        return Err("cannot provide both alphabet and complementary symbols".to_string());
    }

    // Determine the alphabet candidate characters sequence based on provided options:
    // - If alphabet is provided, use it as the candidate (complementary symbols are already included in the alphabet)
    // - Otherwise, check if complementary symbols are provided, and if so, replace the last two symbols of the default
    //   alphabet with the provided complementary symbols.
    // - If neither is provided, return an error indicating that at least one of them must be provided.
    let alphabet_candidate: Result<String, String> = if let Some(candidate) = alphabet {
        let alphabet_char_set: HashSet<char> = candidate.chars().collect();
        if alphabet_char_set.len() != 64 {
            Err(
                format!(
                    "alphabet must contain exactly 64 unique symbols, but has {}.",
                    alphabet_char_set.len()
                )
            )
        } else {
            Ok(candidate)
        }
    } else if let Some(complementary_symbols_candidate) = complementary_symbols {
        let complementary_symbols_char_set: HashSet<char> = complementary_symbols_candidate.chars().collect();
        if complementary_symbols_char_set.len() != 2 {
            Err(
                format!(
                    "complementary symbols must contain exactly 2 unique symbols, but has {}.",
                    complementary_symbols_char_set.len()
                )
            )
        } else {
            // Replace the last two symbols of the default alphabet with the provided complementary symbols
            let candidate = DEFAULT_ALPHABET[..62].to_string() + &complementary_symbols_candidate;
            Ok(candidate)
        }
    } else {
        Err("either alphabet or complementary symbols must be provided".to_string())
    };

    match alphabet_candidate {
        Ok(alphabet) => {
            let non_printable_characters = alphabet
                .chars()
                .filter(|&c| !is_printable_character(c as u8))
                .collect::<Vec<char>>();

            if non_printable_characters.len() > 0 {
                return Err(format!("alphabet contains non-printable symbols: {:?}", non_printable_characters));
            }

            return Ok(alphabet);
        }
        Err(e) => return Err(e),
    }
}

pub fn validate_padding_symbol(padding_symbol: char, alphabet: &String) -> Result<char, String> {
    if !is_printable_character(padding_symbol as u8) {
        return Err(format!("padding symbol '{}' is a non-printable character", padding_symbol));
    }

    if alphabet.contains(padding_symbol) {
        return Err(format!("padding symbol '{}' is part of the alphabet", padding_symbol));
    }

    Ok(padding_symbol)
}

pub fn build_alphabet_mapping(alphabet: &String) -> [u8; 64] {
    // TODO: Read comment below:
    // Building a mapping between 24-bits input and 4 characters of the alphabet output
    // will result in an excessive memory consumption, however, will potentially show faster performance.
    // Now the simple 6-bit input to 1 character output mapping is used. Performance will be optimized
    // with SIMD instructions in the future, but for now, this is a simple and straightforward approach.
    let mut mapping = [0_u8; 64];
    for (i, c) in alphabet.chars().enumerate().take(64) {
        mapping[i] = c as u8;
    }
    mapping
}

fn is_printable_character(byte: u8) -> bool {
    // Check if the byte is a printable ASCII character (32-126)
    byte >= 32_u8 && byte <= 126_u8
}
