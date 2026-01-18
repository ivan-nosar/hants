use std::env;

pub mod constants;

fn main() {
    let args: Vec<String> = env::args().collect();

    let raw_args: RawArgs = match parse_raw_args(&args) {
        Ok(raw_args) => raw_args,
        Err(error_message) => {
            println!("Failure! Message: {error_message}");
            return
        }
    };

    println!("Command: {}; Command args: {:?}", raw_args.command, raw_args.command_args);
}

fn parse_raw_args(args: &Vec<String>) -> Result<RawArgs, &'static str> {
    // First argument is always set and represents the name of executable/command the binary was called with.
    // Second argument is the command. All further arguments are command-specific parameters.
    let hants_args: &[String] = &args[1..];
    match hants_args {
        [command, command_args @ ..] => Ok(RawArgs {
            command,
            command_args
        }),
        [] => Err(constants::strings::errors::COMMAND_NOT_PROVIDED_MESSAGE),
    }
}

struct RawArgs<'a> {
    command: &'a String,
    command_args: &'a [String],
}
