fn main() {
    if let Err(error) = hants::run() {
        eprintln!("Error: {error}");
        std::process::exit(1);
    }
}
