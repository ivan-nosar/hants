use crate::base64::alphabet::{
    build_decoding_alphabet_mapping, validate_alphabet, validate_padding_symbol,
};
use crate::base64::engine::decode_with_alphabet;
use crate::commands::base64::Args;
use crate::io::{read_input_bytes, write_output_bytes};

pub fn run(args: Args) -> Result<(), String> {
    let alphabet = validate_alphabet(args.alphabet, args.complementary_symbols)?;

    let padding_symbol = validate_padding_symbol(args.padding_symbol, args.no_pad, &alphabet)?;

    let alphabet_mapping = build_decoding_alphabet_mapping(&alphabet);

    let input_data = read_input_bytes(args.input)?;

    let decoded_data = decode_with_alphabet(&input_data, alphabet_mapping, padding_symbol)?;

    write_output_bytes(args.output, &decoded_data)
}

#[cfg(test)]
mod tests {
    use super::{Args, run};
    use crate::base64::alphabet::DEFAULT_ALPHABET;
    use crate::commands::base64::encode;
    use crate::io::IoTarget;
    use std::fs;
    use std::path::PathBuf;
    use tempfile::{TempDir, tempdir};

    const DIGITS_FIRST_ALPHABET: &str =
        "0123456789ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz+/";

    fn input_target(dir: &TempDir, payload: &[u8]) -> IoTarget {
        let path = dir.path().join("input.txt");
        fs::write(&path, payload).unwrap();
        IoTarget::File(path)
    }

    fn output_target(dir: &TempDir) -> (IoTarget, PathBuf) {
        let path = dir.path().join("output.bin");
        (IoTarget::File(path.clone()), path)
    }

    fn args(input: IoTarget, output: IoTarget) -> Args {
        Args {
            output,
            input,
            alphabet: None,
            complementary_symbols: None,
            padding_symbol: '=',
            no_pad: false,
        }
    }

    #[test]
    fn decodes_file_input_with_default_options() {
        let dir = tempdir().unwrap();
        let input = input_target(&dir, b"Zm9vYmFy");
        let (output, path) = output_target(&dir);

        run(args(input, output)).unwrap();

        assert_eq!(fs::read(path).unwrap(), b"foobar");
    }

    #[test]
    fn decodes_to_binary_file_output() {
        let dir = tempdir().unwrap();
        let input = input_target(&dir, b"FPucA9l+");
        let (output, path) = output_target(&dir);

        run(args(input, output)).unwrap();

        assert_eq!(
            fs::read(path).unwrap(),
            [0x14, 0xfb, 0x9c, 0x03, 0xd9, 0x7e]
        );
    }

    #[test]
    fn decodes_padded_input() {
        let dir = tempdir().unwrap();
        let input = input_target(&dir, b"Zm9vYmE=");
        let (output, path) = output_target(&dir);

        run(args(input, output)).unwrap();

        assert_eq!(fs::read(path).unwrap(), b"fooba");
    }

    #[test]
    fn decodes_empty_input_to_empty_output() {
        let dir = tempdir().unwrap();
        let input = input_target(&dir, b"");
        let (output, path) = output_target(&dir);

        run(args(input, output)).unwrap();

        assert_eq!(fs::read(path).unwrap(), Vec::<u8>::new());
    }

    #[test]
    fn ignores_whitespace_surrounding_the_payload() {
        let dir = tempdir().unwrap();
        let input = input_target(&dir, b"  Zm9vYmFy \r\n");
        let (output, path) = output_target(&dir);

        run(args(input, output)).unwrap();

        assert_eq!(fs::read(path).unwrap(), b"foobar");
    }

    #[test]
    fn decodes_with_custom_alphabet() {
        let dir = tempdir().unwrap();
        let input = input_target(&dir, b"PczlOc5o");
        let (output, path) = output_target(&dir);
        let mut args = args(input, output);
        args.alphabet = Some(DIGITS_FIRST_ALPHABET.to_string());

        run(args).unwrap();

        assert_eq!(fs::read(path).unwrap(), b"foobar");
    }

