use crate::constants;
use super::traits::Command;

pub struct Base64 {

}

impl Base64 {
    pub const NAME: &'static str = constants::strings::commands::BASE64_COMMAND_NAME;

    pub fn new() -> Self {
        Base64 {}
    }
}

impl Command for Base64 {

    fn execute(&self) -> Result<(), String> {
        todo!()
    }

    fn parse_args(&self, args_map: &[String]) {
        todo!()
    }
}