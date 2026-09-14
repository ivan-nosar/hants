use std::collections::{HashMap, HashSet};

const ENCODE_CHUNK_SIZE: usize = 3;
const DECODE_CHUNK_SIZE: usize = 4;
const SIX_BITS_MASK: u32 = 0x3f;
const BITS_PER_SYMBOL: u8 = 6;
const WORD_SIZE_IN_BITS: u8 = 32;

pub fn encode_with_alphabet(
    input_bytes: &[u8],
    alphabet_mapping: [u8; 64],
    padding_symbol: u8,
) -> String {
    let mut encoded_buffer: Vec<char> =
        Vec::with_capacity(calculate_encoded_length(input_bytes.len()));

    // Process "body" of input payload (sequence of full 3-bytes chunks)
    let (chunks, tail) = input_bytes.as_chunks::<ENCODE_CHUNK_SIZE>();

    for chunk in chunks {
        // We use Big Endian arrangement to ensure chunk[0] is the most significant byte,
        // while chunk[2] is the least significant byte of the 3-byte chunk. That way, reading
        // first 6 bits from the left corresponds to the most significant bits of chunk[0].
        let chunk_value = u32::from_be_bytes([chunk[0], chunk[1], chunk[2], 0]);

        // Get all 4 6-bits segments from the `chunk_value` and encode them using the `alphabet_mapping`
        encoded_buffer
            .push(alphabet_mapping[((chunk_value >> 26) & SIX_BITS_MASK) as usize] as char);
        encoded_buffer
            .push(alphabet_mapping[((chunk_value >> 20) & SIX_BITS_MASK) as usize] as char);
        encoded_buffer
            .push(alphabet_mapping[((chunk_value >> 14) & SIX_BITS_MASK) as usize] as char);
        encoded_buffer
            .push(alphabet_mapping[((chunk_value >> 8) & SIX_BITS_MASK) as usize] as char);
    }

    // Process tail of input payload (sequence of 1 or 2 remaining bytes)
    let padding_char = padding_symbol as char;
    if tail.len() == 1 {
        let tail_value = u16::from_be_bytes([tail[0], 0]);
        encoded_buffer
            .push(alphabet_mapping[((tail_value >> 10) & (SIX_BITS_MASK as u16)) as usize] as char);
        encoded_buffer
            .push(alphabet_mapping[((tail_value >> 4) & (SIX_BITS_MASK as u16)) as usize] as char);
        encoded_buffer.push(padding_char);
        encoded_buffer.push(padding_char);
    } else if tail.len() == 2 {
        let tail_value = u16::from_be_bytes([tail[0], tail[1]]);
        encoded_buffer
            .push(alphabet_mapping[((tail_value >> 10) & (SIX_BITS_MASK as u16)) as usize] as char);
        encoded_buffer
            .push(alphabet_mapping[((tail_value >> 4) & (SIX_BITS_MASK as u16)) as usize] as char);
        encoded_buffer
            .push(alphabet_mapping[((tail_value << 2) & (SIX_BITS_MASK as u16)) as usize] as char);
        encoded_buffer.push(padding_char);
    }

    encoded_buffer.into_iter().collect()
}

