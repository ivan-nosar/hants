use super::base64::Base64;
use super::help::Help;
use super::traits::Command;
use crate::constants;

pub fn resolve_command(command_name: &str) -> Result<Box<dyn Command>, String> {
    match command_name.to_lowercase().as_str() {
        Help::NAME => Ok(Box::new(Help::new())),
        Base64::NAME => Ok(Box::new(Base64::new())),
        other => Err(
            format!("'{other}': {}", constants::strings::errors::COMMAND_IS_NOT_SUPPORTED_MESSAGE)
        )
    }
}
