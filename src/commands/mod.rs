pub mod build;
pub mod clean;
pub mod tests;
pub mod handler;

use std::collections::HashMap;

pub trait Command {
    fn execute(&self) {
        println!("Execute command {}", self.cmd_str())
    }

    fn cmd_str(&self) -> &str;
}

pub fn get_supported_cmds() -> HashMap<&'static str, Box<dyn Command>> {
    let mut supported_cmds: HashMap<&'static str, Box<dyn Command>> = HashMap::new();

    // Add supported commands to the HashMap
    supported_cmds.insert("build", Box::new(BuildCommand::new()));
    supported_cmds.insert("clean", Box::new(CleanCommand::new()));

    // Add more commands as needed

    supported_cmds
}

pub use build::BuildCommand;
pub use clean::CleanCommand;
pub use handler::CmdHandler;