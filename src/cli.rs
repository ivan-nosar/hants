use std::fmt::Arguments;
use fern::{
    colors::{Color, ColoredLevelConfig},
    FormatCallback
};
use log::Record;
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

pub fn setup_logger<'a>() -> Result<(), &'a str> {
    // TODO: Allow specifying log level
    // TODO: Use different formats for debug and release build configurations.
    // TODO: Implement better output with `colored` crate. Default logging isn't quite suitable for good formatting.
    // TODO: Another option for implementation: `crossterm` or `ratatui` crates.
    let colors = ColoredLevelConfig::new()
        .trace(Color::White)
        .debug(Color::Blue)
        .info(Color::BrightWhite)
        .warn(Color::BrightYellow)
        .error(Color::BrightRed);

    let logger_init_result = fern::Dispatch::new()
        .format(move |out: FormatCallback, message: &Arguments, record: &Record| {
            out.finish(format_args!(
                "[{}] {}",
                colors.color(record.level()),
                message
            ))
        })
        .level(log::LevelFilter::Trace)
        .chain(std::io::stdout())
        .apply();

    match logger_init_result {
        Ok(_) => Ok(()),
        Err(_) => Err(constants::strings::errors::HANTS_INITIALIZATION_ERROR_MESSAGE)
    }
}
