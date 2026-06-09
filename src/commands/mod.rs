use crate::cli::Command;

pub mod base64;
pub mod password;

pub fn run(command: Command) -> Result<(), String> {
    match command {
        Command::Password(args) => password::run(args),
        Command::Base64(args) => base64::run(args),
        // TODO: Not yet implemented
        // Command::Json(args) => json::run(args),
        // Command::Jwt(args) => jwt::run(args),
    }
}
