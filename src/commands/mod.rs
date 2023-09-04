pub mod build;
pub mod clean;
pub mod tests;
pub mod handler;
pub mod executer;

use std::collections::HashMap;
use crate::error::BError;

// Bakery SubCommand
pub trait BCommand {
    fn execute(&self, cli_matches: &clap::ArgMatches) -> Result<(), BError>{
        println!("Execute command {}", cli_matches.subcommand_name().unwrap());
        Ok(())
    }

    // Return a clap sub-command containing the args
    // for the bakery command
    fn subcommand(&self) -> &clap::Command;

    fn cmd_str(&self) -> &str;
}

pub struct BBaseCommand {
    _cmd_str: String,
    _subcmd: clap::Command,
    _interactive: bool,
    //_env: Vars,
}

pub fn get_supported_cmds() -> HashMap<&'static str, Box<dyn BCommand>> {
    let mut supported_cmds: HashMap<&'static str, Box<dyn BCommand>> = HashMap::new();

    // Add supported commands to the HashMap
    supported_cmds.insert("build", Box::new(BuildCommand::new()));
    supported_cmds.insert("clean", Box::new(CleanCommand::new()));

    // Add more commands as needed

    supported_cmds
}

pub use build::BuildCommand;
pub use clean::CleanCommand;
pub use handler::CmdHandler;
pub use executer::Executer;