    #[test]
    fn decodes_with_complementary_symbols() {
        let dir = tempdir().unwrap();
        let input = input_target(&dir, b"-_-_");
        let (output, path) = output_target(&dir);
        let mut args = args(input, output);
        args.complementary_symbols = Some("-_".to_string());

        run(args).unwrap();

        assert_eq!(fs::read(path).unwrap(), [0xfb, 0xff, 0xbf]);
    }

    #[test]
    fn decodes_with_custom_padding_symbol() {
        let dir = tempdir().unwrap();
        let input = input_target(&dir, b"Zg..");
        let (output, path) = output_target(&dir);
        let mut args = args(input, output);
        args.padding_symbol = '.';

        run(args).unwrap();

        assert_eq!(fs::read(path).unwrap(), b"f");
    }

    #[test]
    fn decodes_unpadded_input_when_padding_is_disabled() {
        for (payload, expected) in [
            (b"Zg".as_slice(), b"f".as_slice()),
            (b"Zm8", b"fo"),
            (b"Zm9v", b"foo"),
            (b"Zm9vYg", b"foob"),
            (b"Zm9vYmE", b"fooba"),
            (b"Zm9vYmFy", b"foobar"),
        ] {
            let dir = tempdir().unwrap();
            let input = input_target(&dir, payload);
            let (output, path) = output_target(&dir);
            let mut args = args(input, output);
            args.no_pad = true;

            run(args).unwrap();

            assert_eq!(fs::read(path).unwrap(), expected, "for {payload:02x?}");
        }
    }

    #[test]
    fn decodes_to_binary_file_output_without_padding() {
        let dir = tempdir().unwrap();
        let input = input_target(&dir, b"FPucA9k");
        let (output, path) = output_target(&dir);
        let mut args = args(input, output);
        args.no_pad = true;

        run(args).unwrap();

        assert_eq!(fs::read(path).unwrap(), [0x14, 0xfb, 0x9c, 0x03, 0xd9]);
    }

    #[test]
    fn decodes_empty_input_to_empty_output_without_padding() {
        let dir = tempdir().unwrap();
        let input = input_target(&dir, b"");
        let (output, path) = output_target(&dir);
        let mut args = args(input, output);
        args.no_pad = true;

        run(args).unwrap();

        assert_eq!(fs::read(path).unwrap(), Vec::<u8>::new());
    }

    #[test]
    fn ignores_whitespace_surrounding_the_payload_without_padding() {
        let dir = tempdir().unwrap();
        let input = input_target(&dir, b"  Zm9vYmE \r\n");
        let (output, path) = output_target(&dir);
        let mut args = args(input, output);
        args.no_pad = true;

        run(args).unwrap();

        assert_eq!(fs::read(path).unwrap(), b"fooba");
    }

    #[test]
    fn decodes_with_custom_alphabet_without_padding() {
        let dir = tempdir().unwrap();
        let input = input_target(&dir, b"Pcy");
        let (output, path) = output_target(&dir);
        let mut args = args(input, output);
        args.alphabet = Some(DIGITS_FIRST_ALPHABET.to_string());
        args.no_pad = true;

        run(args).unwrap();

        assert_eq!(fs::read(path).unwrap(), b"fo");
    }

    #[test]
    fn decodes_with_complementary_symbols_without_padding() {
        let dir = tempdir().unwrap();
        let input = input_target(&dir, b"-_8");
        let (output, path) = output_target(&dir);
        let mut args = args(input, output);
        args.complementary_symbols = Some("-_".to_string());
        args.no_pad = true;

        run(args).unwrap();

        assert_eq!(fs::read(path).unwrap(), [0xfb, 0xff]);
    }

