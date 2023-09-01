use crate::commands::Command;

pub struct BuildCommand {
    _cmd_str: String,
    // Your struct fields and methods here
}

impl Command for BuildCommand {
    fn cmd_str(&self) -> &str {
        &self._cmd_str
    }
}

impl BuildCommand {
    pub fn new() -> Self {
        // Initialize and return a new BuildCommand instance
        BuildCommand {
            // Initialize fields if any
            _cmd_str: String::from("build"),
        }
    }
}