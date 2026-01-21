use crate::cli::RawArgs;
use super::base64::Base64;
use super::help::Help;
use super::traits::Command;
use crate::constants;

pub fn resolve_command(raw_args: &RawArgs) -> Result<Box<dyn Command>, String> {
    match raw_args.command.to_lowercase().as_str() {
        Help::NAME => Ok(Box::new(Help::new())),
        Base64::NAME => {
            match Base64::new(raw_args.command_args) {
                Ok(base64) => Ok(Box::new(base64)),
                Err(error_message) => Err(error_message),
            }
        },
        other => Err(
            format!("'{other}': {}", constants::strings::errors::COMMAND_IS_NOT_SUPPORTED_MESSAGE)
        )
    }
}
