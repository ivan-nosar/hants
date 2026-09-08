use crate::base64::alphabet::{build_alphabet_mapping, validate_alphabet, validate_padding_symbol};
use crate::base64::engine::encode_with_alphabet;
use crate::io::{IoTarget, parse_input_option, parse_output_option, read_input, write_output};

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
        short = 'a',
        long = "alphabet",
        conflicts_with = "complementary_symbols",
        help = "Use custom alphabet. Must be a string consisting of exactly \n\
        64 unique symbols. If not provided - default alphabet is used: \n\
        ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/"
    )]
    alphabet: Option<String>,

    #[arg(
        short = 'c',
        long = "complementary-symbols",
        conflicts_with = "alphabet",
        help = "Use symbols provided as a replacement for default complementary symbols \n\
        (63th and 64th character in alphabet: +/)."
    )]
    complementary_symbols: Option<String>,

    #[arg(
        short = 'p',
        long = "padding-symbol",
        help = "Use symbol provided as padding character.",
        default_value = "="
    )]
    padding_symbol: char,
}

pub fn run(args: Args) -> Result<(), String> {
    let alphabet = match validate_alphabet(args.alphabet, args.complementary_symbols) {
        Err(e) => return Err(e),
        Ok(alphabet) => alphabet,
    };

    let padding_symbol = match validate_padding_symbol(args.padding_symbol, &alphabet) {
        Err(e) => return Err(e),
        Ok(padding_symbol) => padding_symbol,
    };

    let alphabet_mapping = build_alphabet_mapping(&alphabet);

    let input_data = match read_input(args.input) {
        Err(e) => return Err(e),
        Ok(data) => data,
    };

    let encoded_data = match encode_with_alphabet(input_data, alphabet_mapping, padding_symbol) {
        Err(e) => return Err(e),
        Ok(data) => data,
    };

    write_output(args.output, encoded_data)
}