    #[test]
    fn skips_padding_symbol_validation_when_padding_is_disabled() {
        // Both symbols are rejected by `validate_padding_symbol` unless `--no-pad` short-circuits it.
        for padding_symbol in ['+', '\n'] {
            let dir = tempdir().unwrap();
            let input = input_target(&dir, b"Zg");
            let (output, path) = output_target(&dir);
            let mut args = args(input, output);
            args.padding_symbol = padding_symbol;
            args.no_pad = true;

            run(args).unwrap();

            assert_eq!(fs::read(path).unwrap(), b"f", "for {padding_symbol:?}");
        }
    }

    #[test]
    fn round_trips_output_of_the_encode_command_without_padding() {
        let dir = tempdir().unwrap();
        let source = dir.path().join("source.bin");
        let encoded = dir.path().join("encoded.txt");
        let decoded = dir.path().join("decoded.bin");
        // 256 is not a multiple of 3, so the encoded payload ends with an unpadded tail.
        let payload: Vec<u8> = (0_u8..=255).collect();
        fs::write(&source, &payload).unwrap();

        let mut encode_args = args(IoTarget::File(source), IoTarget::File(encoded.clone()));
        encode_args.no_pad = true;
        encode::run(encode_args).unwrap();

        let mut decode_args = args(IoTarget::File(encoded), IoTarget::File(decoded.clone()));
        decode_args.no_pad = true;
        run(decode_args).unwrap();

        assert_eq!(fs::read(decoded).unwrap(), payload);
    }

    #[test]
    fn returns_err_when_padded_input_is_decoded_with_padding_disabled() {
        // The padding symbol is not part of the alphabet, so it is reported as an invalid symbol.
        for payload in [b"Zg==".as_slice(), b"Zm8=", b"Zm9vYmE="] {
            let dir = tempdir().unwrap();
            let input = input_target(&dir, payload);
            let (output, path) = output_target(&dir);
            let mut args = args(input, output);
            args.no_pad = true;

            assert!(run(args).is_err(), "for {payload:02x?}");
            assert!(!path.exists(), "no output must be produced on failure");
        }
    }

    #[test]
    fn returns_err_when_the_final_quantum_holds_a_single_symbol_and_padding_is_disabled() {
        let dir = tempdir().unwrap();
        let input = input_target(&dir, b"Zm9vY");
        let (output, path) = output_target(&dir);
        let mut args = args(input, output);
        args.no_pad = true;

        assert!(run(args).is_err());
        assert!(!path.exists(), "no output must be produced on failure");
    }

    #[test]
    fn returns_err_when_input_has_non_zero_pad_bits_and_padding_is_disabled() {
        // RFC 4648 section 3.5 still applies without padding: 'Zg' is the only spelling of "f".
        let dir = tempdir().unwrap();
        let input = input_target(&dir, b"Zh");
        let (output, path) = output_target(&dir);
        let mut args = args(input, output);
        args.no_pad = true;

        assert!(run(args).is_err());
        assert!(!path.exists(), "no output must be produced on failure");
    }

    #[test]
    fn returns_err_when_input_contains_symbols_outside_the_alphabet_and_padding_is_disabled() {
        let dir = tempdir().unwrap();
        let input = input_target(&dir, b"Zm9v!mFy");
        let (output, path) = output_target(&dir);
        let mut args = args(input, output);
        args.no_pad = true;

        assert!(run(args).is_err());
        assert!(!path.exists(), "no output must be produced on failure");
    }

    #[test]
    fn round_trips_output_of_the_encode_command() {
        let dir = tempdir().unwrap();
        let source = dir.path().join("source.bin");
        let encoded = dir.path().join("encoded.txt");
        let decoded = dir.path().join("decoded.bin");
        let payload: Vec<u8> = (0_u8..=255).collect();
        fs::write(&source, &payload).unwrap();

        encode::run(args(
            IoTarget::File(source),
            IoTarget::File(encoded.clone()),
        ))
        .unwrap();
        run(args(
            IoTarget::File(encoded),
            IoTarget::File(decoded.clone()),
        ))
        .unwrap();

        assert_eq!(fs::read(decoded).unwrap(), payload);
    }

