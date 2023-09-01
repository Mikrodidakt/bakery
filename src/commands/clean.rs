use crate::commands::Command;

pub struct CleanCommand {
    _cmd_str: String,
    // Your struct fields and methods here
}

impl Command for CleanCommand {
    fn cmd_str(&self) -> &str {
        &self._cmd_str
    }
}

impl CleanCommand {
    pub fn new() -> Self {
        // Initialize and return a new BuildCommand instance
        CleanCommand {
            // Initialize fields if any
            _cmd_str: String::from("clean")
        }
    }
}