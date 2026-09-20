use crate::base64::alphabet::MISSED_ALPHABET_SYMBOL;

const ENCODE_CHUNK_SIZE: usize = 3;
const DECODE_CHUNK_SIZE: usize = 4;
const SIX_BITS_MASK: u32 = 0x3f;

pub fn encode_with_alphabet(
    input_bytes: &[u8],
    alphabet_mapping: [u8; 64],
    padding_symbol: Option<u8>,
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
    if tail.len() == 1 {
        let tail_value = u16::from_be_bytes([tail[0], 0]);
        encoded_buffer
            .push(alphabet_mapping[((tail_value >> 10) & (SIX_BITS_MASK as u16)) as usize] as char);
        encoded_buffer
            .push(alphabet_mapping[((tail_value >> 4) & (SIX_BITS_MASK as u16)) as usize] as char);

        if let Some(defined_padding_symbol) = padding_symbol {
            let padding_char = defined_padding_symbol as char;
            encoded_buffer.push(padding_char);
            encoded_buffer.push(padding_char);
        }
    } else if tail.len() == 2 {
        let tail_value = u16::from_be_bytes([tail[0], tail[1]]);
        encoded_buffer
            .push(alphabet_mapping[((tail_value >> 10) & (SIX_BITS_MASK as u16)) as usize] as char);
        encoded_buffer
            .push(alphabet_mapping[((tail_value >> 4) & (SIX_BITS_MASK as u16)) as usize] as char);
        encoded_buffer
            .push(alphabet_mapping[((tail_value << 2) & (SIX_BITS_MASK as u16)) as usize] as char);

        if let Some(defined_padding_symbol) = padding_symbol {
            let padding_char = defined_padding_symbol as char;
            encoded_buffer.push(padding_char);
        }
    }

    encoded_buffer.into_iter().collect()
}

pub fn decode_with_alphabet(
    input_bytes: &[u8],
    alphabet_mapping: [u8; 256],
    padding_symbol: Option<u8>,
) -> Result<Vec<u8>, String> {
    const BITS_PER_SYMBOL: u8 = 6;
    const BITS_PER_BYTE: usize = 8;
    const WORD_SIZE_IN_BITS: u8 = 32;

    // Trim from start and end: leading and trailing whitespaces are acceptable
    let (trimmed_input_bytes, trimmed_from_start) = trim_whitespaces(input_bytes, padding_symbol);

    if trimmed_input_bytes.is_empty() {
        return Ok(Vec::new());
    }

    let mut decoded_buffer: Vec<u8> =
        Vec::with_capacity(calculate_decoded_length(trimmed_input_bytes.len()));

    // A final chunk shorter than 4 symbols is only acceptable when `--no-pad` is active: the
    // symbols that padding would have occupied are simply absent. Otherwise the input has to be
    // aligned to the decoding block size.
    if padding_symbol.is_some() && trimmed_input_bytes.len() % DECODE_CHUNK_SIZE != 0 {
        return Err(format!(
            "input payload is malformed: its length must be aligned to the decoding block \
            size of 4 bytes, but the actual length is {} bytes.",
            trimmed_input_bytes.len()
        ));
    }

    // When the input is aligned to the decoding block size, `as_chunks` leaves the tail empty and
    // the padding symbols end up inside the last chunk. Decoding the tail requires different logic
    // because of that padding, so the last chunk is always split off explicitly.
    let (mut chunks, mut tail) = trimmed_input_bytes.as_chunks::<DECODE_CHUNK_SIZE>();

    if tail.is_empty() {
        (chunks, tail) = (&chunks[..chunks.len() - 1], &chunks[chunks.len() - 1]);
    }

    // The tail is the most error-prone part of the payload, so it is processed ahead of the body:
    // a malformed tail fails fast, before any compute cycles are spent on decoding the body.

    // Process last chunk (also known as tail). It can contain 0, 1, or 2 padding chars.
    // Calculate number of trailing padding symbols and adjust output buffer size based on that.
    let padding_symbols_count = if let Some(defined_padding_symbol) = padding_symbol {
        let padding_symbols_count = tail
            .iter()
            .rev()
            .take_while(|&&symbol| symbol == defined_padding_symbol)
            .count();

        if padding_symbols_count > 2 {
            return Err(format!(
                "input payload is malformed: not more than 2 padding symbols is expected, \
                but {} found.",
                padding_symbols_count
            ));
        }

        tail = &tail[..DECODE_CHUNK_SIZE - padding_symbols_count];
        padding_symbols_count
    } else {
        0
    };

    // The meaningful part of the tail can never be shorter than 2 symbols: even a single source
    // byte needs 2 symbols to carry its 8 bits.
    if tail.len() < 2 {
        return Err(format!(
            "input payload is malformed: meaningful tail can't be shorter than 2 symbols, but found {}: '{}'.",
            tail.len(),
            &String::from_utf8_lossy(tail)
        ));
    }

    let mut decoded_tail_value = 0_u32;
    for (index, symbol) in tail.iter().enumerate() {
        if alphabet_mapping[*symbol as usize] == MISSED_ALPHABET_SYMBOL {
            let char_position_in_payload =
                trimmed_from_start + chunks.len() * DECODE_CHUNK_SIZE + index + 1;

            return Err(invalid_symbol_at_position_message(
                *symbol as char,
                char_position_in_payload,
            ));
        }

        let bitwise_shift = WORD_SIZE_IN_BITS - (index as u8 + 1) * BITS_PER_SYMBOL;
        decoded_tail_value |=
            (alphabet_mapping[*symbol as usize] as u32 & SIX_BITS_MASK) << bitwise_shift;
    }

    // Number of bytes in `decoded_tail_value` that carry actual decoded bits.
    let meaningful_bytes_count = if padding_symbol.is_some() {
        ENCODE_CHUNK_SIZE - padding_symbols_count
    } else {
        // Formally this is `floor((tail.len() * BITS_PER_SYMBOL) / BITS_PER_BYTE)`, but the tail is
        // never longer than 4 symbols, so a plain subtraction yields the same result and looks much simpler:
        // - 1 source byte -> 8 bits -> 2 encoded symbols (4 bits are padding zeroes)
        // - 2 source bytes -> 16 meaningful bits -> 3 encoded symbols (2 bits are padding zeroes)
        // - 3 source bytes -> 24 meaningful bits -> 4 encoded symbols (0 padding zeroes)
        tail.len() - 1
    };

    // RFC 4648 section 3.5: bits of the last symbol that carry no data must be zero.
    if decoded_tail_value & (u32::MAX >> (meaningful_bytes_count * BITS_PER_BYTE)) != 0 {
        let last_symbol_index = tail.len() - 1;
        let char_position_in_payload =
            trimmed_from_start + chunks.len() * DECODE_CHUNK_SIZE + last_symbol_index + 1;

        return Err(format!(
            "non-zero padding bits detected in input payload: '{}' (position: {})",
            tail[last_symbol_index] as char, char_position_in_payload
        ));
    }

    // Process "body" of input payload (sequence chunks without padding symbols)
    for (index, chunk) in chunks.iter().enumerate() {
        if let Some(invalid_char_index) = chunk
            .iter()
            .position(|c| alphabet_mapping[*c as usize] == MISSED_ALPHABET_SYMBOL)
        {
            let char_position_in_payload =
                trimmed_from_start + index * DECODE_CHUNK_SIZE + invalid_char_index + 1;

            return Err(invalid_symbol_at_position_message(
                chunk[invalid_char_index] as char,
                char_position_in_payload,
            ));
        }

        let bit_group_1 = alphabet_mapping[chunk[0] as usize] as u32 & SIX_BITS_MASK;
        let bit_group_2 = alphabet_mapping[chunk[1] as usize] as u32 & SIX_BITS_MASK;
        let bit_group_3 = alphabet_mapping[chunk[2] as usize] as u32 & SIX_BITS_MASK;
        let bit_group_4 = alphabet_mapping[chunk[3] as usize] as u32 & SIX_BITS_MASK;

        let decoded_chunk_value =
            bit_group_1 << 26 | bit_group_2 << 20 | bit_group_3 << 14 | bit_group_4 << 8;

        pack_decoded_value(decoded_chunk_value, None, &mut decoded_buffer);
    }

    // Pack the preliminary-calculated tail data to the output buffer
    pack_decoded_value(
        decoded_tail_value,
        Some(meaningful_bytes_count),
        &mut decoded_buffer,
    );

    Ok(decoded_buffer)
}

pub fn calculate_encoded_length(decoded_length: usize) -> usize {
    decoded_length.div_ceil(ENCODE_CHUNK_SIZE) * DECODE_CHUNK_SIZE
}

pub fn calculate_decoded_length(encoded_length: usize) -> usize {
    // This function gives a best-effort assumption that may overshoot the actual decoded
    // length by 1 or 2 bytes: the precise value may be 1 or 2 bytes less because of
    // padding symbols.
    let message_chunks_count = (encoded_length as f64 / DECODE_CHUNK_SIZE as f64).ceil() as usize;

    message_chunks_count * ENCODE_CHUNK_SIZE
}

fn trim_whitespaces(input_bytes: &[u8], padding_symbol: Option<u8>) -> (&[u8], usize) {
    const SPACE_CHAR_CODE: u8 = 0x20;
    const HORIZONTAL_TAB_CHAR_CODE: u8 = 0x9;
    const LINE_FEED_CHAR_CODE: u8 = 0xA;
    const FORM_FEED_CHAR_CODE: u8 = 0xC;
    const CARRIAGE_RETURN_CHAR_CODE: u8 = 0xD;

    // A whitespace padding symbol carries data, so it must survive trimming. When padding is
    // disabled there is nothing to preserve and every ASCII whitespace byte is trimmable.
    let is_trimmable = |symbol: u8| {
        padding_symbol != Some(symbol)
            && matches!(
                symbol,
                SPACE_CHAR_CODE
                    | HORIZONTAL_TAB_CHAR_CODE
                    | LINE_FEED_CHAR_CODE
                    | FORM_FEED_CHAR_CODE
                    | CARRIAGE_RETURN_CHAR_CODE
            )
    };

    let mut start_shift: usize = 0;

    while start_shift < input_bytes.len() && is_trimmable(input_bytes[start_shift]) {
        start_shift += 1;
    }

    if start_shift == input_bytes.len() {
        return (&[], start_shift);
    }

    let mut end_shift: usize = input_bytes.len() - 1;
    while end_shift > 0 && is_trimmable(input_bytes[end_shift]) {
        end_shift -= 1;
    }

    (&input_bytes[start_shift..=end_shift], start_shift)
}