pub fn decode_with_alphabet(
    input_bytes: &[u8],
    alphabet_mapping: HashMap<u8, u8>,
    padding_symbol: u8,
) -> Result<Vec<u8>, String> {
    // TODO: Low performance. Optimize
    // Trim from start and end: leading and trailing whitespaces are acceptable
    let trimmed_input_bytes = input_bytes.trim_ascii();

    let mut decoded_buffer: Vec<u8> =
        Vec::with_capacity(calculate_decoded_length(input_bytes.len()));

    // Current implementation demands paddings to be set correctly, so no "tail" expected.
    let (chunks, tail) = trimmed_input_bytes.as_chunks::<DECODE_CHUNK_SIZE>();

    if tail.len() > 0 {
        return Err(
            format!(
                "input payload is malformed: unexpected tail bytes detected in the end: '{}'",
                String::from_utf8_lossy(tail)
            )
        )
    }

    let valid_base64_symbols = alphabet_mapping.keys().copied().collect::<HashSet<u8>>();

    // Process "body" of input payload (sequence chunks without padding symbols)
    for (index, chunk) in chunks[..chunks.len() - 1].iter().enumerate() {
        if let Some(invalid_char_index) = chunk.iter().position(|c| !valid_base64_symbols.contains(c)) {
            let char_position_in_payload = index * DECODE_CHUNK_SIZE + invalid_char_index + 1;

            return Err(
                format!(
                    "invalid symbol detected in input payload: '{}' (position: {})",
                    chunk[invalid_char_index] as char,
                    char_position_in_payload
                )
            )
        }

        let bit_group_1 = alphabet_mapping[&chunk[0]] as u32 & SIX_BITS_MASK;
        let bit_group_2 = alphabet_mapping[&chunk[1]] as u32 & SIX_BITS_MASK;
        let bit_group_3 = alphabet_mapping[&chunk[2]] as u32 & SIX_BITS_MASK;
        let bit_group_4 = alphabet_mapping[&chunk[3]] as u32 & SIX_BITS_MASK;

        let decoded_chunk_value = bit_group_1 << 26 | bit_group_2 << 20 | bit_group_3 << 14 | bit_group_4 << 8;

        pack_decoded_value(decoded_chunk_value, None, &mut decoded_buffer);
    }

    // Process last chunk (also known as tail). It can contain 0, 1, or 2 padding chars.
    // Calculate number of trailing padding symbols and adjust output buffer size based on that.
    let tail = chunks[chunks.len() - 1];
    let mut i = tail.len();
    while i > 0 {
        if tail[i - 1] != padding_symbol {
            break;
        }
        i -= 1;
    }
    let padding_symbols_count = DECODE_CHUNK_SIZE - (i);

    if padding_symbols_count > 2 {
        return Err("more than 2 padding symbols detected".to_string());
    }

    let tail_without_padding = &tail[..DECODE_CHUNK_SIZE - padding_symbols_count];
    let mut decoded_tail_value = 0_u32;
    for (index, symbol) in tail_without_padding.iter().enumerate() {
        if !valid_base64_symbols.contains(symbol) {
            let char_position_in_payload = (chunks.len() - 1) * DECODE_CHUNK_SIZE + index + 1;

            return Err(
                format!(
                    "invalid symbol detected in input payload: '{}' (position: {})",
                    *symbol as char,
                    char_position_in_payload
                )
            )
        }

        let bitwise_shift = WORD_SIZE_IN_BITS - (index as u8 + 1) * BITS_PER_SYMBOL;
        decoded_tail_value |= (alphabet_mapping[&symbol] as u32 & SIX_BITS_MASK) << bitwise_shift;
    }

    pack_decoded_value(decoded_tail_value, Some(ENCODE_CHUNK_SIZE - padding_symbols_count), &mut decoded_buffer);

    Ok(decoded_buffer)
}

pub fn calculate_encoded_length(decoded_length: usize) -> usize {
    let full_chunks_count = decoded_length / ENCODE_CHUNK_SIZE;
    let tail_length = decoded_length % ENCODE_CHUNK_SIZE;

    let mut encoded_length = full_chunks_count * DECODE_CHUNK_SIZE;
    if tail_length > 0 {
        encoded_length += DECODE_CHUNK_SIZE;
    }

    encoded_length
}

pub fn calculate_decoded_length(encoded_length: usize) -> usize {
    // This function gives a best-effort assumption: precise value
    // may be 1 or 2 bytes less because to padding symbols.
    (encoded_length / DECODE_CHUNK_SIZE) * ENCODE_CHUNK_SIZE
}

fn pack_decoded_value(decoded_value: u32, meaningful_bytes_count: Option<usize>, decoded_buffer: &mut Vec<u8>) {
    // We use Big Endian arrangement to ensure chunk[0] is the most significant byte,
    // while chunk[2] is the least significant byte of the 3-byte chunk. That way, reading
    // first 6 bits from the left corresponds to the most significant bits of chunk[0].
    let decoded_tail_bytes = u32::to_be_bytes(decoded_value);

    // TODO: Suboptimal. Unfold loop.
    // Usually only 3 leading bytes are required: last byte is empty. The only exception is tail:
    // based on number of padding symbols there might be 1, 2, or 3 meaningful bytes.
    let bytes_to_put_count = meaningful_bytes_count.unwrap_or(ENCODE_CHUNK_SIZE);
    for i in 0..bytes_to_put_count {
        decoded_buffer.push(decoded_tail_bytes[i]);
    }
}

