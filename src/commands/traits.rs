pub trait Command {
    fn execute(&self) -> Result<(), String>;

    fn parse_args(&self, args_map: &[String]);
}
