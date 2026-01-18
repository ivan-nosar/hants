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
        // TODO
        Ok(())
    }

    fn parse_args(&self, _: &[String]) {
        log::debug!("{}", constants::strings::debug::HELP_COMMAND_PARSE_ARGS_MESSAGE);
    }

    fn get_help(&self) -> String {
        todo!()
    }
}