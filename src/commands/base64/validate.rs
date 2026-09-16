use crate::base64::alphabet::{validate_alphabet, validate_padding_symbol};
use crate::base64::engine::{validate_with_alphabet};
use crate::commands::base64::Args;
use crate::io::{read_input_bytes, write_output_string};

pub fn run(args: Args) -> Result<(), String> {
    let alphabet = validate_alphabet(args.alphabet, args.complementary_symbols)?;

    let padding_symbol = validate_padding_symbol(args.padding_symbol, &alphabet)?;

    let input_data = read_input_bytes(args.input)?;

    let validation_result = validate_with_alphabet(&input_data, alphabet, padding_symbol);

    write_output_string(args.output, validation_result)
}

#[cfg(test)]
mod tests {
    use super::{Args, run};
    use crate::base64::alphabet::DEFAULT_ALPHABET;
    use crate::io::IoTarget;
    use std::fs;
    use std::path::PathBuf;
    use tempfile::{TempDir, tempdir};

    /// RFC 4648 section 5 "URL and Filename safe" alphabet.
    const URL_SAFE_ALPHABET: &str =
        "ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789-_";

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
    fn accepts_valid_file_input_with_default_options() {
        let dir = tempdir().unwrap();
        let input = input_target(&dir, b"Zm9vYmFy");
        let (output, path) = output_target(&dir);

        run(args(input, output)).unwrap();

        assert_eq!(fs::read_to_string(path).unwrap(), "Valid");
    }

    #[test]
    fn accepts_padded_and_whitespace_surrounded_input() {
        let dir = tempdir().unwrap();
        let input = input_target(&dir, b"  Zm9vYg==\r\n");
        let (output, path) = output_target(&dir);

        run(args(input, output)).unwrap();

        assert_eq!(fs::read_to_string(path).unwrap(), "Valid");
    }

    #[test]
    fn accepts_empty_input() {
        let dir = tempdir().unwrap();
        let input = input_target(&dir, b"");
        let (output, path) = output_target(&dir);

        run(args(input, output)).unwrap();

        assert_eq!(fs::read_to_string(path).unwrap(), "Valid");
    }

    #[test]
    fn reports_invalid_payload_without_returning_err() {
        let dir = tempdir().unwrap();
        let input = input_target(&dir, b"Zm9vYmF!");
        let (output, path) = output_target(&dir);

        run(args(input, output)).unwrap();

        assert_eq!(
            fs::read_to_string(path).unwrap(),
            "Invalid; Invalid symbol detected in input payload: '!' (position: 8)"
        );
    }

    #[test]
    fn reports_malformed_payload_length() {
        let dir = tempdir().unwrap();
        let input = input_target(&dir, b"Zm9vYmF");
        let (output, path) = output_target(&dir);

        run(args(input, output)).unwrap();

        assert_eq!(
            fs::read_to_string(path).unwrap(),
            "Invalid; Input payload is malformed: unexpected tail bytes detected in the end: 'YmF'"
        );
    }

    #[test]
    fn validates_against_custom_alphabet() {
        let dir = tempdir().unwrap();
        let input = input_target(&dir, b"-_-_");
        let (output, path) = output_target(&dir);
        let mut args = args(input, output);
        args.alphabet = Some(URL_SAFE_ALPHABET.to_string());

        run(args).unwrap();

        assert_eq!(fs::read_to_string(path).unwrap(), "Valid");
    }

    #[test]
    fn validates_against_complementary_symbols() {
        let dir = tempdir().unwrap();
        let input = input_target(&dir, b"-_-_");
        let (output, path) = output_target(&dir);
        let mut args = args(input, output);
        args.complementary_symbols = Some("-_".to_string());

        run(args).unwrap();

        assert_eq!(fs::read_to_string(path).unwrap(), "Valid");
    }

    #[test]
    fn rejects_symbols_that_are_not_part_of_the_default_alphabet() {
        let dir = tempdir().unwrap();
        let input = input_target(&dir, b"-_-_");
        let (output, path) = output_target(&dir);

        run(args(input, output)).unwrap();

        assert_eq!(
            fs::read_to_string(path).unwrap(),
            "Invalid; Invalid symbol detected in input payload: '-' (position: 1)"
        );
    }

    #[test]
    fn validates_with_custom_padding_symbol() {
        let dir = tempdir().unwrap();
        let input = input_target(&dir, b"Zg..");
        let (output, path) = output_target(&dir);
        let mut args = args(input, output);
        args.padding_symbol = '.';

        run(args).unwrap();

        assert_eq!(fs::read_to_string(path).unwrap(), "Valid");
    }

    #[test]
    fn rejects_padding_symbol_that_was_not_configured() {
        let dir = tempdir().unwrap();
        let input = input_target(&dir, b"Zg..");
        let (output, path) = output_target(&dir);

        run(args(input, output)).unwrap();

        assert_eq!(
            fs::read_to_string(path).unwrap(),
            "Invalid; Invalid symbol detected in input payload: '.' (position: 3)"
        );
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
        let (output, path) = output_target(&dir);
        let input = IoTarget::File(dir.path().join("missing.bin"));

        assert!(run(args(input, output)).is_err());
        assert!(!path.exists(), "no output must be produced on failure");
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
