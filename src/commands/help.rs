use colored::Colorize;
use log::info;
use crate::commands::base64::Base64;
use crate::constants;
use super::traits::Command;

pub struct Help();

impl Help {
    pub const NAME: &'static str = constants::strings::commands::HELP_COMMAND_NAME;

    const COMMAND_DESCRIPTION_PADDING: usize = 14;

    pub fn new() -> Self {
        Help {}
    }
}

impl Command for Help {

    fn execute(&self) -> Result<(), String> {
        // TODO: Try to reuse list of commands with `resolve_command` function

        let base64_command = Base64::new();

        println!("{}", self.get_help());
        println!("{}", base64_command.get_help());
        Ok(())

        // **Usage:**
        // hants [command] [options...]
        //
        // **Commands:**
        // help             Use `help` command to get the usage instructions.
        //                  _Example:_ hants help
        //
        // base64           Use `base64` command to encode or decode Base64 content.
        //                  _Example:_ hants base64 [action] [options...]
        // _Supported actions:_
        //      encode      Encode input sequence to Base64 format.
        //      decode      Decode input Base64 sequence.
        //      encode      Check if input string is a valid Base64 sequence.
        // _Options:_
        //      Input options. These are exclusive options and cannot be used simultaneously.
        //          -fi <filePath>, --file-input <filePath>     Read input sequence from file with specified path.
        //          -cbi, --clipboard-input                     Read input sequence from clipboard.
        //          -ci <string>, --console-input <string>      Specify input sequence directly in parameters list.
        //
        //      Output options. These are exclusive options and cannot be used simultaneously.
        //          -fo <filePath>, --file-output <filePath>    Write output to the file with specified path.
        //                                                      File must not exist prior to command execution.
        //          -cbo, --clipboard-output                    Write output to the clipboard.
        //          -co, --console-output                       Print output in the console. Default option.
        //
        //      Alphabet options.
        //          -ps <symbol>, --padding-symbol <symbol>             Use symbol provided as padding character.
        //                                                              Default: '='
        //          -cs <symbols>, --complementary-symbols <symbols>    Use symbols provided as a replacement for default
        //                                                              complementary symbols (63th and 64th character in
        //                                                              alphabet). Default: '+/'.
        //          -a <alphabet>, --alphabet <alphabet>                Use custom alphabet. Must be a string consisting
        //                                                              of exactly 64 unique symbols. Default:
        //                                                              'ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/'
    }

    fn parse_args(&self, _: &[String]) {
        log::debug!("{}", constants::strings::debug::HELP_COMMAND_PARSE_ARGS_MESSAGE);
    }

    fn get_help(&self) -> String {
        format!(
            "{command_name:<padding$}{command_description}\n{space:<padding$}{example}: {hants_name} {command_name}",
            command_name = constants::strings::commands::HELP_COMMAND_NAME.cyan().bold(),
            command_description = constants::strings::help::HELP_COMMAND_DESCRIPTION,
            space = " ",
            example = constants::strings::help::EXAMPLE.italic(),
            hants_name = constants::strings::help::HANTS_NAME.bright_white().bold(),
            padding = Help::COMMAND_DESCRIPTION_PADDING
        )
    }
}