#[cfg(test)]
mod tests {
    use super::{calculate_encoded_length, encode_with_alphabet};
    // Leading `::` disambiguates the external crate from this crate's own `base64` module.
    use ::base64::Engine as _;
    use ::base64::alphabet::{Alphabet, Symbol};
    use ::base64::engine::general_purpose::{self, GeneralPurpose};

    const STANDARD_ALPHABET: &str =
        "ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";

    /// RFC 4648 section 5 "URL and Filename safe" alphabet.
    const URL_SAFE_ALPHABET: &str =
        "ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789-_";

    /// Permutation of the standard alphabet that places digits first.
    const DIGITS_FIRST_ALPHABET: &str =
        "0123456789ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz+/";

    fn mapping_of(alphabet: &str) -> [u8; 64] {
        let bytes = alphabet.as_bytes();
        assert_eq!(bytes.len(), 64, "test alphabet must be 64 ASCII bytes");

        let mut mapping = [0_u8; 64];
        mapping.copy_from_slice(bytes);
        mapping
    }

    fn encode(input: &[u8], alphabet: &str, padding_symbol: u8) -> String {
        encode_with_alphabet(input, mapping_of(alphabet), padding_symbol)
    }

    fn reference_engine(alphabet: &str, padding: u8) -> GeneralPurpose {
        let padding_symbol = Symbol::new(padding).unwrap();
        let alphabet =
            Alphabet::new_with_padding(alphabet, padding_symbol).expect("test alphabet must be a valid base64 alphabet");
        GeneralPurpose::new(&alphabet, general_purpose::PAD)
    }

    /// RFC 4648 encoder from the `base64` crate, used as a cross-check oracle.
    fn reference_encode(input: &[u8], alphabet: &str, padding: u8) -> String {
        reference_engine(alphabet, padding).encode(input)
    }

    /// Packs 6-bit values back into octets; requires a multiple of 4 values.
    fn pack_six_bit_values(values: &[u8]) -> Vec<u8> {
        assert_eq!(values.len() % 4, 0);

        let symbols: String = values
            .iter()
            .map(|&value| STANDARD_ALPHABET.as_bytes()[value as usize] as char)
            .collect();

        reference_engine(STANDARD_ALPHABET, b'=')
            .decode(symbols)
            .expect("six-bit values must form a canonical base64 payload")
    }

    struct XorShift64(u64);

    impl XorShift64 {
        fn bytes(&mut self, length: usize) -> Vec<u8> {
            (0..length)
                .map(|_| {
                    self.0 ^= self.0 << 13;
                    self.0 ^= self.0 >> 7;
                    self.0 ^= self.0 << 17;
                    (self.0 >> 24) as u8
                })
                .collect()
        }
    }

    #[test]
    fn matches_rfc4648_section10_test_vectors() {
        let vectors = [
            ("", ""),
            ("f", "Zg=="),
            ("fo", "Zm8="),
            ("foo", "Zm9v"),
            ("foob", "Zm9vYg=="),
            ("fooba", "Zm9vYmE="),
            ("foobar", "Zm9vYmFy"),
        ];

        for (input, expected) in vectors {
            assert_eq!(
                encode(input.as_bytes(), STANDARD_ALPHABET, b'='),
                expected,
                "RFC 4648 test vector for {input:?}"
            );
        }
    }

    #[test]
    fn matches_rfc4648_section9_binary_illustrations() {
        let vectors: [(&[u8], &str); 3] = [
            (&[0x14, 0xfb, 0x9c, 0x03, 0xd9, 0x7e], "FPucA9l+"),
            (&[0x14, 0xfb, 0x9c, 0x03, 0xd9], "FPucA9k="),
            (&[0x14, 0xfb, 0x9c, 0x03], "FPucAw=="),
        ];

        for (input, expected) in vectors {
            assert_eq!(
                encode(input, STANDARD_ALPHABET, b'='),
                expected,
                "RFC 4648 illustration for {input:02x?}"
            );
        }
    }

    #[test]
    fn encodes_empty_input_to_empty_output() {
        for alphabet in [STANDARD_ALPHABET, URL_SAFE_ALPHABET, DIGITS_FIRST_ALPHABET] {
            assert_eq!(encode(&[], alphabet, b'='), "");
        }
    }

