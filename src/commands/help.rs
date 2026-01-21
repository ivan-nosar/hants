use crate::constants;
use super::traits::Command;

pub struct Help();

impl Help {
    pub const NAME: &'static str = constants::strings::commands::HELP_COMMAND_NAME;

    pub fn new() -> Self {
        Help {}
    }
}

impl Command for Help {

    fn execute(&self) -> Result<(), String> {
        println!("{}", constants::strings::help::HANTS_HELP_TEXT);

        Ok(())
    }
}