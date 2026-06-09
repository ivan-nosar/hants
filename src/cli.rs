use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(
    version,
    name = "hants",
    about = "HANdy ToolSet - A lightweight command-line interface utility that consolidates \
    several small tools to streamline everyday development tasks."
)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Command,
}

#[derive(Subcommand)]
pub enum Command {
    #[command(about = "Generate a secure password")]
    Password(crate::commands::password::Args),

    #[command(subcommand, about = "Encode/decode/validate Base64 content")]
    Base64(crate::commands::base64::Command),
    // TODO: Not yet implemented
    // Json(crate::commands::json::Args),
    // Jwt(crate::commands::jwt::Args),
}