    #[test]
    fn returns_err_when_input_contains_symbols_outside_the_alphabet() {
        let dir = tempdir().unwrap();
        let input = input_target(&dir, b"Zm9v!mFy");
        let (output, path) = output_target(&dir);

        assert!(run(args(input, output)).is_err());
        assert!(!path.exists(), "no output must be produced on failure");
    }

    #[test]
    fn returns_err_when_input_length_is_not_a_multiple_of_four() {
        let dir = tempdir().unwrap();
        let input = input_target(&dir, b"Zm9");
        let (output, path) = output_target(&dir);

        assert!(run(args(input, output)).is_err());
        assert!(!path.exists(), "no output must be produced on failure");
    }

    #[test]
    fn returns_err_when_input_has_non_zero_pad_bits() {
        // RFC 4648 section 3.5: 'Zg==' is the only canonical spelling of "f".
        let dir = tempdir().unwrap();
        let input = input_target(&dir, b"Zh==");
        let (output, path) = output_target(&dir);

        assert!(run(args(input, output)).is_err());
        assert!(!path.exists(), "no output must be produced on failure");
    }

    #[test]
    fn returns_err_when_input_has_too_many_padding_symbols() {
        let dir = tempdir().unwrap();
        let input = input_target(&dir, b"Zm9vY===");
        let (output, path) = output_target(&dir);

        assert!(run(args(input, output)).is_err());
        assert!(!path.exists(), "no output must be produced on failure");
    }

    #[test]
    fn returns_err_when_alphabet_is_invalid() {
        let dir = tempdir().unwrap();
        let input = input_target(&dir, b"Zm9vYmFy");
        let (output, path) = output_target(&dir);
        let mut args = args(input, output);
        args.alphabet = Some("too-short".to_string());

        assert!(run(args).is_err());
        assert!(!path.exists(), "no output must be produced on failure");
    }

    #[test]
    fn returns_err_when_alphabet_and_complementary_symbols_are_both_set() {
        let dir = tempdir().unwrap();
        let input = input_target(&dir, b"Zm9vYmFy");
        let (output, _) = output_target(&dir);
        let mut args = args(input, output);
        args.alphabet = Some(DEFAULT_ALPHABET.to_string());
        args.complementary_symbols = Some("-_".to_string());

        assert!(run(args).is_err());
    }

    #[test]
    fn returns_err_when_padding_symbol_is_part_of_the_alphabet() {
        let dir = tempdir().unwrap();
        let input = input_target(&dir, b"Zm9vYmFy");
        let (output, _) = output_target(&dir);
        let mut args = args(input, output);
        args.padding_symbol = '+';

        assert!(run(args).is_err());
    }

    #[test]
    fn returns_err_when_padding_symbol_is_non_printable() {
        let dir = tempdir().unwrap();
        let input = input_target(&dir, b"Zm9vYmFy");
        let (output, _) = output_target(&dir);
        let mut args = args(input, output);
        args.padding_symbol = '\n';

        assert!(run(args).is_err());
    }

    #[test]
    fn returns_err_when_input_file_is_missing() {
        let dir = tempdir().unwrap();
        let (output, _) = output_target(&dir);
        let input = IoTarget::File(dir.path().join("missing.txt"));

        assert!(run(args(input, output)).is_err());
    }

    #[test]
    fn returns_err_when_output_file_already_exists() {
        let dir = tempdir().unwrap();
        let input = input_target(&dir, b"Zm9vYmFy");
        let (output, path) = output_target(&dir);
        fs::write(&path, "existing").unwrap();

        assert!(run(args(input, output)).is_err());
        assert_eq!(fs::read_to_string(path).unwrap(), "existing");
    }
}
