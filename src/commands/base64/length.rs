use crate::base64::engine::{calculate_decoded_length, calculate_encoded_length};
use crate::io::{
    IoTarget, parse_input_option, parse_output_option, read_input_bytes, write_output_string,
};

#[derive(clap::Args)]
pub struct Args {
    #[arg(
        short = 'o',
        long = "output",
        help = "The output location for the command result. Supported values:\n\
        - c / console:      Print output of the command to the standard console output;\n\
        - cb / clipboard:   Write output of the command to the system clipboard;\n\
        - <file path>:      Write output of the command to the file with specified path.\n\
        \t\t      File must not exist prior to command execution\n",
        value_parser = parse_output_option,
        default_value = "clipboard")]
    output: IoTarget,

    #[arg(
        short = 'i',
        long = "input",
        help = "The target location for command to consume input from. Supported values:\n\
        - c / console:      Read input for the command from the stdin. Most suitable for using with pipes;\n\
        - cb / clipboard:   Read input for the command from the system clipboard;\n\
        - <file path>:      Read input for the command from the file with specified path.\n\
        \t\t      File must exist prior to command execution\n",
        value_parser = parse_input_option,
        default_value = "clipboard")]
    input: IoTarget,

    #[arg(
        short = 'm',
        long = "mode",
        help = "A command mode that allows to calculate either the length of an encoded or decoded message.\n\
        Supported values:\n\
        - e / encoded:  Calculate the length of input sequence if it would be encoded as Base64.\n\
        - d / decoded:   Calculate the length of input sequence if it would be decoded from Base64 format.",
        value_parser = parse_mode,
    )]
    mode: Mode,
}

#[derive(Clone)]
pub enum Mode {
    Encoded,
    Decoded,
}

pub fn run(args: Args) -> Result<(), String> {
    let input_data = read_input_bytes(args.input)?;

    let calculated_length = match args.mode {
        Mode::Encoded => calculate_encoded_length(input_data.len()),
        Mode::Decoded => calculate_decoded_length(input_data.len()),
    };

    write_output_string(args.output, calculated_length.to_string())
}

fn parse_mode(s: &str) -> Result<Mode, String> {
    match s.to_lowercase().as_str() {
        "e" | "encoded" => Ok(Mode::Encoded),
        "d" | "decoded" => Ok(Mode::Decoded),
        other => Err(format!(
            "unexpected mode option '{}' found. Supported options are 'e', 'encoded', 'd', or 'decoded'.",
            other
        )),
    }
}

#[cfg(test)]
mod tests {
    use super::{Args, Mode, parse_mode, run};
    use crate::base64::alphabet::{DEFAULT_ALPHABET, build_encoding_alphabet_mapping};
    use crate::base64::engine::encode_with_alphabet;
    use crate::io::IoTarget;
    use std::fs;
    use std::path::{Path, PathBuf};
    use tempfile::{TempDir, tempdir};

    fn input_target(dir: &TempDir, bytes: &[u8]) -> IoTarget {
        named_input_target(dir, "input.bin", bytes)
    }

    fn named_input_target(dir: &TempDir, name: &str, bytes: &[u8]) -> IoTarget {
        let path = dir.path().join(name);
        fs::write(&path, bytes).unwrap();
        IoTarget::File(path)
    }

    fn output_target(dir: &TempDir) -> (IoTarget, PathBuf) {
        named_output_target(dir, "output.txt")
    }

    fn named_output_target(dir: &TempDir, name: &str) -> (IoTarget, PathBuf) {
        let path = dir.path().join(name);
        (IoTarget::File(path.clone()), path)
    }

    fn args(input: IoTarget, output: IoTarget, mode: Mode) -> Args {
        Args {
            output,
            input,
            mode,
        }
    }

    fn run_for(dir: &TempDir, name: &str, payload: &[u8], mode: Mode) -> String {
        let input = named_input_target(dir, &format!("{}.in", name), payload);
        let (output, path) = named_output_target(dir, &format!("{}.out", name));

        run(args(input, output, mode)).unwrap();

        read_output(&path)
    }

    fn read_output(path: &Path) -> String {
        fs::read_to_string(path).unwrap()
    }

    #[test]
    fn calculates_encoded_length_for_empty_input() {
        let dir = tempdir().unwrap();

        assert_eq!(run_for(&dir, "empty", b"", Mode::Encoded), "0");
    }

    #[test]
    fn calculates_encoded_length_for_full_chunks() {
        let dir = tempdir().unwrap();

        assert_eq!(run_for(&dir, "one-chunk", b"foo", Mode::Encoded), "4");
        assert_eq!(run_for(&dir, "two-chunks", b"foobar", Mode::Encoded), "8");
    }

    #[test]
    fn calculates_encoded_length_for_incomplete_tail() {
        let dir = tempdir().unwrap();

        assert_eq!(run_for(&dir, "tail-one", b"f", Mode::Encoded), "4");
        assert_eq!(run_for(&dir, "tail-two", b"fo", Mode::Encoded), "4");
        assert_eq!(run_for(&dir, "chunk-tail-one", b"foob", Mode::Encoded), "8");
        assert_eq!(
            run_for(&dir, "chunk-tail-two", b"fooba", Mode::Encoded),
            "8"
        );
    }

