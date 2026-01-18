use std::env;

pub mod cli;
pub mod commands;
pub mod constants;

fn main() {
    match cli::setup_logger() {
        Ok(_) => {},
        Err(error_message) => {
            log::error!("{}", error_message);
            return
        }
    }

    let args: Vec<String> = env::args().collect();

    // TODO: Consider returning Result from main?
    let raw_args: cli::RawArgs = match cli::parse_raw_args(&args) {
        Ok(raw_args) => raw_args,
        Err(error_message) => {
            log::error!("{}", error_message);
            return
        }
    };

    log::debug!("Running `hants` command: {}; Command args: {:?}", raw_args.command, raw_args.command_args);

    let command = match commands::command_resolver::resolve_command(&raw_args.command) {
        Ok(command) => command,
        Err(error_message) => {
            log::error!("{error_message}");
            return
        }
    };

    command.parse_args(raw_args.command_args);
    match command.execute() {
        Ok(_) => (),
        Err(error_message) => {
            log::error!("{error_message}");
        }
    }
}
