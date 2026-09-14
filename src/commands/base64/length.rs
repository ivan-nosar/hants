use crate::base64::engine::calculate_encoded_length;
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
}

pub fn run(args: Args) -> Result<(), String> {
    // TODO: let to select which length to calculate: encoded or decoded
    let input_data = read_input_bytes(args.input)?;

    let encoded_data = calculate_encoded_length(input_data.len());

    write_output_string(args.output, encoded_data.to_string())
}
