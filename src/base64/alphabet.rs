use std::collections::HashSet;

pub const DEFAULT_ALPHABET: &str =
    "ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";

pub fn validate_alphabet(
    alphabet: Option<String>,
    complementary_symbols: Option<String>,
) -> Result<String, String> {
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
        Ok(candidate)
    } else if let Some(complementary_symbols_candidate) = complementary_symbols {
        let complementary_symbols_char_set: HashSet<char> =
            complementary_symbols_candidate.chars().collect();
        if complementary_symbols_char_set.len() != 2 {
            Err(format!(
                "complementary symbols must contain exactly 2 unique symbols, but has {}.",
                complementary_symbols_char_set.len()
            ))
        } else {
            // Replace the last two symbols of the default alphabet with the provided complementary symbols
            let candidate = DEFAULT_ALPHABET[..62].to_string() + &complementary_symbols_candidate;
            Ok(candidate)
        }
    } else {
        Ok(DEFAULT_ALPHABET.to_string())
    };

    match alphabet_candidate {
        Ok(alphabet) => {
            // A 64-byte long alphabet cannot hold multi-byte Unicode symbols.
            let alphabet_bytes = alphabet.as_bytes();
            if alphabet_bytes.len() != 64 {
                return Err(format!(
                    "alphabet must contain exactly 64 single-byte symbols, but has {}: '{}'",
                    alphabet_bytes.len(),
                    alphabet
                ));
            }

            let alphabet_byte_set: HashSet<u8> = alphabet_bytes.iter().copied().collect();
            if alphabet_byte_set.len() != 64 {
                return Err(format!(
                    "alphabet must contain exactly 64 unique symbols, but has {}: '{}'",
                    alphabet_byte_set.len(),
                    alphabet
                ));
            }

            let non_printable_characters = alphabet
                .chars()
                .filter(|&c| !is_printable_character(c))
                .collect::<Vec<char>>();

            if !non_printable_characters.is_empty() {
                return Err(format!(
                    "alphabet contains non-printable symbols: {:?}",
                    non_printable_characters
                ));
            }

            Ok(alphabet)
        }
        Err(e) => Err(e),
    }
}

pub fn validate_padding_symbol(padding_symbol: char, alphabet: &str) -> Result<u8, String> {
    if !is_printable_character(padding_symbol) {
        return Err(format!(
            "padding symbol '{}' is a non-printable character",
            padding_symbol
        ));
    }

    if alphabet.contains(padding_symbol) {
        return Err(format!(
            "padding symbol '{}' is part of the alphabet",
            padding_symbol
        ));
    }

    Ok(padding_symbol as u8)
}