    #[test]
    fn calculates_encoded_length_for_binary_input() {
        let dir = tempdir().unwrap();
        let payload = [0x14, 0xfb, 0x9c, 0x03, 0xd9, 0x7e, 0x00];

        assert_eq!(run_for(&dir, "binary", &payload, Mode::Encoded), "12");
    }

    #[test]
    fn calculated_encoded_length_matches_encoder_output() {
        let dir = tempdir().unwrap();
        let alphabet_mapping = build_encoding_alphabet_mapping(DEFAULT_ALPHABET);

        for (index, payload) in [
            &b""[..],
            b"f",
            b"fo",
            b"foo",
            b"foob",
            b"fooba",
            b"foobar",
            b"the quick brown fox",
        ]
        .into_iter()
        .enumerate()
        {
            let expected = encode_with_alphabet(payload, alphabet_mapping, b'=').len();
            let reported = run_for(&dir, &format!("match-{}", index), payload, Mode::Encoded);

            assert_eq!(reported, expected.to_string(), "payload: {:?}", payload);
        }
    }

    #[test]
    fn calculates_decoded_length_for_empty_input() {
        let dir = tempdir().unwrap();

        assert_eq!(run_for(&dir, "empty", b"", Mode::Decoded), "0");
    }

    #[test]
    fn calculates_decoded_length_for_full_chunks() {
        let dir = tempdir().unwrap();

        assert_eq!(run_for(&dir, "one-chunk", b"Zm9v", Mode::Decoded), "3");
        assert_eq!(run_for(&dir, "two-chunks", b"Zm9vYmFy", Mode::Decoded), "6");
    }

    #[test]
    fn calculates_decoded_length_as_upper_bound_for_padded_input() {
        // Documented best-effort behaviour: padding symbols are counted as payload,
        // so the reported value is up to 2 bytes larger than the real decoded length.
        let dir = tempdir().unwrap();

        assert_eq!(run_for(&dir, "one-pad", b"Zm9vYg==", Mode::Decoded), "6");
        assert_eq!(run_for(&dir, "two-pads", b"Zg==", Mode::Decoded), "3");
    }

    #[test]
    fn calculates_decoded_length_for_unpadded_input() {
        // Unpadded (and therefore incomplete) chunks are dropped by the integer division.
        let dir = tempdir().unwrap();

        assert_eq!(run_for(&dir, "unpadded-two", b"Zg", Mode::Decoded), "0");
        assert_eq!(run_for(&dir, "unpadded-three", b"Zm9", Mode::Decoded), "0");
        assert_eq!(run_for(&dir, "unpadded-six", b"Zm9vYg", Mode::Decoded), "3");
    }

    #[test]
    fn counts_whitespaces_of_decoded_input_as_payload() {
        // Whitespaces are ignored by the decoder, but not by the length calculation.
        let dir = tempdir().unwrap();

        assert_eq!(
            run_for(&dir, "trailing-newline", b"Zm9vYmFy\n", Mode::Decoded),
            "6"
        );
        assert_eq!(
            run_for(&dir, "surrounded", b"  Zm9vYmFy  ", Mode::Decoded),
            "9"
        );
    }

    #[test]
    fn writes_result_to_console_target() {
        let dir = tempdir().unwrap();
        let input = input_target(&dir, b"foobar");

        run(args(input, IoTarget::Console, Mode::Encoded)).unwrap();
    }

    #[test]
    fn returns_err_when_input_file_is_missing() {
        let dir = tempdir().unwrap();
        let (output, path) = output_target(&dir);
        let input = IoTarget::File(dir.path().join("missing.bin"));

        assert!(run(args(input, output, Mode::Encoded)).is_err());
        assert!(!path.exists(), "no output must be produced on failure");
    }

    #[test]
    fn returns_err_when_output_file_already_exists() {
        let dir = tempdir().unwrap();
        let input = input_target(&dir, b"foobar");
        let (output, path) = output_target(&dir);
        fs::write(&path, "existing").unwrap();

        assert!(run(args(input, output, Mode::Decoded)).is_err());
        assert_eq!(read_output(&path), "existing");
    }

    #[test]
    fn parses_encoded_mode() {
        for value in ["e", "E", "encoded", "Encoded", "ENCODED"] {
            assert!(
                matches!(parse_mode(value), Ok(Mode::Encoded)),
                "value: {}",
                value
            );
        }
    }

    #[test]
    fn parses_decoded_mode() {
        for value in ["d", "D", "decoded", "Decoded", "DECODED"] {
            assert!(
                matches!(parse_mode(value), Ok(Mode::Decoded)),
                "value: {}",
                value
            );
        }
    }

    #[test]
    fn returns_err_on_unknown_mode() {
        for value in ["", " ", "en", "encode", "decode", "x"] {
            let error = parse_mode(value)
                .err()
                .unwrap_or_else(|| panic!("value '{}' must not be parsed as a valid mode", value));

            assert_eq!(
                error,
                format!(
                    "unexpected mode option '{}' found. Supported options are 'e', 'encoded', 'd', or 'decoded'.",
                    value.to_lowercase()
                )
            );
        }
    }
}
