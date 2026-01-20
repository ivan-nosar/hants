use crate::constants;

pub struct RawArgs<'a> {
    pub command: &'a String,
    pub command_args: &'a [String],
}

pub fn parse_raw_args(args: &Vec<String>) -> Result<RawArgs, &'static str> {
    // First argument is always set and represents the name of executable/command the binary was called with.
    // Second argument is the command. All further arguments are command-specific parameters.
    let hants_args: &[String] = match &args[..] {
        [_, hants_args @ ..] => hants_args,
        [] => return Err(constants::strings::errors::CLI_ARGS_INVALID_MESSAGE),
    };

    match hants_args {
        [command, command_args @ ..] => Ok(RawArgs {
            command,
            command_args
        }),
        [] => Err(constants::strings::errors::COMMAND_NOT_PROVIDED_MESSAGE),
    }
}
