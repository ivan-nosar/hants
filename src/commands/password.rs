use crate::io::output::{parse_output_option, write_output};
use crate::io::output::OutputTarget;
use crate::password::alphabet::get_sorted_alphabet_chars;
use crate::password::generator::generate_password;

#[derive(clap::Args)]
pub struct Args {
    #[arg(
        short = 'l',
        long = "length",
        default_value_t = 12,
        allow_negative_numbers = false,
        help = "The length of the password")]
    length: usize,

    #[arg(
        short = 'a',
        long = "symbol_classes",
        help = "The symbol classes for the password construction. Supported values:\n\
        - a: Lower-case alphabetic latin symbols\n\
        - A: Upper-case alphabetic latin symbols\n\
        - n: Digits\n\
        - b: Braces: ()<>[]{}\n\
        - q: Quotes: '\"`\n\
        - p: Punctuation: !?.,;:\n\
        - m: Math operations: +-*/=\n\
        - w: Whitespace symbols: space, \\t\\n\\r\n\
        - s: Special symbols: \\^~@$&%_\n",
        default_value = "aAnbqpms"
    )]
    symbol_classes: String,

    #[arg(short = 's', long = "seed", help = "The seed for the random values generator")]
    seed: Option<u64>,

    #[arg(
        short = 'o',
        long = "output",
        help = "The output location for the command result. Supported values:\n\
        - c / console:      Print output of the command to the standard console output\n\
        - cb / clipboard:   Write output of the command to the system clipboard\n\
        - <file path>:      Write output  of the command to the file with specified path.\n\
        \t\t      File must not exist prior to command execution\n",
        value_parser = parse_output_option,
        default_value = "clipboard")]
    output: OutputTarget,
}

pub fn run(args: Args) -> Result<(), String> {
    if args.length <= 0 {
        return Err("length must be greater than 0".to_string());
    }

    let chars = match get_sorted_alphabet_chars(args.symbol_classes) {
        Ok(chars) => chars,
        Err(e) => return Err(e),
    };

    if chars.is_empty() {
        return Err("alphabet is empty".to_string());
    }

    let password = generate_password(args.length, chars, args.seed);

    write_output(args.output, password)
}

#[cfg(test)]
mod tests {
    use super::{run, Args};
    use crate::io::output::OutputTarget;
    use std::collections::HashSet;
    use tempfile::tempdir;

    /// Characters allowed by the "aAn" classes (latin letters and digits).
    fn alphanumeric_set() -> HashSet<char> {
        ('a'..='z').chain('A'..='Z').chain('0'..='9').collect()
    }

    #[test]
    fn returns_err_when_length_is_zero() {
        let args = Args {
            length: 0,
            symbol_classes: "aAn".to_string(),
            seed: None,
            output: OutputTarget::Console,
        };

        assert!(run(args).is_err());
    }

    #[test]
    fn returns_err_when_symbol_classes_is_empty() {
        let args = Args {
            length: 12,
            symbol_classes: String::new(),
            seed: None,
            output: OutputTarget::Console,
        };

        assert!(run(args).is_err());
    }

    #[test]
    fn returns_err_when_symbol_classes_contains_unknown_class() {
        for classes in ["Z", "aZ", "aAnX", "1"] {
            let args = Args {
                length: 12,
                symbol_classes: classes.to_string(),
                seed: None,
                output: OutputTarget::Console,
            };

            assert!(run(args).is_err(), "expected Err for classes '{classes}'");
        }
    }

    #[test]
    fn same_seed_produces_same_password() {
        let dir = tempdir().unwrap();
        let first_path = dir.path().join("first.txt");
        let second_path = dir.path().join("second.txt");

        run(Args {
            length: 24,
            symbol_classes: "aAn".to_string(),
            seed: Some(777),
            output: OutputTarget::File(first_path.clone()),
        })
        .unwrap();
        run(Args {
            length: 24,
            symbol_classes: "aAn".to_string(),
            seed: Some(777),
            output: OutputTarget::File(second_path.clone()),
        })
        .unwrap();

        let first = std::fs::read_to_string(first_path).unwrap();
        let second = std::fs::read_to_string(second_path).unwrap();
        assert_eq!(first, second);
    }

    #[test]
    fn produces_password_of_requested_length_using_only_requested_classes() {
        let dir = tempdir().unwrap();
        let path = dir.path().join("password.txt");
        let length: usize = 64;

        run(Args {
            length,
            symbol_classes: "aAn".to_string(),
            seed: Some(2_024),
            output: OutputTarget::File(path.clone()),
        })
        .unwrap();

        let password = std::fs::read_to_string(path).unwrap();
        let allowed = alphanumeric_set();

        assert_eq!(password.chars().count(), length);
        for ch in password.chars() {
            assert!(
                allowed.contains(&ch),
                "unexpected char {ch:?} not in requested classes"
            );
        }
    }
}