    #[test]
    fn encodes_every_alphabet_index_in_order() {
        let indices: Vec<u8> = (0..64).collect();
        let input = pack_six_bit_values(&indices);

        for alphabet in [STANDARD_ALPHABET, URL_SAFE_ALPHABET, DIGITS_FIRST_ALPHABET] {
            assert_eq!(encode(&input, alphabet, b'='), alphabet);
        }
    }

    #[test]
    fn url_safe_alphabet_replaces_the_62nd_and_63rd_symbols() {
        // Every byte triple below maps exclusively onto alphabet indices 62 and 63.
        let vectors: [(&[u8], &str, &str); 3] = [
            (&[0xfb, 0xef, 0xbe], "++++", "----"),
            (&[0xff, 0xff, 0xff], "////", "____"),
            (&[0xfb, 0xff, 0xbf], "+/+/", "-_-_"),
        ];

        for (input, standard, url_safe) in vectors {
            assert_eq!(encode(input, STANDARD_ALPHABET, b'='), standard);
            assert_eq!(encode(input, URL_SAFE_ALPHABET, b'='), url_safe);
        }
    }

    #[test]
    fn url_safe_alphabet_keeps_alphanumeric_vectors_unchanged() {
        let vectors = [
            ("f", "Zg=="),
            ("fo", "Zm8="),
            ("foo", "Zm9v"),
            ("foobar", "Zm9vYmFy"),
        ];

        for (input, expected) in vectors {
            assert_eq!(encode(input.as_bytes(), URL_SAFE_ALPHABET, b'='), expected);
        }
    }

    #[test]
    fn reordered_alphabet_produces_permuted_output() {
        let vectors = [
            ("", ""),
            ("f", "PW=="),
            ("fo", "Pcy="),
            ("foo", "Pczl"),
            ("foobar", "PczlOc5o"),
        ];

        for (input, expected) in vectors {
            assert_eq!(
                encode(input.as_bytes(), DIGITS_FIRST_ALPHABET, b'='),
                expected,
                "digits-first alphabet for {input:?}"
            );
        }
    }

    #[test]
    fn honours_custom_padding_symbols() {
        for padding_symbol in *b"=.*~! %A" {
            let padding_char = padding_symbol as char;

            assert_eq!(
                encode(b"f", STANDARD_ALPHABET, padding_symbol),
                format!("Zg{padding_char}{padding_char}")
            );
            assert_eq!(
                encode(b"fo", STANDARD_ALPHABET, padding_symbol),
                format!("Zm8{padding_char}")
            );
            assert_eq!(encode(b"foo", STANDARD_ALPHABET, padding_symbol), "Zm9v");
        }
    }

    #[test]
    fn places_padding_according_to_the_input_remainder() {
        const PADDING: u8 = b'.';
        let padding_char = PADDING as char;
        let data: Vec<u8> = (0_u8..=32).collect();

        for length in 0..=data.len() {
            let encoded = encode(&data[..length], STANDARD_ALPHABET, PADDING);
            let expected_padding_count = match length % 3 {
                0 => 0,
                1 => 2,
                _ => 1,
            };
            let symbols: Vec<char> = encoded.chars().collect();

            assert_eq!(
                symbols.len() % 4,
                0,
                "length {length} must emit full quanta"
            );
            assert_eq!(
                symbols.iter().filter(|&&c| c == padding_char).count(),
                expected_padding_count,
                "unexpected padding count for length {length}"
            );
            assert!(
                symbols[symbols.len() - expected_padding_count..]
                    .iter()
                    .all(|&c| c == padding_char),
                "padding must form a suffix for length {length}"
            );
        }
    }

    #[test]
    fn zeroes_the_pad_bits_of_the_final_quantum() {
        // RFC 4648 section 3.5: unused bits of the last emitted symbol must be zero.
        let index_of = |symbol: char| STANDARD_ALPHABET.find(symbol).unwrap();

        for byte in 0_u8..=255 {
            let one_byte_tail = encode(&[byte], STANDARD_ALPHABET, b'=');
            let last_symbol = one_byte_tail.chars().nth(1).unwrap();
            assert_eq!(
                index_of(last_symbol) & 0b1111,
                0,
                "one-byte tail {byte:#04x} leaked pad bits"
            );

            let two_byte_tail = encode(&[0x00, byte], STANDARD_ALPHABET, b'=');
            let last_symbol = two_byte_tail.chars().nth(2).unwrap();
            assert_eq!(
                index_of(last_symbol) & 0b11,
                0,
                "two-byte tail {byte:#04x} leaked pad bits"
            );
        }
    }

