use crate::constants;
use super::traits::Command;

// TODO: Extract to separate file
pub enum CommandInputOption {
    File(String),
    Clipboard,
    Console(String)
}

pub enum Base64Action {
    Encode,
    Decode,
    Validate,
}


pub struct Base64 {
    action: Base64Action,
    // input_option: CommandInputOption,
}

struct Base64ParsedArgs {
    action: Base64Action,
    // input_option: CommandInputOption,
}

impl Base64 {
    pub const NAME: &'static str = constants::strings::commands::BASE64_COMMAND_NAME;

    pub fn new(args: &[String]) -> Result<Self, String> {
        match Self::parse_args(args) {
            Ok(parsed_args) => Ok(Base64 {
                action: parsed_args.action,
                // input_option: parsed_args.input_option,
            }),
            Err(error_message) => Err(error_message),
        }
    }

    fn parse_args(args_list: &[String]) -> Result<Base64ParsedArgs, String> {
        // First argument should represent action.
        // All subsequent arguments represent options and have an undefined order.
        let (action_string, options): (&String, &[String]) = match &args_list[..] {
            [action_string, options @ ..] => (action_string, options),
            [] => return Err(
                String::from(constants::strings::errors::BASE64_ACTION_NOT_PROVIDED_MESSAGE)
            ),
        };

        // Parse action
        let action = match Self::parse_action_string(action_string) {
            Ok(action) => action,
            Err(error_message) => return Err(error_message),
        };

        // Parse options

        Ok(Base64ParsedArgs {
            action
        })
    }

    fn parse_action_string(action: &str) -> Result<Base64Action, String> {
        match action.to_lowercase().as_str() {
            "encode" => Ok(Base64Action::Encode),
            "decode" => Ok(Base64Action::Decode),
            "validate" => Ok(Base64Action::Validate),
            other => Err(
                format!("'{other}': {}", constants::strings::errors::BASE64_ACTION_NOT_SUPPORTED_MESSAGE)
            )
        }
    }
}

impl Command for Base64 {

    fn execute(&self) -> Result<(), String> {
        // println!("Running action {:?}", self.action);

        Ok(())
    }
}