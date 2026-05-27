use std::collections::HashMap;
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



pub enum ArgType {
    Flag,
    AssignedValue,
}

// pub struct ArgRegistration<'a> {
//     name: &'a str,
//     arg_type: ArgType,
// }

pub fn parse_registered_args(args: &[String], registered_args: HashMap<String, ArgType>)
    -> Result<HashMap<&String, String>, String> {
    // Args missed in `args` will be silently skipped; Caller should decide what to do with it.
    // Unregistered args that were present in `args` will be skipped as well.

    let mut result: HashMap<&String, String> = HashMap::new();

    let mut index = 0;
    while index < args.len() {
        let item = &args[index];

        match registered_args.get(item) {
            Some(&ArgType::Flag) => {
                result.insert(item, String::from("true"));
            }
            Some(&ArgType::AssignedValue) => {
                if index == args.len() - 1 {
                    return Err(
                        format!("'{item}': {}", constants::strings::errors::COMMAND_NOT_PROVIDED_MESSAGE)
                    );
                }

                let arg_value = &args[index + 1];
                result.insert(item, arg_value.clone());
            }
            None => continue
        }

        index += 1;
    }

    Ok(result)
}
