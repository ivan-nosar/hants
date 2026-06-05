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
    if args.length == 0 {
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