fn pack_decoded_value(
    decoded_value: u32,
    meaningful_bytes_count: Option<usize>,
    decoded_buffer: &mut Vec<u8>,
) {
    // We use Big Endian arrangement to ensure chunk[0] is the most significant byte,
    // while chunk[2] is the least significant byte of the 3-byte chunk. That way, reading
    // first 6 bits from the left corresponds to the most significant bits of chunk[0].
    let decoded_tail_bytes = u32::to_be_bytes(decoded_value);

    // Usually only 3 leading bytes are required: last byte is empty. The only exception is tail:
    // based on number of padding symbols there might be 1, 2, or 3 meaningful bytes.
    let bytes_to_put_count = meaningful_bytes_count.unwrap_or(ENCODE_CHUNK_SIZE);
    decoded_buffer.extend_from_slice(&decoded_tail_bytes[..bytes_to_put_count]);
}

fn invalid_symbol_at_position_message(symbol: char, position: usize) -> String {
    format!(
        "input payload is malformed: invalid symbol '{}' (position: {})",
        symbol, position
    )
}

#[cfg(test)]
mod tests {
    use super::{
        calculate_decoded_length, calculate_encoded_length, decode_with_alphabet,
        encode_with_alphabet, pack_decoded_value, trim_whitespaces,
    };
    // Leading `::` disambiguates the external crate from this crate's own `base64` module.
    use crate::base64::alphabet::MISSED_ALPHABET_SYMBOL;
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

    fn encode(input: &[u8], alphabet: &str, padding_symbol: Option<u8>) -> String {
        encode_with_alphabet(input, mapping_of(alphabet), padding_symbol)
    }

    fn decoding_mapping_of(alphabet: &str) -> [u8; 256] {
        let bytes = alphabet.as_bytes();
        assert_eq!(bytes.len(), 64, "test alphabet must be 64 ASCII bytes");

        let mut mapping = [MISSED_ALPHABET_SYMBOL; 256];
        for (i, c) in alphabet.chars().enumerate().take(64) {
            mapping[c as usize] = i as u8;
        }
        mapping
    }

    fn decode(input: &[u8], alphabet: &str, padding_symbol: Option<u8>) -> Result<Vec<u8>, String> {
        decode_with_alphabet(input, decoding_mapping_of(alphabet), padding_symbol)
    }

    fn reference_engine(alphabet: &str, padding: u8) -> GeneralPurpose {
        let padding_symbol = Symbol::new(padding).unwrap();
        let alphabet = Alphabet::new_with_padding(alphabet, padding_symbol)
            .expect("test alphabet must be a valid base64 alphabet");
        GeneralPurpose::new(&alphabet, general_purpose::PAD)
    }

    /// RFC 4648 encoder from the `base64` crate, used as a cross-check oracle.
    fn reference_encode(input: &[u8], alphabet: &str, padding: u8) -> String {
        reference_engine(alphabet, padding).encode(input)
    }

