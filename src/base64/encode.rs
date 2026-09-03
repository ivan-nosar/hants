use crate::io::{IoTarget, parse_input_option, parse_output_option};

#[derive(clap::Args)]
pub struct Args {
    #[arg(
        short = 'o',
        long = "output",
        help = "The output location for the command result. Supported values:\n\
        - c / console:      Print output of the command to the standard console output\n\
        - cb / clipboard:   Write output of the command to the system clipboard\n\
        - <file path>:      Write output of the command to the file with specified path.\n\
        \t\t      File must not exist prior to command execution\n",
        value_parser = parse_output_option,
        default_value = "clipboard")]
    output: IoTarget,

    #[arg(
        short = 'i',
        long = "input",
        help = "The target location for command to consume input from. Supported values:\n\
        - c / console:      Read input for the command from the standard console input\n\
        - cb / clipboard:   Read input for the command from the system clipboard\n\
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
        ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/",
    )]
    alphabet: Option<String>,

    #[arg(
        short = 'c',
        long = "complementary-symbols",
        conflicts_with = "alphabet",
        help = "Use symbols provided as a replacement for default complementary symbols \n\
        (63th and 64th character in alphabet: +/).",
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
    // if args.length == 0 {
    //     return Err("length must be greater than 0".to_string());
    // }
    //
    // let chars = get_sorted_alphabet_chars(args.symbol_classes)?;
    //
    // if chars.is_empty() {
    //     return Err("alphabet is empty".to_string());
    // }
    //
    // let password = generate_password(args.length, chars, args.seed);
    //
    // write_output(args.output, password)
    println!("Not yet implemented");
    Ok(())
}