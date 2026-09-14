use crate::base64::alphabet::{
    build_encoding_alphabet_mapping, validate_alphabet, validate_padding_symbol,
};
use crate::base64::engine::encode_with_alphabet;
use crate::commands::base64::Args;
use crate::io::{read_input_bytes, write_output_string};

pub fn run(args: Args) -> Result<(), String> {
    let alphabet = validate_alphabet(args.alphabet, args.complementary_symbols)?;

    let padding_symbol = validate_padding_symbol(args.padding_symbol, &alphabet)?;

    let alphabet_mapping = build_encoding_alphabet_mapping(&alphabet);

    let input_data = read_input_bytes(args.input)?;

    let encoded_data = encode_with_alphabet(&input_data, alphabet_mapping, padding_symbol);

    write_output_string(args.output, encoded_data)
}

#[cfg(test)]
mod tests {
    use super::{Args, run};
    use crate::base64::alphabet::DEFAULT_ALPHABET;
    use crate::io::IoTarget;
    use std::fs;
    use std::path::PathBuf;
    use tempfile::{TempDir, tempdir};

    const DIGITS_FIRST_ALPHABET: &str =
        "0123456789ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz+/";

    fn input_target(dir: &TempDir, bytes: &[u8]) -> IoTarget {
        let path = dir.path().join("input.bin");
        fs::write(&path, bytes).unwrap();
        IoTarget::File(path)
    }

    fn output_target(dir: &TempDir) -> (IoTarget, PathBuf) {
        let path = dir.path().join("output.txt");
        (IoTarget::File(path.clone()), path)
    }

    fn args(input: IoTarget, output: IoTarget) -> Args {
        Args {
            output,
            input,
            alphabet: None,
            complementary_symbols: None,
            padding_symbol: '=',
        }
    }

    #[test]
    fn encodes_file_input_with_default_options() {
        let dir = tempdir().unwrap();
        let input = input_target(&dir, b"foobar");
        let (output, path) = output_target(&dir);

        run(args(input, output)).unwrap();

        assert_eq!(fs::read_to_string(path).unwrap(), "Zm9vYmFy");
    }

    #[test]
    fn encodes_binary_file_input() {
        let dir = tempdir().unwrap();
        let input = input_target(&dir, &[0x14, 0xfb, 0x9c, 0x03, 0xd9, 0x7e]);
        let (output, path) = output_target(&dir);

        run(args(input, output)).unwrap();

        assert_eq!(fs::read_to_string(path).unwrap(), "FPucA9l+");
    }

    #[test]
    fn encodes_with_custom_alphabet() {
        let dir = tempdir().unwrap();
        let input = input_target(&dir, b"foobar");
        let (output, path) = output_target(&dir);
        let mut args = args(input, output);
        args.alphabet = Some(DIGITS_FIRST_ALPHABET.to_string());

        run(args).unwrap();

        assert_eq!(fs::read_to_string(path).unwrap(), "PczlOc5o");
    }

    #[test]
    fn encodes_with_complementary_symbols() {
        let dir = tempdir().unwrap();
        let input = input_target(&dir, &[0xfb, 0xff, 0xbf]);
        let (output, path) = output_target(&dir);
        let mut args = args(input, output);
        args.complementary_symbols = Some("-_".to_string());

        run(args).unwrap();

        assert_eq!(fs::read_to_string(path).unwrap(), "-_-_");
    }

    #[test]
    fn encodes_with_custom_padding_symbol() {
        let dir = tempdir().unwrap();
        let input = input_target(&dir, b"f");
        let (output, path) = output_target(&dir);
        let mut args = args(input, output);
        args.padding_symbol = '.';

        run(args).unwrap();

        assert_eq!(fs::read_to_string(path).unwrap(), "Zg..");
    }

    #[test]
    fn returns_err_when_alphabet_is_invalid() {
        let dir = tempdir().unwrap();
        let input = input_target(&dir, b"foobar");
        let (output, path) = output_target(&dir);
        let mut args = args(input, output);
        args.alphabet = Some("too-short".to_string());

        assert!(run(args).is_err());
        assert!(!path.exists(), "no output must be produced on failure");
    }

    #[test]
    fn returns_err_when_alphabet_and_complementary_symbols_are_both_set() {
        let dir = tempdir().unwrap();
        let input = input_target(&dir, b"foobar");
        let (output, _) = output_target(&dir);
        let mut args = args(input, output);
        args.alphabet = Some(DEFAULT_ALPHABET.to_string());
        args.complementary_symbols = Some("-_".to_string());

        assert!(run(args).is_err());
    }

    #[test]
    fn returns_err_when_padding_symbol_is_part_of_the_alphabet() {
        let dir = tempdir().unwrap();
        let input = input_target(&dir, b"foobar");
        let (output, _) = output_target(&dir);
        let mut args = args(input, output);
        args.padding_symbol = '+';

        assert!(run(args).is_err());
    }

    #[test]
    fn returns_err_when_padding_symbol_is_non_printable() {
        let dir = tempdir().unwrap();
        let input = input_target(&dir, b"foobar");
        let (output, _) = output_target(&dir);
        let mut args = args(input, output);
        args.padding_symbol = '\n';

        assert!(run(args).is_err());
    }

    #[test]
    fn returns_err_when_input_file_is_missing() {
        let dir = tempdir().unwrap();
        let (output, _) = output_target(&dir);
        let input = IoTarget::File(dir.path().join("missing.bin"));

        assert!(run(args(input, output)).is_err());
    }

    #[test]
    fn returns_err_when_output_file_already_exists() {
        let dir = tempdir().unwrap();
        let input = input_target(&dir, b"foobar");
        let (output, path) = output_target(&dir);
        fs::write(&path, "existing").unwrap();

        assert!(run(args(input, output)).is_err());
        assert_eq!(fs::read_to_string(path).unwrap(), "existing");
    }
}