    #[test]
    fn encodes_all_zero_and_all_one_byte_sequences() {
        assert_eq!(encode(&[0x00], STANDARD_ALPHABET, b'='), "AA==");
        assert_eq!(encode(&[0x00; 2], STANDARD_ALPHABET, b'='), "AAA=");
        assert_eq!(encode(&[0x00; 3], STANDARD_ALPHABET, b'='), "AAAA");
        assert_eq!(encode(&[0x00; 6], STANDARD_ALPHABET, b'='), "AAAAAAAA");
        assert_eq!(encode(&[0xff], STANDARD_ALPHABET, b'='), "/w==");
        assert_eq!(encode(&[0xff; 2], STANDARD_ALPHABET, b'='), "//8=");
        assert_eq!(encode(&[0xff; 3], STANDARD_ALPHABET, b'='), "////");
        assert_eq!(encode(&[0xff; 4], STANDARD_ALPHABET, b'='), "/////w==");
    }

    #[test]
    fn encodes_the_full_byte_range() {
        let input: Vec<u8> = (0_u8..=255).collect();
        let encoded = encode(&input, STANDARD_ALPHABET, b'=');

        assert!(encoded.starts_with("AAECAwQF"), "{encoded}");
        assert!(encoded.ends_with("/P3+/w=="), "{encoded}");
        assert_eq!(encoded, reference_encode(&input, STANDARD_ALPHABET, b'='));
    }

    #[test]
    fn encodes_sequences_that_are_not_valid_utf8() {
        let inputs: [&[u8]; 5] = [
            &[0x00, 0xff, 0x00],
            &[0xc3, 0x28],
            &[0xed, 0xa0, 0x80],
            &[0xfe, 0xff],
            &[0xf0, 0x9f, 0x92, 0xa9, 0x00, 0x01, 0x80],
        ];

        for input in inputs {
            assert_eq!(
                encode(input, STANDARD_ALPHABET, b'='),
                reference_encode(input, STANDARD_ALPHABET, b'='),
                "binary input {input:02x?}"
            );
        }
    }

    #[test]
    fn matches_reference_encoder_for_pseudorandom_binary_input() {
        let mut rng = XorShift64(0x2545_f491_4f6c_dd1d);

        for length in 0..=96 {
            let input = rng.bytes(length);

            for (alphabet, padding_symbol) in [
                (STANDARD_ALPHABET, b'='),
                (URL_SAFE_ALPHABET, b'.'),
                (DIGITS_FIRST_ALPHABET, b'*'),
            ] {
                assert_eq!(
                    encode(&input, alphabet, padding_symbol),
                    reference_encode(&input, alphabet, padding_symbol),
                    "length {length} with alphabet {alphabet}"
                );
            }
        }
    }

    #[test]
    fn output_length_matches_calculate_output_length() {
        let mut rng = XorShift64(0x9e37_79b9_7f4a_7c15);

        for length in 0..=64 {
            let input = rng.bytes(length);
            let encoded = encode(&input, STANDARD_ALPHABET, b'=');

            assert_eq!(
                encoded.chars().count(),
                calculate_encoded_length(length),
                "length {length}"
            );
        }
    }

    #[test]
    fn calculates_output_length_for_known_inputs() {
        let expectations = [
            (0, 0),
            (1, 4),
            (2, 4),
            (3, 4),
            (4, 8),
            (5, 8),
            (6, 8),
            (7, 12),
            (299, 400),
            (300, 400),
            (301, 404),
        ];

        for (input_length, expected) in expectations {
            assert_eq!(
                calculate_encoded_length(input_length),
                expected,
                "for input length {input_length}"
            );
        }
    }

    #[test]
    fn calculates_output_length_as_padded_quanta_count() {
        for input_length in 0..=1_000_usize {
            assert_eq!(
                calculate_encoded_length(input_length),
                input_length.div_ceil(3) * 4,
                "for input length {input_length}"
            );
        }
    }
}
