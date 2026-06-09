use clap::Subcommand;

#[derive(Subcommand, Debug)]
pub enum Command {
    #[command(about = "Encode input sequence to Base64 format")]
    Encode,

    #[command(about = "Decode input Base64 sequence")]
    Decode,

    #[command(about = "Check if input string is a valid Base64 sequence")]
    Validate,
}

pub fn run(command: Command) -> Result<(), String> {
    println!("base64 {:?}: not implemented yet", command);
    Ok(())
}