    /// RFC 4648 section 3.2 permits omitting the padding; this drops it from a padded payload.
    fn reference_encode_without_padding(input: &[u8], alphabet: &str) -> String {
        reference_encode(input, alphabet, b'=')
            .trim_end_matches('=')
            .to_string()
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

    /// 68-symbol body holding a symbol outside the alphabet at position 1, followed by `tail`.
    fn payload_with_malformed_body(tail: &str) -> String {
        format!("!m9v{}{tail}", "Zm9v".repeat(16))
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
                encode(input.as_bytes(), STANDARD_ALPHABET, Some(b'=')),
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
                encode(input, STANDARD_ALPHABET, Some(b'=')),
                expected,
                "RFC 4648 illustration for {input:02x?}"
            );
        }
    }

    #[test]
    fn matches_rfc4648_section10_test_vectors_without_padding() {
        let vectors = [
            ("", ""),
            ("f", "Zg"),
            ("fo", "Zm8"),
            ("foo", "Zm9v"),
            ("foob", "Zm9vYg"),
            ("fooba", "Zm9vYmE"),
            ("foobar", "Zm9vYmFy"),
        ];

        for (input, expected) in vectors {
            assert_eq!(
                encode(input.as_bytes(), STANDARD_ALPHABET, None),
                expected,
                "RFC 4648 test vector for {input:?}"
            );
        }
    }

    #[test]
    fn matches_rfc4648_section9_binary_illustrations_without_padding() {
        let vectors: [(&[u8], &str); 3] = [
            (&[0x14, 0xfb, 0x9c, 0x03, 0xd9, 0x7e], "FPucA9l+"),
            (&[0x14, 0xfb, 0x9c, 0x03, 0xd9], "FPucA9k"),
            (&[0x14, 0xfb, 0x9c, 0x03], "FPucAw"),
        ];

        for (input, expected) in vectors {
            assert_eq!(
                encode(input, STANDARD_ALPHABET, None),
                expected,
                "RFC 4648 illustration for {input:02x?}"
            );
        }
    }

    #[test]
    fn encodes_empty_input_to_empty_output() {
        for alphabet in [STANDARD_ALPHABET, URL_SAFE_ALPHABET, DIGITS_FIRST_ALPHABET] {
            assert_eq!(encode(&[], alphabet, Some(b'=')), "");
        }
    }

    #[test]
    fn encodes_empty_input_to_empty_output_without_padding() {
        for alphabet in [STANDARD_ALPHABET, URL_SAFE_ALPHABET, DIGITS_FIRST_ALPHABET] {
            assert_eq!(encode(&[], alphabet, None), "");
        }
    }

    #[test]
    fn encodes_every_alphabet_index_in_order() {
        let indices: Vec<u8> = (0..64).collect();
        let input = pack_six_bit_values(&indices);

        for alphabet in [STANDARD_ALPHABET, URL_SAFE_ALPHABET, DIGITS_FIRST_ALPHABET] {
            assert_eq!(encode(&input, alphabet, Some(b'=')), alphabet);
        }
    }

    #[test]
    fn encodes_every_alphabet_index_in_order_without_padding() {
        // The input is a whole number of quanta, so disabling padding changes nothing.
        let indices: Vec<u8> = (0..64).collect();
        let input = pack_six_bit_values(&indices);

        for alphabet in [STANDARD_ALPHABET, URL_SAFE_ALPHABET, DIGITS_FIRST_ALPHABET] {
            assert_eq!(encode(&input, alphabet, None), alphabet);
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
            assert_eq!(encode(input, STANDARD_ALPHABET, Some(b'=')), standard);
            assert_eq!(encode(input, URL_SAFE_ALPHABET, Some(b'=')), url_safe);
        }
    }

    #[test]
    fn url_safe_alphabet_replaces_the_62nd_and_63rd_symbols_without_padding() {
        let vectors: [(&[u8], &str, &str); 3] = [
            (&[0xfb, 0xef, 0xbe], "++++", "----"),
            (&[0xff, 0xff, 0xff], "////", "____"),
            (&[0xfb, 0xff, 0xbf], "+/+/", "-_-_"),
        ];

        for (input, standard, url_safe) in vectors {
            assert_eq!(encode(input, STANDARD_ALPHABET, None), standard);
            assert_eq!(encode(input, URL_SAFE_ALPHABET, None), url_safe);
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
            assert_eq!(
                encode(input.as_bytes(), URL_SAFE_ALPHABET, Some(b'=')),
                expected
            );
        }
    }

    #[test]
    fn url_safe_alphabet_keeps_alphanumeric_vectors_unchanged_without_padding() {
        let vectors = [
            ("f", "Zg"),
            ("fo", "Zm8"),
            ("foo", "Zm9v"),
            ("foobar", "Zm9vYmFy"),
        ];

        for (input, expected) in vectors {
            assert_eq!(encode(input.as_bytes(), URL_SAFE_ALPHABET, None), expected);
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
                encode(input.as_bytes(), DIGITS_FIRST_ALPHABET, Some(b'=')),
                expected,
                "digits-first alphabet for {input:?}"
            );
        }
    }

    #[test]
    fn reordered_alphabet_produces_permuted_output_without_padding() {
        let vectors = [
            ("", ""),
            ("f", "PW"),
            ("fo", "Pcy"),
            ("foo", "Pczl"),
            ("foobar", "PczlOc5o"),
        ];

        for (input, expected) in vectors {
            assert_eq!(
                encode(input.as_bytes(), DIGITS_FIRST_ALPHABET, None),
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
                encode(b"f", STANDARD_ALPHABET, Some(padding_symbol)),
                format!("Zg{padding_char}{padding_char}")
            );
            assert_eq!(
                encode(b"fo", STANDARD_ALPHABET, Some(padding_symbol)),
                format!("Zm8{padding_char}")
            );
            assert_eq!(
                encode(b"foo", STANDARD_ALPHABET, Some(padding_symbol)),
                "Zm9v"
            );
        }
    }

    #[test]
    fn emits_no_padding_symbol_at_all_when_padding_is_disabled() {
        assert_eq!(encode(b"f", STANDARD_ALPHABET, None), "Zg");
        assert_eq!(encode(b"fo", STANDARD_ALPHABET, None), "Zm8");
        assert_eq!(encode(b"foo", STANDARD_ALPHABET, None), "Zm9v");

        // Whatever `--padding-symbol` would have produced is absent from the output.
        for padding_symbol in *b"=.*~! %A" {
            let padding_char = padding_symbol as char;

            for input in [b"f".as_slice(), b"fo", b"foo", b"foobar"] {
                let encoded = encode(input, STANDARD_ALPHABET, None);

                assert!(
                    !encoded.ends_with(padding_char),
                    "{encoded:?} must not end with {padding_char:?}"
                );
            }
        }
    }

    #[test]
    fn places_padding_according_to_the_input_remainder() {
        const PADDING: u8 = b'.';
        let padding_char = PADDING as char;
        let data: Vec<u8> = (0_u8..=32).collect();

        for length in 0..=data.len() {
            let encoded = encode(&data[..length], STANDARD_ALPHABET, Some(PADDING));
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
    fn emits_no_padding_for_any_input_remainder() {
        let data: Vec<u8> = (0_u8..=32).collect();

        for length in 0..=data.len() {
            let encoded = encode(&data[..length], STANDARD_ALPHABET, None);
            let padded = encode(&data[..length], STANDARD_ALPHABET, Some(b'='));

            // Every source byte still costs 8 bits, so the output holds ceil(8 * length / 6) symbols.
            assert_eq!(
                encoded.chars().count(),
                (length * 4).div_ceil(3),
                "unexpected unpadded length for input length {length}"
            );
            assert_eq!(
                encoded,
                padded.trim_end_matches('='),
                "unpadded output must equal the padded one without its suffix for length {length}"
            );
        }
    }

    #[test]
    fn zeroes_the_pad_bits_of_the_final_quantum() {
        // RFC 4648 section 3.5: unused bits of the last emitted symbol must be zero.
        let index_of = |symbol: char| STANDARD_ALPHABET.find(symbol).unwrap();

        for byte in 0_u8..=255 {
            let one_byte_tail = encode(&[byte], STANDARD_ALPHABET, Some(b'='));
            let last_symbol = one_byte_tail.chars().nth(1).unwrap();
            assert_eq!(
                index_of(last_symbol) & 0b1111,
                0,
                "one-byte tail {byte:#04x} leaked pad bits"
            );

            let two_byte_tail = encode(&[0x00, byte], STANDARD_ALPHABET, Some(b'='));
            let last_symbol = two_byte_tail.chars().nth(2).unwrap();
            assert_eq!(
                index_of(last_symbol) & 0b11,
                0,
                "two-byte tail {byte:#04x} leaked pad bits"
            );
        }
    }

    #[test]
    fn zeroes_the_pad_bits_of_the_final_quantum_without_padding() {
        // Dropping the padding must not change the data symbols themselves.
        let index_of = |symbol: char| STANDARD_ALPHABET.find(symbol).unwrap();

        for byte in 0_u8..=255 {
            let one_byte_tail = encode(&[byte], STANDARD_ALPHABET, None);
            assert_eq!(one_byte_tail.chars().count(), 2, "tail {byte:#04x}");
            assert_eq!(
                index_of(one_byte_tail.chars().nth(1).unwrap()) & 0b1111,
                0,
                "one-byte tail {byte:#04x} leaked pad bits"
            );

            let two_byte_tail = encode(&[0x00, byte], STANDARD_ALPHABET, None);
            assert_eq!(two_byte_tail.chars().count(), 3, "tail {byte:#04x}");
            assert_eq!(
                index_of(two_byte_tail.chars().nth(2).unwrap()) & 0b11,
                0,
                "two-byte tail {byte:#04x} leaked pad bits"
            );
        }
    }

    #[test]
    fn encodes_all_zero_and_all_one_byte_sequences() {
        assert_eq!(encode(&[0x00], STANDARD_ALPHABET, Some(b'=')), "AA==");
        assert_eq!(encode(&[0x00; 2], STANDARD_ALPHABET, Some(b'=')), "AAA=");
        assert_eq!(encode(&[0x00; 3], STANDARD_ALPHABET, Some(b'=')), "AAAA");
        assert_eq!(
            encode(&[0x00; 6], STANDARD_ALPHABET, Some(b'=')),
            "AAAAAAAA"
        );
        assert_eq!(encode(&[0xff], STANDARD_ALPHABET, Some(b'=')), "/w==");
        assert_eq!(encode(&[0xff; 2], STANDARD_ALPHABET, Some(b'=')), "//8=");
        assert_eq!(encode(&[0xff; 3], STANDARD_ALPHABET, Some(b'=')), "////");
        assert_eq!(
            encode(&[0xff; 4], STANDARD_ALPHABET, Some(b'=')),
            "/////w=="
        );
    }

    #[test]
    fn encodes_all_zero_and_all_one_byte_sequences_without_padding() {
        assert_eq!(encode(&[0x00], STANDARD_ALPHABET, None), "AA");
        assert_eq!(encode(&[0x00; 2], STANDARD_ALPHABET, None), "AAA");
        assert_eq!(encode(&[0x00; 3], STANDARD_ALPHABET, None), "AAAA");
        assert_eq!(encode(&[0x00; 6], STANDARD_ALPHABET, None), "AAAAAAAA");
        assert_eq!(encode(&[0xff], STANDARD_ALPHABET, None), "/w");
        assert_eq!(encode(&[0xff; 2], STANDARD_ALPHABET, None), "//8");
        assert_eq!(encode(&[0xff; 3], STANDARD_ALPHABET, None), "////");
        assert_eq!(encode(&[0xff; 4], STANDARD_ALPHABET, None), "/////w");
    }

    #[test]
    fn encodes_the_full_byte_range() {
        let input: Vec<u8> = (0_u8..=255).collect();
        let encoded = encode(&input, STANDARD_ALPHABET, Some(b'='));

        assert!(encoded.starts_with("AAECAwQF"), "{encoded}");
        assert!(encoded.ends_with("/P3+/w=="), "{encoded}");
        assert_eq!(encoded, reference_encode(&input, STANDARD_ALPHABET, b'='));
    }

    #[test]
    fn encodes_the_full_byte_range_without_padding() {
        let input: Vec<u8> = (0_u8..=255).collect();
        let encoded = encode(&input, STANDARD_ALPHABET, None);

        assert!(encoded.starts_with("AAECAwQF"), "{encoded}");
        assert!(encoded.ends_with("/P3+/w"), "{encoded}");
        assert_eq!(
            encoded,
            reference_encode_without_padding(&input, STANDARD_ALPHABET)
        );
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
                encode(input, STANDARD_ALPHABET, Some(b'=')),
                reference_encode(input, STANDARD_ALPHABET, b'='),
                "binary input {input:02x?}"
            );
        }
    }

    #[test]
    fn encodes_sequences_that_are_not_valid_utf8_without_padding() {
        let inputs: [&[u8]; 5] = [
            &[0x00, 0xff, 0x00],
            &[0xc3, 0x28],
            &[0xed, 0xa0, 0x80],
            &[0xfe, 0xff],
            &[0xf0, 0x9f, 0x92, 0xa9, 0x00, 0x01, 0x80],
        ];

        for input in inputs {
            assert_eq!(
                encode(input, STANDARD_ALPHABET, None),
                reference_encode_without_padding(input, STANDARD_ALPHABET),
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
                    encode(&input, alphabet, Some(padding_symbol)),
                    reference_encode(&input, alphabet, padding_symbol),
                    "length {length} with alphabet {alphabet}"
                );
            }
        }
    }

    #[test]
    fn matches_reference_encoder_for_pseudorandom_binary_input_without_padding() {
        let mut rng = XorShift64(0x2545_f491_4f6c_dd1d);

        for length in 0..=96 {
            let input = rng.bytes(length);

            for alphabet in [STANDARD_ALPHABET, URL_SAFE_ALPHABET, DIGITS_FIRST_ALPHABET] {
                assert_eq!(
                    encode(&input, alphabet, None),
                    reference_encode_without_padding(&input, alphabet),
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
            let encoded = encode(&input, STANDARD_ALPHABET, Some(b'='));

            assert_eq!(
                encoded.chars().count(),
                calculate_encoded_length(length),
                "length {length}"
            );
        }
    }

    #[test]
    fn calculated_output_length_is_an_upper_bound_without_padding() {
        // `calculate_encoded_length` always counts whole quanta, so it overshoots
        // an unpadded payload by exactly the number of padding symbols it would carry.
        let mut rng = XorShift64(0x9e37_79b9_7f4a_7c15);

        for length in 0..=64 {
            let input = rng.bytes(length);
            let encoded = encode(&input, STANDARD_ALPHABET, None);
            let expected_padding_count = match length % 3 {
                0 => 0,
                1 => 2,
                _ => 1,
            };

            assert_eq!(
                encoded.chars().count() + expected_padding_count,
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

    #[test]
    fn decodes_rfc4648_section10_test_vectors() {
        let vectors = [
            ("", ""),
            ("Zg==", "f"),
            ("Zm8=", "fo"),
            ("Zm9v", "foo"),
            ("Zm9vYg==", "foob"),
            ("Zm9vYmE=", "fooba"),
            ("Zm9vYmFy", "foobar"),
        ];

        for (input, expected) in vectors {
            assert_eq!(
                decode(input.as_bytes(), STANDARD_ALPHABET, Some(b'=')).unwrap(),
                expected.as_bytes(),
                "RFC 4648 test vector for {input:?}"
            );
        }
    }

    #[test]
    fn decodes_rfc4648_section10_test_vectors_without_padding() {
        let vectors = [
            ("", ""),
            ("Zg", "f"),
            ("Zm8", "fo"),
            ("Zm9v", "foo"),
            ("Zm9vYg", "foob"),
            ("Zm9vYmE", "fooba"),
            ("Zm9vYmFy", "foobar"),
        ];

        for (input, expected) in vectors {
            assert_eq!(
                decode(input.as_bytes(), STANDARD_ALPHABET, None).unwrap(),
                expected.as_bytes(),
                "RFC 4648 test vector for {input:?}"
            );
        }
    }

    #[test]
    fn decodes_rfc4648_section9_binary_illustrations() {
        let vectors: [(&str, &[u8]); 3] = [
            ("FPucA9l+", &[0x14, 0xfb, 0x9c, 0x03, 0xd9, 0x7e]),
            ("FPucA9k=", &[0x14, 0xfb, 0x9c, 0x03, 0xd9]),
            ("FPucAw==", &[0x14, 0xfb, 0x9c, 0x03]),
        ];

        for (input, expected) in vectors {
            assert_eq!(
                decode(input.as_bytes(), STANDARD_ALPHABET, Some(b'=')).unwrap(),
                expected,
                "RFC 4648 illustration for {input:?}"
            );
        }
    }

    #[test]
    fn decodes_rfc4648_section9_binary_illustrations_without_padding() {
        let vectors: [(&str, &[u8]); 3] = [
            ("FPucA9l+", &[0x14, 0xfb, 0x9c, 0x03, 0xd9, 0x7e]),
            ("FPucA9k", &[0x14, 0xfb, 0x9c, 0x03, 0xd9]),
            ("FPucAw", &[0x14, 0xfb, 0x9c, 0x03]),
        ];

        for (input, expected) in vectors {
            assert_eq!(
                decode(input.as_bytes(), STANDARD_ALPHABET, None).unwrap(),
                expected,
                "RFC 4648 illustration for {input:?}"
            );
        }
    }

    #[test]
    fn decodes_every_alphabet_index_in_order() {
        let indices: Vec<u8> = (0..64).collect();
        let expected = pack_six_bit_values(&indices);

        for alphabet in [STANDARD_ALPHABET, URL_SAFE_ALPHABET, DIGITS_FIRST_ALPHABET] {
            assert_eq!(
                decode(alphabet.as_bytes(), alphabet, Some(b'=')).unwrap(),
                expected,
                "alphabet {alphabet}"
            );
        }
    }

    #[test]
    fn decodes_every_alphabet_index_in_order_without_padding() {
        let indices: Vec<u8> = (0..64).collect();
        let expected = pack_six_bit_values(&indices);

        for alphabet in [STANDARD_ALPHABET, URL_SAFE_ALPHABET, DIGITS_FIRST_ALPHABET] {
            assert_eq!(
                decode(alphabet.as_bytes(), alphabet, None).unwrap(),
                expected,
                "alphabet {alphabet}"
            );
        }
    }

    #[test]
    fn decodes_the_url_safe_complementary_symbols() {
        // Every payload below maps exclusively onto alphabet indices 62 and 63.
        let vectors: [(&str, &str, &[u8]); 3] = [
            ("++++", "----", &[0xfb, 0xef, 0xbe]),
            ("////", "____", &[0xff, 0xff, 0xff]),
            ("+/+/", "-_-_", &[0xfb, 0xff, 0xbf]),
        ];

        for (standard, url_safe, expected) in vectors {
            assert_eq!(
                decode(standard.as_bytes(), STANDARD_ALPHABET, Some(b'=')).unwrap(),
                expected
            );
            assert_eq!(
                decode(url_safe.as_bytes(), URL_SAFE_ALPHABET, Some(b'=')).unwrap(),
                expected
            );
        }
    }

    #[test]
    fn decodes_the_url_safe_complementary_symbols_without_padding() {
        let vectors: [(&str, &str, &[u8]); 3] = [
            ("++++", "----", &[0xfb, 0xef, 0xbe]),
            ("////", "____", &[0xff, 0xff, 0xff]),
            ("+/+/", "-_-_", &[0xfb, 0xff, 0xbf]),
        ];

        for (standard, url_safe, expected) in vectors {
            assert_eq!(
                decode(standard.as_bytes(), STANDARD_ALPHABET, None).unwrap(),
                expected
            );
            assert_eq!(
                decode(url_safe.as_bytes(), URL_SAFE_ALPHABET, None).unwrap(),
                expected
            );
        }
    }

    #[test]
    fn decodes_reordered_alphabet_payloads() {
        let vectors = [
            ("PW==", "f"),
            ("Pcy=", "fo"),
            ("Pczl", "foo"),
            ("PczlOc5o", "foobar"),
        ];

        for (input, expected) in vectors {
            assert_eq!(
                decode(input.as_bytes(), DIGITS_FIRST_ALPHABET, Some(b'=')).unwrap(),
                expected.as_bytes(),
                "digits-first alphabet for {input:?}"
            );
        }
    }

    #[test]
    fn decodes_reordered_alphabet_payloads_without_padding() {
        let vectors = [
            ("PW", "f"),
            ("Pcy", "fo"),
            ("Pczl", "foo"),
            ("PczlOc5o", "foobar"),
        ];

        for (input, expected) in vectors {
            assert_eq!(
                decode(input.as_bytes(), DIGITS_FIRST_ALPHABET, None).unwrap(),
                expected.as_bytes(),
                "digits-first alphabet for {input:?}"
            );
        }
    }

    #[test]
    fn decodes_with_custom_padding_symbols() {
        for padding_symbol in *b"=.*~!%" {
            let padding_char = padding_symbol as char;

            assert_eq!(
                decode(
                    format!("Zg{padding_char}{padding_char}").as_bytes(),
                    STANDARD_ALPHABET,
                    Some(padding_symbol)
                )
                .unwrap(),
                b"f",
                "padding {padding_char:?}"
            );
            assert_eq!(
                decode(
                    format!("Zm8{padding_char}").as_bytes(),
                    STANDARD_ALPHABET,
                    Some(padding_symbol)
                )
                .unwrap(),
                b"fo",
                "padding {padding_char:?}"
            );
            assert_eq!(
                decode(b"Zm9v", STANDARD_ALPHABET, Some(padding_symbol)).unwrap(),
                b"foo",
                "padding {padding_char:?}"
            );
        }
    }

    #[test]
    fn ignores_leading_and_trailing_ascii_whitespace() {
        for input in [
            " Zm9vYmFy",
            "Zm9vYmFy ",
            "\tZm9vYmFy\r\n",
            "\n\n  Zm9vYmFy  \n\n",
        ] {
            assert_eq!(
                decode(input.as_bytes(), STANDARD_ALPHABET, Some(b'=')).unwrap(),
                b"foobar",
                "for {input:?}"
            );
        }
    }

    #[test]
    fn ignores_leading_and_trailing_ascii_whitespace_without_padding() {
        // Without a padding symbol every ASCII whitespace byte is trimmable, including spaces.
        for input in [
            " Zm9vYmE",
            "Zm9vYmE ",
            "\tZm9vYmE\r\n",
            "\n\n  Zm9vYmE  \n\n",
        ] {
            assert_eq!(
                decode(input.as_bytes(), STANDARD_ALPHABET, None).unwrap(),
                b"fooba",
                "for {input:?}"
            );
        }
    }

    #[test]
    fn decodes_all_zero_and_all_one_byte_sequences() {
        let vectors: [(&str, &[u8]); 8] = [
            ("AA==", &[0x00]),
            ("AAA=", &[0x00; 2]),
            ("AAAA", &[0x00; 3]),
            ("AAAAAAAA", &[0x00; 6]),
            ("/w==", &[0xff]),
            ("//8=", &[0xff; 2]),
            ("////", &[0xff; 3]),
            ("/////w==", &[0xff; 4]),
        ];

        for (input, expected) in vectors {
            assert_eq!(
                decode(input.as_bytes(), STANDARD_ALPHABET, Some(b'=')).unwrap(),
                expected,
                "for {input:?}"
            );
        }
    }

    #[test]
    fn decodes_all_zero_and_all_one_byte_sequences_without_padding() {
        let vectors: [(&str, &[u8]); 8] = [
            ("AA", &[0x00]),
            ("AAA", &[0x00; 2]),
            ("AAAA", &[0x00; 3]),
            ("AAAAAAAA", &[0x00; 6]),
            ("/w", &[0xff]),
            ("//8", &[0xff; 2]),
            ("////", &[0xff; 3]),
            ("/////w", &[0xff; 4]),
        ];

        for (input, expected) in vectors {
            assert_eq!(
                decode(input.as_bytes(), STANDARD_ALPHABET, None).unwrap(),
                expected,
                "for {input:?}"
            );
        }
    }

    #[test]
    fn decodes_the_full_byte_range() {
        let expected: Vec<u8> = (0_u8..=255).collect();
        let encoded = reference_encode(&expected, STANDARD_ALPHABET, b'=');

        assert_eq!(
            decode(encoded.as_bytes(), STANDARD_ALPHABET, Some(b'=')).unwrap(),
            expected
        );
    }

    #[test]
    fn decodes_payloads_that_are_not_valid_utf8() {
        let expectations: [&[u8]; 5] = [
            &[0x00, 0xff, 0x00],
            &[0xc3, 0x28],
            &[0xed, 0xa0, 0x80],
            &[0xfe, 0xff],
            &[0xf0, 0x9f, 0x92, 0xa9, 0x00, 0x01, 0x80],
        ];

        for expected in expectations {
            let encoded = reference_encode(expected, STANDARD_ALPHABET, b'=');

            assert_eq!(
                decode(encoded.as_bytes(), STANDARD_ALPHABET, Some(b'=')).unwrap(),
                expected,
                "binary payload {expected:02x?}"
            );
        }
    }

    #[test]
    fn decodes_payloads_that_are_not_valid_utf8_without_padding() {
        let expectations: [&[u8]; 5] = [
            &[0x00, 0xff, 0x00],
            &[0xc3, 0x28],
            &[0xed, 0xa0, 0x80],
            &[0xfe, 0xff],
            &[0xf0, 0x9f, 0x92, 0xa9, 0x00, 0x01, 0x80],
        ];

        for expected in expectations {
            let encoded = reference_encode_without_padding(expected, STANDARD_ALPHABET);

            assert_eq!(
                decode(encoded.as_bytes(), STANDARD_ALPHABET, None).unwrap(),
                expected,
                "binary payload {expected:02x?}"
            );
        }
    }

    #[test]
    fn round_trips_pseudorandom_binary_input() {
        let mut rng = XorShift64(0x1234_5678_9abc_def0);

        for length in 0..=96 {
            let input = rng.bytes(length);

            for (alphabet, padding_symbol) in [
                (STANDARD_ALPHABET, b'='),
                (URL_SAFE_ALPHABET, b'.'),
                (DIGITS_FIRST_ALPHABET, b'*'),
            ] {
                let encoded = encode(&input, alphabet, Some(padding_symbol));

                assert_eq!(
                    decode(encoded.as_bytes(), alphabet, Some(padding_symbol)).unwrap(),
                    input,
                    "length {length} with alphabet {alphabet}"
                );
            }
        }
    }

    #[test]
    fn round_trips_pseudorandom_binary_input_without_padding() {
        let mut rng = XorShift64(0x1234_5678_9abc_def0);

        for length in 0..=96 {
            let input = rng.bytes(length);

            for alphabet in [STANDARD_ALPHABET, URL_SAFE_ALPHABET, DIGITS_FIRST_ALPHABET] {
                let encoded = encode(&input, alphabet, None);

                assert_eq!(
                    decode(encoded.as_bytes(), alphabet, None).unwrap(),
                    input,
                    "length {length} with alphabet {alphabet}"
                );
            }
        }
    }

    #[test]
    fn round_trips_every_input_length_in_both_padding_modes() {
        // `source == decode(encode(source))` must hold for every tail shape and every alphabet.
        let mut rng = XorShift64(0xf00d_face_b00c_1234);

        for length in 0..=48 {
            let input = rng.bytes(length);

            for (alphabet, padding_symbol) in [
                (STANDARD_ALPHABET, b'='),
                (URL_SAFE_ALPHABET, b'.'),
                (DIGITS_FIRST_ALPHABET, b'*'),
            ] {
                let padded = encode(&input, alphabet, Some(padding_symbol));
                let unpadded = encode(&input, alphabet, None);

                assert_eq!(
                    decode(padded.as_bytes(), alphabet, Some(padding_symbol)).unwrap(),
                    input,
                    "padded round trip of length {length} with alphabet {alphabet}"
                );
                assert_eq!(
                    decode(unpadded.as_bytes(), alphabet, None).unwrap(),
                    input,
                    "unpadded round trip of length {length} with alphabet {alphabet}"
                );
                assert_eq!(
                    unpadded,
                    padded.trim_end_matches(padding_symbol as char),
                    "both modes must emit the same data symbols for length {length}"
                );
            }
        }
    }

    #[test]
    fn matches_reference_decoder_for_pseudorandom_payloads() {
        let mut rng = XorShift64(0x0123_4567_89ab_cdef);

        for length in 0..=96 {
            let input = rng.bytes(length);

            for (alphabet, padding_symbol) in [
                (STANDARD_ALPHABET, b'='),
                (URL_SAFE_ALPHABET, b'.'),
                (DIGITS_FIRST_ALPHABET, b'*'),
            ] {
                let encoded = reference_encode(&input, alphabet, padding_symbol);
                let expected = reference_engine(alphabet, padding_symbol)
                    .decode(&encoded)
                    .expect("reference engine must decode its own output");

                assert_eq!(
                    decode(encoded.as_bytes(), alphabet, Some(padding_symbol)).unwrap(),
                    expected,
                    "length {length} with alphabet {alphabet}"
                );
            }
        }
    }

    #[test]
    fn matches_reference_decoder_for_pseudorandom_payloads_without_padding() {
        let mut rng = XorShift64(0x0123_4567_89ab_cdef);

        for length in 0..=96 {
            let input = rng.bytes(length);

            for alphabet in [STANDARD_ALPHABET, URL_SAFE_ALPHABET, DIGITS_FIRST_ALPHABET] {
                let encoded = reference_encode_without_padding(&input, alphabet);

                assert_eq!(
                    decode(encoded.as_bytes(), alphabet, None).unwrap(),
                    input,
                    "length {length} with alphabet {alphabet}"
                );
            }
        }
    }

    #[test]
    fn accepts_unaligned_input_length_when_padding_is_disabled() {
        // The alignment guard is skipped, so 2- and 3-symbol final quanta are valid input.
        let vectors: [(&str, &[u8]); 6] = [
            ("Zg", b"f"),
            ("Zm8", b"fo"),
            ("Zm9vYg", b"foob"),
            ("Zm9vYmE", b"fooba"),
            ("Zm9vYmFyZg", b"foobarf"),
            ("Zm9vYmFyZm8", b"foobarfo"),
        ];

        for (input, expected) in vectors {
            assert_eq!(
                decode(input.as_bytes(), STANDARD_ALPHABET, None).unwrap(),
                expected,
                "for {input:?}"
            );
        }
    }

    #[test]
    fn returns_err_when_input_length_is_not_a_multiple_of_four() {
        for input in ["Z", "Zm", "Zm9", "Zm9vY", "Zm9vYm", "Zm9vYmF", "Zm9vYmFy="] {
            let error = decode(input.as_bytes(), STANDARD_ALPHABET, Some(b'=')).unwrap_err();

            let input_length = input.len();

            assert!(
                error.contains(&format!(
                    "its length must be aligned to the decoding block size of 4 bytes, \
                     but the actual length is {input_length} bytes"
                )),
                "unexpected error for {input:?}: {error}"
            );
        }
    }

    #[test]
    fn returns_err_when_input_length_is_not_a_multiple_of_four_after_trimming() {
        let input = "  Zm9  ";
        let error = decode(input.as_bytes(), STANDARD_ALPHABET, Some(b'=')).unwrap_err();

        let trimmed_input_length = input.trim().len();

        assert!(
            error.contains(&format!(
                "its length must be aligned to the decoding block size of 4 bytes, \
                 but the actual length is {trimmed_input_length} bytes"
            )),
            "unexpected error for {input:?}: {error}"
        );
    }

    #[test]
    fn does_not_check_input_length_alignment_when_padding_is_disabled() {
        for input in ["Zg", "Zm8", "Zm9vYg", "Zm9vYmE"] {
            assert!(
                decode(input.as_bytes(), STANDARD_ALPHABET, None).is_ok(),
                "unaligned input {input:?} must be accepted"
            );
            assert!(
                decode(input.as_bytes(), STANDARD_ALPHABET, Some(b'=')).is_err(),
                "unaligned input {input:?} must be rejected when padding is required"
            );
        }
    }

    #[test]
    fn returns_err_when_the_final_quantum_holds_a_single_symbol_and_padding_is_disabled() {
        // A lone symbol carries 6 bits, which is not enough to reconstruct a source byte.
        for input in ["Z", "Zm9vY", "Zm9vYmFyZ"] {
            let error = decode(input.as_bytes(), STANDARD_ALPHABET, None).unwrap_err();

            assert!(
                error.contains("meaningful tail can't be shorter than 2 symbols, but found 1"),
                "unexpected error for {input:?}: {error}"
            );
        }
    }

    #[test]
    fn returns_err_when_the_final_quantum_holds_a_single_symbol_and_padding_is_enabled() {
        // The same shape stays aligned to the block size, so it is spelled with 3 padding symbols.
        // The padding counter guards this case before the tail length is ever inspected.
        for input in ["Z===", "Zm9vY===", "Zm9vYmFyZ==="] {
            let error = decode(input.as_bytes(), STANDARD_ALPHABET, Some(b'=')).unwrap_err();

            assert!(
                error.contains("not more than 2 padding symbols is expected, but 3 found"),
                "unexpected error for {input:?}: {error}"
            );
        }
    }

    #[test]
    fn reports_tail_errors_before_body_errors() {
        // The tail starts at offset 68, so a tail error always quotes position 69 or later.
        let expectations = [
            (
                "Z===",
                "not more than 2 padding symbols is expected, but 3 found",
            ),
            ("Z!==", "invalid symbol '!' (position: 70)"),
            (
                "Zh==",
                "non-zero padding bits detected in input payload: 'h' (position: 70)",
            ),
        ];

        for (tail, expected_error) in expectations {
            let input = payload_with_malformed_body(tail);
            let error = decode(input.as_bytes(), STANDARD_ALPHABET, Some(b'=')).unwrap_err();

            assert!(
                error.contains(expected_error),
                "unexpected error for tail {tail:?}: {error}"
            );
            assert!(
                !error.contains("(position: 1)"),
                "the body error must not win for tail {tail:?}: {error}"
            );
        }

        // Same body, well-formed tail: now the body error is the one that surfaces.
        let input = payload_with_malformed_body("Zg==");
        let error = decode(input.as_bytes(), STANDARD_ALPHABET, Some(b'=')).unwrap_err();

        assert!(
            error.contains("invalid symbol '!' (position: 1)"),
            "{error}"
        );
    }

    #[test]
    fn reports_tail_errors_before_body_errors_when_padding_is_disabled() {
        let expectations = [
            (
                "Z",
                "meaningful tail can't be shorter than 2 symbols, but found 1",
            ),
            ("Z!", "invalid symbol '!' (position: 70)"),
            (
                "Zh",
                "non-zero padding bits detected in input payload: 'h' (position: 70)",
            ),
            // Padding symbols are foreign to the alphabet once `--no-pad` is active.
            ("Zg==", "invalid symbol '=' (position: 71)"),
        ];

        for (tail, expected_error) in expectations {
            let input = payload_with_malformed_body(tail);
            let error = decode(input.as_bytes(), STANDARD_ALPHABET, None).unwrap_err();

            assert!(
                error.contains(expected_error),
                "unexpected error for tail {tail:?}: {error}"
            );
            assert!(
                !error.contains("(position: 1)"),
                "the body error must not win for tail {tail:?}: {error}"
            );
        }

        let input = payload_with_malformed_body("Zg");
        let error = decode(input.as_bytes(), STANDARD_ALPHABET, None).unwrap_err();

        assert!(
            error.contains("invalid symbol '!' (position: 1)"),
            "{error}"
        );
    }

    #[test]
    fn returns_err_when_padding_symbols_are_present_and_padding_is_disabled() {
        // With `--no-pad` the padding symbol is simply not part of the alphabet any more.
        let expectations = [
            ("Zg==", '=', 3),
            ("Zm8=", '=', 4),
            ("Zm9vYg==", '=', 7),
            ("Zm9vYmE=", '=', 8),
        ];

        for (input, expected_symbol, expected_position) in expectations {
            let error = decode(input.as_bytes(), STANDARD_ALPHABET, None).unwrap_err();

            assert!(
                error.contains(&format!(
                    "invalid symbol '{expected_symbol}' (position: {expected_position})"
                )),
                "unexpected error for {input:?}: {error}"
            );
        }
    }

    #[test]
    fn returns_err_when_a_body_quantum_contains_a_symbol_outside_the_alphabet() {
        // RFC 4648 section 3.3: symbols outside the alphabet must be rejected.
        let expectations = [
            ("!m9vYmFy", '!', 1),
            ("Zm=vYmFy", '=', 3),
            ("Zm9v!m9vYmFy", '!', 5),
            ("Zm9vYm9v!m9vYmFy", '!', 9),
        ];

        for (input, expected_symbol, expected_position) in expectations {
            let error = decode(input.as_bytes(), STANDARD_ALPHABET, Some(b'=')).unwrap_err();

            assert!(
                error.contains(&format!(
                    "invalid symbol '{expected_symbol}' (position: {expected_position})"
                )),
                "unexpected error for {input:?}: {error}"
            );
        }
    }

    #[test]
    fn returns_err_when_a_body_quantum_contains_a_symbol_outside_the_alphabet_without_padding() {
        let expectations = [
            ("!m9vYmFy", '!', 1),
            ("Zm=vYmFy", '=', 3),
            ("Zm9v!m9vYmFy", '!', 5),
            ("Zm9v!m9vYA", '!', 5),
        ];

        for (input, expected_symbol, expected_position) in expectations {
            let error = decode(input.as_bytes(), STANDARD_ALPHABET, None).unwrap_err();

            assert!(
                error.contains(&format!(
                    "'{expected_symbol}' (position: {expected_position})"
                )),
                "unexpected error for {input:?}: {error}"
            );
        }
    }

    #[test]
    fn returns_err_when_the_final_quantum_contains_a_symbol_outside_the_alphabet() {
        let expectations = [
            ("Zm9v!mFy", '!', 5),
            ("Zm9vYm!y", '!', 7),
            ("Z!==", '!', 2),
            ("Z=g=", '=', 2),
            ("=g==", '=', 1),
        ];

        for (input, expected_symbol, expected_position) in expectations {
            let error = decode(input.as_bytes(), STANDARD_ALPHABET, Some(b'=')).unwrap_err();

            assert!(
                error.contains(&format!(
                    "'{expected_symbol}' (position: {expected_position})"
                )),
                "unexpected error for {input:?}: {error}"
            );
        }
    }

    #[test]
    fn returns_err_when_the_final_quantum_contains_a_symbol_outside_the_alphabet_without_padding() {
        let expectations = [
            ("Zm9v!mFy", '!', 5),
            ("Zm9vYm!y", '!', 7),
            ("Zm9v!m", '!', 5),
            ("Z!", '!', 2),
        ];

        for (input, expected_symbol, expected_position) in expectations {
            let error = decode(input.as_bytes(), STANDARD_ALPHABET, None).unwrap_err();

            assert!(
                error.contains(&format!(
                    "'{expected_symbol}' (position: {expected_position})"
                )),
                "unexpected error for {input:?}: {error}"
            );
        }
    }

    #[test]
    fn returns_err_when_padding_appears_before_the_final_quantum() {
        // Concatenated padded payloads are not a single valid encoding.
        for input in ["Zg==Zg==", "Zm8=Zm9v", "====Zm9v"] {
            let error = decode(input.as_bytes(), STANDARD_ALPHABET, Some(b'=')).unwrap_err();

            assert!(
                error.contains("invalid symbol '='"),
                "unexpected error for {input:?}: {error}"
            );
        }
    }

    #[test]
    fn returns_err_when_more_than_two_padding_symbols_are_present() {
        let expectations = [("Z===", 3), ("====", 4), ("Zm9vY===", 3), ("Zm9v====", 4)];

        for (input, expected_count) in expectations {
            let error = decode(input.as_bytes(), STANDARD_ALPHABET, Some(b'=')).unwrap_err();

            assert!(
                error.contains(&format!(
                    "not more than 2 padding symbols is expected, but {expected_count} found"
                )),
                "unexpected error for {input:?}: {error}"
            );
        }
    }

    #[test]
    fn does_not_count_padding_symbols_when_padding_is_disabled() {
        // The "more than 2 padding symbols" guard is unreachable: they are invalid symbols instead.
        for input in ["Z===", "====", "Zm9vY===", "Zm9v===="] {
            let error = decode(input.as_bytes(), STANDARD_ALPHABET, None).unwrap_err();

            assert!(
                error.contains("invalid symbol '='"),
                "unexpected error for {input:?}: {error}"
            );
        }
    }

    #[test]
    fn returns_err_when_a_two_symbol_quantum_has_non_zero_pad_bits() {
        // RFC 4648 section 3.5: "Zg==" is the only canonical spelling of "f";
        // indices 33..=47 carry the same data byte plus dirty pad bits.
        for index in 33_usize..=47 {
            let symbol = STANDARD_ALPHABET.as_bytes()[index] as char;
            let input = format!("Z{symbol}==");

            let error = decode(input.as_bytes(), STANDARD_ALPHABET, Some(b'=')).unwrap_err();

            assert!(
                error.contains(&format!(
                    "non-zero padding bits detected in input payload: '{symbol}' (position: 2)"
                )),
                "unexpected error for {input:?}: {error}"
            );
        }

        assert_eq!(
            decode(b"Zg==", STANDARD_ALPHABET, Some(b'=')).unwrap(),
            b"f"
        );
    }

    #[test]
    fn returns_err_when_a_two_symbol_quantum_has_non_zero_pad_bits_without_padding() {
        // RFC 4648 section 3.5 applies to unpadded payloads too: "Zg" is the only spelling of "f".
        // indices 33..=47 carry the same data byte plus dirty pad bits.
        for index in 33_usize..=47 {
            let symbol = STANDARD_ALPHABET.as_bytes()[index] as char;
            let input = format!("Z{symbol}");

            let error = decode(input.as_bytes(), STANDARD_ALPHABET, None).unwrap_err();

            assert!(
                error.contains(&format!(
                    "non-zero padding bits detected in input payload: '{symbol}' (position: 2)"
                )),
                "unexpected error for {input:?}: {error}"
            );
        }

        assert_eq!(decode(b"Zg", STANDARD_ALPHABET, None).unwrap(), b"f");
    }

    #[test]
    fn returns_err_when_a_three_symbol_quantum_has_non_zero_pad_bits() {
        // "Zm8=" is the only canonical spelling of "fo"; indices 61..=63 add pad bits.
        for index in 61_usize..=63 {
            let symbol = STANDARD_ALPHABET.as_bytes()[index] as char;
            let input = format!("Zm{symbol}=");

            let error = decode(input.as_bytes(), STANDARD_ALPHABET, Some(b'=')).unwrap_err();

            assert!(
                error.contains(&format!(
                    "non-zero padding bits detected in input payload: '{symbol}' (position: 3)"
                )),
                "unexpected error for {input:?}: {error}"
            );
        }

        assert_eq!(
            decode(b"Zm8=", STANDARD_ALPHABET, Some(b'=')).unwrap(),
            b"fo"
        );
    }

    #[test]
    fn returns_err_when_a_three_symbol_quantum_has_non_zero_pad_bits_without_padding() {
        for index in 61_usize..=63 {
            let symbol = STANDARD_ALPHABET.as_bytes()[index] as char;
            let input = format!("Zm{symbol}");

            let error = decode(input.as_bytes(), STANDARD_ALPHABET, None).unwrap_err();

            assert!(
                error.contains(&format!(
                    "non-zero padding bits detected in input payload: '{symbol}' (position: 3)"
                )),
                "unexpected error for {input:?}: {error}"
            );
        }

        assert_eq!(decode(b"Zm8", STANDARD_ALPHABET, None).unwrap(), b"fo");
    }

    #[test]
    fn accepts_a_final_symbol_only_when_its_pad_bits_are_zero() {
        for (index, &symbol) in STANDARD_ALPHABET.as_bytes().iter().enumerate() {
            let symbol = symbol as char;

            // Two data symbols leave four pad bits.
            let two_symbol_quantum = format!("A{symbol}==");
            assert_eq!(
                decode(two_symbol_quantum.as_bytes(), STANDARD_ALPHABET, Some(b'=')).is_ok(),
                index % 16 == 0,
                "for {two_symbol_quantum:?}"
            );

            // Three data symbols leave two pad bits.
            let three_symbol_quantum = format!("AA{symbol}=");
            assert_eq!(
                decode(
                    three_symbol_quantum.as_bytes(),
                    STANDARD_ALPHABET,
                    Some(b'=')
                )
                .is_ok(),
                index % 4 == 0,
                "for {three_symbol_quantum:?}"
            );

            // A full quantum has no pad bits, so every symbol is canonical.
            let full_quantum = format!("AAA{symbol}");
            assert!(
                decode(full_quantum.as_bytes(), STANDARD_ALPHABET, Some(b'=')).is_ok(),
                "for {full_quantum:?}"
            );
        }
    }

    #[test]
    fn accepts_a_final_symbol_only_when_its_pad_bits_are_zero_without_padding() {
        for (index, &symbol) in STANDARD_ALPHABET.as_bytes().iter().enumerate() {
            let symbol = symbol as char;

            // Two data symbols leave four pad bits.
            let two_symbol_quantum = format!("A{symbol}");
            assert_eq!(
                decode(two_symbol_quantum.as_bytes(), STANDARD_ALPHABET, None).is_ok(),
                index % 16 == 0,
                "for {two_symbol_quantum:?}"
            );

            // Three data symbols leave two pad bits.
            let three_symbol_quantum = format!("AA{symbol}");
            assert_eq!(
                decode(three_symbol_quantum.as_bytes(), STANDARD_ALPHABET, None).is_ok(),
                index % 4 == 0,
                "for {three_symbol_quantum:?}"
            );

            // A full quantum has no pad bits, so every symbol is canonical.
            let full_quantum = format!("AAA{symbol}");
            assert!(
                decode(full_quantum.as_bytes(), STANDARD_ALPHABET, None).is_ok(),
                "for {full_quantum:?}"
            );
        }
    }

    #[test]
    fn matches_reference_decoder_on_non_canonical_final_symbols() {
        let engine = reference_engine(STANDARD_ALPHABET, b'=');

        for &symbol in STANDARD_ALPHABET.as_bytes() {
            let symbol = symbol as char;

            for quantum in [format!("A{symbol}=="), format!("AA{symbol}=")] {
                assert_eq!(
                    decode(quantum.as_bytes(), STANDARD_ALPHABET, Some(b'=')).is_ok(),
                    engine.decode(&quantum).is_ok(),
                    "for {quantum:?}"
                );
            }
        }
    }

    #[test]
    fn rejects_non_zero_pad_bits_for_every_alphabet() {
        // Index 33 is never a multiple of 16, so its pad bits are dirty in any alphabet.
        for alphabet in [STANDARD_ALPHABET, URL_SAFE_ALPHABET, DIGITS_FIRST_ALPHABET] {
            let symbol = alphabet.as_bytes()[33] as char;
            let first_symbol = alphabet.as_bytes()[0] as char;
            let input = format!("{first_symbol}{symbol}==");

            let error = decode(input.as_bytes(), alphabet, Some(b'=')).unwrap_err();

            assert!(
                error.contains("non-zero padding bits"),
                "unexpected error for {input:?} with {alphabet}: {error}"
            );
        }
    }

    #[test]
    fn reports_non_zero_pad_bit_positions_relative_to_the_original_input() {
        let expectations = [("  Zm9vZh==", 'h', 8), ("\r\n\tZm9vZm9=\r\n", '9', 10)];

        for (input, expected_symbol, expected_position) in expectations {
            let error = decode(input.as_bytes(), STANDARD_ALPHABET, Some(b'=')).unwrap_err();

            assert!(
                error.contains(&format!(
                    "'{expected_symbol}' (position: {expected_position})"
                )),
                "unexpected error for {input:?}: {error}"
            );
            assert_eq!(
                input.as_bytes()[expected_position - 1] as char,
                expected_symbol,
                "test vector {input:?} must point at the offending byte"
            );
        }
    }

    #[test]
    fn rejects_non_zero_pad_bits_for_every_alphabet_without_padding() {
        for alphabet in [STANDARD_ALPHABET, URL_SAFE_ALPHABET, DIGITS_FIRST_ALPHABET] {
            let symbol = alphabet.as_bytes()[33] as char;
            let first_symbol = alphabet.as_bytes()[0] as char;
            let input = format!("{first_symbol}{symbol}");

            let error = decode(input.as_bytes(), alphabet, None).unwrap_err();

            assert!(
                error.contains("non-zero padding bits"),
                "unexpected error for {input:?} with {alphabet}: {error}"
            );
        }
    }

    #[test]
    fn reports_non_zero_pad_bit_positions_relative_to_the_original_input_without_padding() {
        let expectations = [("  Zm9vZh", 'h', 8), ("\r\n\tZm9vZm9\r\n", '9', 10)];

        for (input, expected_symbol, expected_position) in expectations {
            let error = decode(input.as_bytes(), STANDARD_ALPHABET, None).unwrap_err();

            assert!(
                error.contains(&format!(
                    "'{expected_symbol}' (position: {expected_position})"
                )),
                "unexpected error for {input:?}: {error}"
            );
            assert_eq!(
                input.as_bytes()[expected_position - 1] as char,
                expected_symbol,
                "test vector {input:?} must point at the offending byte"
            );
        }
    }

    #[test]
    fn calculates_decoded_length_for_known_inputs() {
        let expectations = [
            (0, 0),
            (1, 3),
            (2, 3),
            (3, 3),
            (4, 3),
            (5, 6),
            (7, 6),
            (8, 6),
            (12, 9),
            (400, 300),
            (404, 303),
        ];

        for (encoded_length, expected) in expectations {
            assert_eq!(
                calculate_decoded_length(encoded_length),
                expected,
                "for encoded length {encoded_length}"
            );
        }
    }

    #[test]
    fn calculates_decoded_length_as_rounded_up_quanta_count() {
        // A partial quantum still produces output, so it has to be counted as a whole one.
        for encoded_length in 0..=1_000_usize {
            assert_eq!(
                calculate_decoded_length(encoded_length),
                encoded_length.div_ceil(4) * 3,
                "for encoded length {encoded_length}"
            );
        }
    }

    #[test]
    fn calculates_decoded_length_without_losing_precision_on_large_payloads() {
        // An `f32` stops representing consecutive integers exactly at 2^24, which would make
        // the estimate undershoot for payloads larger than ~16 MB.
        let boundaries = [
            16_777_215_usize,
            16_777_216,
            16_777_217,
            16_777_219,
            33_554_433,
            1_000_000_001,
            4_294_967_293,
        ];

        for encoded_length in boundaries {
            assert_eq!(
                calculate_decoded_length(encoded_length),
                encoded_length.div_ceil(4) * 3,
                "for encoded length {encoded_length}"
            );
        }
    }

    #[test]
    fn decoded_length_is_an_upper_bound_of_any_payload_size() {
        // Six bits per symbol is the theoretical maximum any payload can decode to.
        for encoded_length in 0..=1_000_usize {
            assert!(
                calculate_decoded_length(encoded_length) >= (encoded_length * 6) / 8,
                "for encoded length {encoded_length}"
            );
        }
    }

    #[test]
    fn decoded_length_is_an_upper_bound_of_the_real_payload_size() {
        let mut rng = XorShift64(0xdead_beef_cafe_f00d);

        for length in 0..=64 {
            let input = rng.bytes(length);
            let encoded = encode(&input, STANDARD_ALPHABET, Some(b'='));
            let decoded = decode(encoded.as_bytes(), STANDARD_ALPHABET, Some(b'=')).unwrap();
            let estimate = calculate_decoded_length(encoded.len());

            assert_eq!(decoded.len(), length, "length {length}");
            assert!(
                estimate >= length && estimate - length <= 2,
                "estimate {estimate} must overshoot length {length} by at most 2"
            );
        }
    }

    #[test]
    fn decodes_unpadded_payloads_to_their_original_length() {
        let mut rng = XorShift64(0xdead_beef_cafe_f00d);

        for length in 0..=64 {
            let input = rng.bytes(length);
            let encoded = encode(&input, STANDARD_ALPHABET, None);
            let decoded = decode(encoded.as_bytes(), STANDARD_ALPHABET, None).unwrap();
            let estimate = calculate_decoded_length(encoded.len());

            assert_eq!(decoded, input, "length {length}");
            assert!(
                estimate >= length && estimate - length <= 2,
                "estimate {estimate} must overshoot length {length} by at most 2"
            );
        }
    }

    #[test]
    fn packs_decoded_values_as_big_endian_bytes() {
        let mut buffer = Vec::new();
        pack_decoded_value(0x11_22_33_44, None, &mut buffer);

        assert_eq!(buffer, [0x11, 0x22, 0x33], "the low byte is always dropped");
    }

    #[test]
    fn packs_only_the_requested_number_of_meaningful_bytes() {
        let expectations: [(usize, &[u8]); 4] = [
            (0, &[]),
            (1, &[0xaa]),
            (2, &[0xaa, 0xbb]),
            (3, &[0xaa, 0xbb, 0xcc]),
        ];

        for (meaningful_bytes_count, expected) in expectations {
            let mut buffer = Vec::new();
            pack_decoded_value(0xaa_bb_cc_dd, Some(meaningful_bytes_count), &mut buffer);

            assert_eq!(buffer, expected, "for {meaningful_bytes_count} bytes");
        }
    }

    #[test]
    fn packs_values_by_appending_to_the_buffer() {
        let mut buffer = vec![0x01];
        pack_decoded_value(0x11_22_33_44, None, &mut buffer);
        pack_decoded_value(0xaa_bb_cc_dd, Some(1), &mut buffer);

        assert_eq!(buffer, [0x01, 0x11, 0x22, 0x33, 0xaa]);
    }

    #[test]
    fn decodes_empty_input_to_empty_output() {
        for alphabet in [STANDARD_ALPHABET, URL_SAFE_ALPHABET, DIGITS_FIRST_ALPHABET] {
            assert_eq!(
                decode(&[], alphabet, Some(b'=')).unwrap(),
                Vec::<u8>::new(),
                "alphabet {alphabet}"
            );
        }
    }

    #[test]
    fn decodes_empty_input_to_empty_output_without_padding() {
        for alphabet in [STANDARD_ALPHABET, URL_SAFE_ALPHABET, DIGITS_FIRST_ALPHABET] {
            assert_eq!(
                decode(&[], alphabet, None).unwrap(),
                Vec::<u8>::new(),
                "alphabet {alphabet}"
            );
        }
    }

    #[test]
    fn decodes_whitespace_only_input_to_empty_output() {
        for input in [" ", "\n", "  \r\n\t ", "\u{c}\u{c}"] {
            assert_eq!(
                decode(input.as_bytes(), STANDARD_ALPHABET, Some(b'=')).unwrap(),
                Vec::<u8>::new(),
                "for {input:?}"
            );
        }
    }

    #[test]
    fn decodes_whitespace_only_input_to_empty_output_without_padding() {
        for input in [" ", "\n", "  \r\n\t ", "\u{c}\u{c}"] {
            assert_eq!(
                decode(input.as_bytes(), STANDARD_ALPHABET, None).unwrap(),
                Vec::<u8>::new(),
                "for {input:?}"
            );
        }
    }

    #[test]
    fn honours_whitespace_padding_symbols() {
        // `validate_padding_symbol` accepts 0x20, so the engine must accept it too.
        assert_eq!(
            decode(b"Zg  ", STANDARD_ALPHABET, Some(b' ')).unwrap(),
            b"f"
        );
        assert_eq!(
            decode(b"Zm8 ", STANDARD_ALPHABET, Some(b' ')).unwrap(),
            b"fo"
        );
        assert_eq!(
            decode(b"\r\nZm9vYmE \r\n", STANDARD_ALPHABET, Some(b' ')).unwrap(),
            b"fooba"
        );
    }

    #[test]
    fn reports_invalid_symbol_positions_relative_to_the_original_input() {
        // Positions are 1-based offsets into the payload as the caller supplied it.
        let expectations = [
            // Invalid symbol inside a body quantum.
            ("  !m9vYmFy", '!', 3),
            ("\n\n\nZm9v!m9vYmFy", '!', 8),
            // Invalid symbol inside the final quantum.
            ("  Zm9v!mFy", '!', 7),
            ("\t\tZm9vYm!y", '!', 9),
            ("   Z!==", '!', 5),
            ("\r\n Zm9vY=g=\r\n", '=', 9),
        ];

        for (input, expected_symbol, expected_position) in expectations {
            let error = decode(input.as_bytes(), STANDARD_ALPHABET, Some(b'=')).unwrap_err();

            assert!(
                error.contains(&format!(
                    "'{expected_symbol}' (position: {expected_position})"
                )),
                "unexpected error for {input:?}: {error}"
            );
            assert_eq!(
                input.as_bytes()[expected_position - 1] as char,
                expected_symbol,
                "test vector {input:?} must point at the offending byte"
            );
        }
    }

    #[test]
    fn reports_invalid_symbol_positions_relative_to_the_original_input_without_padding() {
        let expectations = [
            // Invalid symbol inside a body quantum.
            ("  !m9vYmFy", '!', 3),
            ("\n\n\nZm9v!m9vYmFy", '!', 8),
            // Invalid symbol inside the final quantum.
            ("  Zm9v!mFy", '!', 7),
            ("\t\tZm9vYm!y", '!', 9),
            ("   Z!", '!', 5),
            ("\r\n Zm9vY=g\r\n", '=', 9),
        ];

        for (input, expected_symbol, expected_position) in expectations {
            let error = decode(input.as_bytes(), STANDARD_ALPHABET, None).unwrap_err();

            assert!(
                error.contains(&format!(
                    "'{expected_symbol}' (position: {expected_position})"
                )),
                "unexpected error for {input:?}: {error}"
            );
            assert_eq!(
                input.as_bytes()[expected_position - 1] as char,
                expected_symbol,
                "test vector {input:?} must point at the offending byte"
            );
        }
    }

    #[test]
    fn trims_every_ascii_whitespace_symbol_from_both_ends() {
        for whitespace in [b' ', b'\t', b'\n', 0x0c, b'\r'] {
            let input = [whitespace, whitespace, b'Z', b'g', b'=', b'=', whitespace];

            assert_eq!(
                trim_whitespaces(&input, Some(b'=')),
                (b"Zg==".as_slice(), 2),
                "whitespace {whitespace:#04x}"
            );
        }
    }

    #[test]
    fn keeps_payloads_without_surrounding_whitespace_untouched() {
        for input in [b"Zm9vYmFy".as_slice(), b"Zg==", b"Z"] {
            assert_eq!(
                trim_whitespaces(input, Some(b'=')),
                (input, 0),
                "for {input:02x?}"
            );
        }
    }

    #[test]
    fn keeps_whitespace_inside_the_payload() {
        assert_eq!(
            trim_whitespaces(b" Zm9v YmFy ", Some(b'=')),
            (b"Zm9v YmFy".as_slice(), 1)
        );
    }

    #[test]
    fn returns_an_empty_slice_for_empty_and_whitespace_only_input() {
        assert_eq!(trim_whitespaces(b"", Some(b'=')), (b"".as_slice(), 0));

        for input in [b" ".as_slice(), b"\n", b"  \r\n\t "] {
            assert_eq!(
                trim_whitespaces(input, Some(b'=')),
                (b"".as_slice(), input.len()),
                "for {input:02x?}"
            );
        }
    }

    #[test]
    fn keeps_spaces_when_space_is_the_padding_symbol() {
        assert_eq!(
            trim_whitespaces(b"Zg  ", Some(b' ')),
            (b"Zg  ".as_slice(), 0)
        );
        assert_eq!(
            trim_whitespaces(b"\r\nZm8 \r\n", Some(b' ')),
            (b"Zm8 ".as_slice(), 2)
        );
        // A space-padded payload is no longer trimmable from the start.
        assert_eq!(
            trim_whitespaces(b" Zg  ", Some(b' ')),
            (b" Zg  ".as_slice(), 0)
        );
        // First occurrence of the padding symbol will stop trimming.
        assert_eq!(
            trim_whitespaces(b"\t\n  Zg  ", Some(b' ')),
            (b"  Zg  ".as_slice(), 2)
        );
        assert_eq!(
            trim_whitespaces(b" \t\n  Zg  ", Some(b' ')),
            (b" \t\n  Zg  ".as_slice(), 0)
        );
        assert_eq!(
            trim_whitespaces(b" Zg  \n\t", Some(b' ')),
            (b" Zg  ".as_slice(), 0)
        );
        assert_eq!(
            trim_whitespaces(b" Zg  \n\t ", Some(b' ')),
            (b" Zg  \n\t ".as_slice(), 0)
        );
    }

    #[test]
    fn does_not_trim_control_characters_that_are_not_ascii_whitespace() {
        for symbol in [0x00_u8, 0x0b, 0x1f, 0x7f] {
            let input = [symbol, b'Z', b'g', b'=', b'=', symbol];

            assert_eq!(
                trim_whitespaces(&input, Some(b'=')),
                (input.as_slice(), 0),
                "symbol {symbol:#04x}"
            );
        }
    }

    #[test]
    fn matches_trim_ascii_for_non_whitespace_padding_symbols() {
        // `trim_whitespaces` replaced `<[u8]>::trim_ascii`; the two must agree
        // whenever the padding symbol is not itself an ASCII whitespace.
        let payloads: [&[u8]; 8] = [
            b"",
            b"   ",
            b"\t\n\x0c\r ",
            b"Zm9vYmFy",
            b" Zm9vYmFy ",
            b"\r\n\tZm9vYmFy\x0c ",
            b"Zm9v YmFy",
            b"\0Zm9vYmFy\0",
        ];

        for payload in payloads {
            let (trimmed, trimmed_from_start) = trim_whitespaces(payload, Some(b'='));

            assert_eq!(trimmed, payload.trim_ascii(), "for {payload:02x?}");
            assert_eq!(
                trimmed_from_start,
                payload.len() - payload.trim_ascii_start().len(),
                "offset for {payload:02x?}"
            );
        }
    }

    #[test]
    fn matches_trim_ascii_when_padding_is_disabled() {
        // No padding symbol means nothing has to survive trimming, spaces included.
        let payloads: [&[u8]; 9] = [
            b"",
            b"   ",
            b"\t\n\x0c\r ",
            b"Zm9vYmFy",
            b" Zm9vYmFy ",
            b"\r\n\tZm9vYmFy\x0c ",
            b"Zm9v YmFy",
            b"\0Zm9vYmFy\0",
            b" Zg  ",
        ];

        for payload in payloads {
            let (trimmed, trimmed_from_start) = trim_whitespaces(payload, None);

            assert_eq!(trimmed, payload.trim_ascii(), "for {payload:02x?}");
            assert_eq!(
                trimmed_from_start,
                payload.len() - payload.trim_ascii_start().len(),
                "offset for {payload:02x?}"
            );
        }
    }

    #[test]
    fn trims_spaces_when_padding_is_disabled() {
        // Mirror image of `keeps_spaces_when_space_is_the_padding_symbol`.
        assert_eq!(trim_whitespaces(b"Zg  ", None), (b"Zg".as_slice(), 0));
        assert_eq!(
            trim_whitespaces(b"\r\nZm8 \r\n", None),
            (b"Zm8".as_slice(), 2)
        );
        assert_eq!(trim_whitespaces(b" Zg  ", None), (b"Zg".as_slice(), 1));
        assert_eq!(trim_whitespaces(b"\t\n  Zg  ", None), (b"Zg".as_slice(), 4));
        assert_eq!(trim_whitespaces(b" Zg  \n\t ", None), (b"Zg".as_slice(), 1));
    }

    #[test]
    fn does_not_trim_control_characters_that_are_not_ascii_whitespace_when_padding_is_disabled() {
        for symbol in [0x00_u8, 0x0b, 0x1f, 0x7f] {
            let input = [symbol, b'Z', b'g', symbol];

            assert_eq!(
                trim_whitespaces(&input, None),
                (input.as_slice(), 0),
                "symbol {symbol:#04x}"
            );
        }
    }
}