pub fn build_alphabet_mapping(alphabet: &str) -> [u8; 64] {
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

fn is_printable_character(character: char) -> bool {
    // `as u8` truncates any wider code point, so non-ASCII must be rejected up front.
    character.is_ascii() && (32_u8..=126_u8).contains(&(character as u8))
}

#[cfg(test)]
mod tests {
    use super::{
        DEFAULT_ALPHABET, build_alphabet_mapping, is_printable_character, validate_alphabet,
        validate_padding_symbol,
    };
    use std::collections::HashSet;

    /// RFC 4648 section 5 "URL and Filename safe" alphabet.
    const URL_SAFE_ALPHABET: &str =
        "ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789-_";

    /// Permutation of the standard alphabet that places digits first.
    const DIGITS_FIRST_ALPHABET: &str =
        "0123456789ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz+/";

    /// Alphanumeric prefix shared by the standard and URL-safe alphabets.
    fn alphanumeric_prefix() -> &'static str {
        &DEFAULT_ALPHABET[..62]
    }

    #[test]
    fn default_alphabet_matches_rfc4648_table1() {
        assert_eq!(
            DEFAULT_ALPHABET,
            "ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/"
        );
        assert_eq!(DEFAULT_ALPHABET.chars().count(), 64);
        assert_eq!(DEFAULT_ALPHABET.chars().collect::<HashSet<_>>().len(), 64);
    }

    #[test]
    fn returns_default_alphabet_when_no_option_is_provided() {
        assert_eq!(validate_alphabet(None, None).unwrap(), DEFAULT_ALPHABET);
    }

    #[test]
    fn returns_err_when_both_alphabet_and_complementary_symbols_are_provided() {
        let error = validate_alphabet(Some(DEFAULT_ALPHABET.to_string()), Some("-_".to_string()))
            .unwrap_err();

        assert!(error.contains("cannot provide both"), "{error}");
    }

    #[test]
    fn accepts_custom_alphabet_with_64_unique_symbols() {
        for alphabet in [DEFAULT_ALPHABET, URL_SAFE_ALPHABET, DIGITS_FIRST_ALPHABET] {
            assert_eq!(
                validate_alphabet(Some(alphabet.to_string()), None).unwrap(),
                alphabet
            );
        }
    }

    #[test]
    fn returns_err_when_custom_alphabet_byte_length_is_not_64() {
        let candidates = [
            String::new(),
            DEFAULT_ALPHABET[..63].to_string(),
            format!("{DEFAULT_ALPHABET}~"),
            // 64 characters, but 'Ł' occupies two bytes.
            format!("{}\u{141}", &DEFAULT_ALPHABET[..63]),
        ];

        for candidate in candidates {
            let error = validate_alphabet(Some(candidate.clone()), None).unwrap_err();

            assert!(
                error.contains("exactly 64 single-byte symbols"),
                "unexpected error for {candidate:?}: {error}"
            );
            assert!(
                error.contains(&candidate),
                "error must quote the alphabet: {error}"
            );
        }
    }

    #[test]
    fn returns_err_when_custom_alphabet_contains_duplicate_symbols() {
        let candidate = format!("A{}", &DEFAULT_ALPHABET[..63]);

        let error = validate_alphabet(Some(candidate.clone()), None).unwrap_err();

        assert!(
            error.contains("exactly 64 unique symbols, but has 63"),
            "{error}"
        );
        assert!(
            error.contains(&candidate),
            "error must quote the alphabet: {error}"
        );
    }

    #[test]
    fn returns_err_when_custom_alphabet_contains_non_ascii_symbols() {
        // 62 ASCII symbols plus 'Ł' (0xc5 0x81) form exactly 64 unique bytes,
        // so only the ASCII check is able to reject this alphabet.
        let candidate = format!("{}\u{141}", alphanumeric_prefix());
        assert_eq!(candidate.len(), 64, "precondition: 64 bytes");

        let error = validate_alphabet(Some(candidate), None).unwrap_err();

        assert!(error.contains("non-printable"), "{error}");
    }

    #[test]
    fn returns_err_when_custom_alphabet_contains_non_printable_symbols() {
        for symbol in ['\u{0}', '\t', '\n', '\r', '\u{1f}', '\u{7f}'] {
            let candidate = format!("{}{symbol}", &DEFAULT_ALPHABET[..63]);
            let error = validate_alphabet(Some(candidate), None).unwrap_err();

            assert!(error.contains("non-printable"), "for {symbol:?}: {error}");
        }
    }

    #[test]
    fn accepts_printable_ascii_boundary_symbols_in_custom_alphabet() {
        // 0x20 (space) and 0x7e (tilde) are the inclusive bounds of the printable range.
        let candidate = format!("{} ~", alphanumeric_prefix());

        assert_eq!(
            validate_alphabet(Some(candidate.clone()), None).unwrap(),
            candidate
        );
    }

    #[test]
    fn replaces_complementary_symbols_of_the_default_alphabet() {
        assert_eq!(
            validate_alphabet(None, Some("-_".to_string())).unwrap(),
            URL_SAFE_ALPHABET
        );
        assert_eq!(
            validate_alphabet(None, Some("*~".to_string())).unwrap(),
            format!("{}*~", alphanumeric_prefix())
        );
    }

    #[test]
    fn returns_err_when_complementary_symbols_count_is_not_two() {
        for candidate in ["", "-", "--", "-_.", "-_.!"] {
            let error = validate_alphabet(None, Some(candidate.to_string())).unwrap_err();

            assert!(
                error.contains("exactly 2 unique symbols"),
                "unexpected error for {candidate:?}: {error}"
            );
        }
    }

    #[test]
    fn returns_err_when_complementary_symbols_are_non_printable() {
        for candidate in ["\n\t", "\u{0}\u{1}", "-\u{7f}"] {
            let error = validate_alphabet(None, Some(candidate.to_string())).unwrap_err();

            assert!(
                error.contains("non-printable"),
                "unexpected error for {candidate:?}: {error}"
            );
        }
    }

    #[test]
    fn returns_err_when_complementary_symbols_contain_duplicates_within_themselves() {
        // A repeated symbol collapses the unique count below two.
        for candidate in ["--", "__", "++"] {
            let error = validate_alphabet(None, Some(candidate.to_string())).unwrap_err();

            assert!(
                error.contains("exactly 2 unique symbols, but has 1"),
                "unexpected error for {candidate:?}: {error}"
            );
        }

        // Two unique symbols, but the repetition makes the resulting alphabet too long.
        for candidate in ["-_-", "--_"] {
            let error = validate_alphabet(None, Some(candidate.to_string())).unwrap_err();

            assert!(
                error.contains("exactly 64 single-byte symbols, but has 65"),
                "unexpected error for {candidate:?}: {error}"
            );
        }
    }

    #[test]
    fn returns_err_when_complementary_symbols_duplicate_alphabet_symbols() {
        for candidate in ["AB", "0z", "a9"] {
            let error = validate_alphabet(None, Some(candidate.to_string())).unwrap_err();

            assert!(
                error.contains("exactly 64 unique symbols, but has 62"),
                "unexpected error for {candidate:?}: {error}"
            );
            assert!(
                error.contains(&format!("{}{candidate}", alphanumeric_prefix())),
                "error must quote the resulting alphabet: {error}"
            );
        }
    }

    #[test]
    fn returns_err_when_complementary_symbols_are_non_ascii() {
        // Two unique characters, but four bytes on top of the 62-byte prefix.
        let error = validate_alphabet(None, Some("\u{141}\u{df}".to_string())).unwrap_err();

        assert!(error.contains("exactly 64 single-byte symbols"), "{error}");
    }

    #[test]
    fn accepts_padding_symbol_outside_the_alphabet() {
        for symbol in ['=', '.', '*', '~', '!', ' ', '%', '-', '_'] {
            assert_eq!(
                validate_padding_symbol(symbol, DEFAULT_ALPHABET).unwrap(),
                symbol as u8
            );
        }
    }

    #[test]
    fn accepts_padding_symbol_at_printable_ascii_boundaries() {
        assert_eq!(
            validate_padding_symbol(' ', DEFAULT_ALPHABET).unwrap(),
            b' '
        );
        assert_eq!(
            validate_padding_symbol('~', DEFAULT_ALPHABET).unwrap(),
            b'~'
        );
    }

    #[test]
    fn returns_err_when_padding_symbol_is_part_of_the_alphabet() {
        for symbol in ['A', 'Z', 'a', 'z', '0', '9', '+', '/'] {
            let error = validate_padding_symbol(symbol, DEFAULT_ALPHABET).unwrap_err();

            assert!(
                error.contains("part of the alphabet"),
                "for {symbol:?}: {error}"
            );
        }

        // The same symbol is accepted once the alphabet no longer contains it.
        assert_eq!(
            validate_padding_symbol('+', URL_SAFE_ALPHABET).unwrap(),
            b'+'
        );
    }

    #[test]
    fn returns_err_when_padding_symbol_is_non_printable() {
        for symbol in ['\u{0}', '\t', '\n', '\r', '\u{1f}', '\u{7f}'] {
            let error = validate_padding_symbol(symbol, DEFAULT_ALPHABET).unwrap_err();

            assert!(error.contains("non-printable"), "for {symbol:?}: {error}");
        }
    }

    #[test]
    fn returns_err_when_padding_symbol_is_non_ascii() {
        // 'Ł', 'Ľ' and 'ž' all truncate into printable ASCII under a lossy `as u8` cast.
        for symbol in ['\u{141}', '\u{13d}', '\u{17e}', '\u{1f4a9}'] {
            let error = validate_padding_symbol(symbol, DEFAULT_ALPHABET).unwrap_err();

            assert!(error.contains("non-printable"), "for {symbol:?}: {error}");
        }
    }

    #[test]
    fn builds_index_to_symbol_mapping() {
        for alphabet in [DEFAULT_ALPHABET, URL_SAFE_ALPHABET, DIGITS_FIRST_ALPHABET] {
            let mapping = build_alphabet_mapping(alphabet);

            assert_eq!(mapping, *alphabet.as_bytes(), "mapping for {alphabet}");
        }
    }

    #[test]
    fn maps_validated_complementary_symbols_to_the_last_two_indices() {
        let alphabet = validate_alphabet(None, Some("-_".to_string())).unwrap();
        let mapping = build_alphabet_mapping(&alphabet);

        assert_eq!(mapping[62], b'-');
        assert_eq!(mapping[63], b'_');
    }

    #[test]
    fn ignores_alphabet_symbols_beyond_the_first_64() {
        let oversized = format!("{DEFAULT_ALPHABET}~!@");

        assert_eq!(
            build_alphabet_mapping(&oversized),
            build_alphabet_mapping(DEFAULT_ALPHABET)
        );
    }

    #[test]
    fn treats_the_whole_printable_ascii_range_as_printable() {
        for byte in 32_u8..=126 {
            assert!(
                is_printable_character(byte as char),
                "{byte:#04x} must be printable"
            );
        }
    }

    #[test]
    fn treats_control_ascii_characters_as_non_printable() {
        for byte in 0_u8..=31 {
            assert!(
                !is_printable_character(byte as char),
                "{byte:#04x} must be non-printable"
            );
        }

        assert!(!is_printable_character('\u{7f}'), "DEL must be rejected");
    }

    #[test]
    fn treats_non_ascii_characters_as_non_printable() {
        // 0x141, 0x13d and 0x17e truncate back into printable ASCII under an `as u8` cast.
        for code_point in 0x80_u32..=0x2ff {
            let symbol = char::from_u32(code_point).unwrap();

            assert!(
                !is_printable_character(symbol),
                "{symbol:?} (U+{code_point:04X}) must be rejected"
            );
        }

        for symbol in ['\u{1f4a9}', '\u{10ffff}'] {
            assert!(
                !is_printable_character(symbol),
                "{symbol:?} must be rejected"
            );
        }
    }
}
