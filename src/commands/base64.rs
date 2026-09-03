use clap::Subcommand;
use crate::base64::{encode};

#[derive(Subcommand)]
pub enum Command {
    #[command(about = "Encode input sequence to Base64 format")]
    Encode(encode::Args),

    #[command(about = "Decode input Base64 sequence")]
    Decode,

    #[command(about = "Check if input string is a valid Base64 sequence")]
    Validate,
}

pub fn run(command: Command) -> Result<(), String> {
    match command {
        Command::Encode(args) => encode::run(args),
        _ => {
            println!("Not implemented yet");
            Ok(())
        }
        // Command::Decode(args) => base64::run(args),
        // Command::Validate(args) => base64::run(args)
    }
}
