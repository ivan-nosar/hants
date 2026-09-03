extern crate core;

use clap::Parser;

pub mod base64;
pub mod cli;
pub mod commands;
pub mod io;
pub mod password;

pub fn run() -> Result<(), String> {
    let cli = cli::Cli::parse();
    commands::run(cli.command)
}
 