use crate::io::{IoTarget, parse_input_option, parse_output_option};
use clap::Subcommand;

pub mod decode;
pub mod encode;

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
        help = "Use custom alphabet. Must be a string consisting of exactly\n\
        64 unique symbols. If not provided - default alphabet is used:\n\
        ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/"
    )]
    alphabet: Option<String>,

    #[arg(
        short = 'c',
        long = "complementary-symbols",
        conflicts_with = "alphabet",
        allow_hyphen_values = true,
        help = "Use symbols provided as a replacement for default complementary symbols\n\
        (63th and 64th character in alphabet: +/)."
    )]
    complementary_symbols: Option<String>,

    #[arg(
        short = 'p',
        long = "padding-symbol",
        conflicts_with = "no_pad",
        help = "Use symbol provided as padding character.",
        default_value = "="
    )]
    padding_symbol: char,

    #[arg(
        short = 'n',
        long = "no-pad",
        conflicts_with = "padding_symbol",
        help = "Disable padding. When encoding, no trailing padding symbols\n\
        are emitted; when decoding, the input is expected to carry none.",
        action = clap::ArgAction::SetTrue
    )]
    no_pad: bool,
}

#[derive(Subcommand)]
pub enum Command {
    #[command(about = "Encode input sequence to Base64 format")]
    Encode(Args),

    #[command(about = "Decode input Base64 sequence")]
    Decode(Args),
}

pub fn run(command: Command) -> Result<(), String> {
    match command {
        Command::Encode(args) => encode::run(args),
        Command::Decode(args) => decode::run(args),
    }
}
