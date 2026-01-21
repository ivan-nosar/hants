use std::env;

pub mod cli;
pub mod commands;
pub mod constants;

fn main() {
    let args: Vec<String> = env::args().collect();

    // TODO: Consider returning Result from main?
    let raw_args: cli::RawArgs = match cli::parse_raw_args(&args) {
        Ok(raw_args) => raw_args,
        Err(error_message) => {
            eprintln!("{}", error_message);
            return
        }
    };

    dbg!(raw_args.command, raw_args.command_args);

    let command = match commands::registered_commands::resolve_command(&raw_args) {
        Ok(command) => command,
        Err(error_message) => {
            eprintln!("{error_message}");
            return
        }
    };

    match command.execute() {
        Ok(_) => (),
        Err(error_message) => {
            eprintln!("{error_message}");
        }
    }
}
