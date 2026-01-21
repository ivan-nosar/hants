pub trait Command {
    fn execute(&self) -> Result<(), String>;
}
