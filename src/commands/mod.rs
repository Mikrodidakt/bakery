pub mod build;
pub mod clean;
pub mod list;
pub mod tests;
pub mod handler;

use std::collections::HashMap;

use crate::error::BError;
use crate::cli::Cli;
use crate::workspace::Workspace;

// Bakery SubCommand
pub trait BCommand {
    fn execute(&self, cli: &Cli, workspace: &Workspace) -> Result<(), BError> {
        cli.info(format!("Execute command {}", self.cmd_str()));
        Ok(())
    }

    fn is_docker_required(&self) -> bool {
        false
    }

    // Return a clap sub-command containing the args
    // for the bakery command
    fn subcommand(&self) -> &clap::Command;

    fn cmd_str(&self) -> &str;
}

pub struct BBaseCommand {
    cmd_str: String,
    sub_cmd: clap::Command,
    interactive: bool,
    require_docker: bool,
    //_env: Vars,
}

pub fn get_supported_cmds() -> HashMap<&'static str, Box<dyn BCommand>> {
    let mut supported_cmds: HashMap<&'static str, Box<dyn BCommand>> = HashMap::new();

    // Add supported commands to the HashMap
    supported_cmds.insert("build", Box::new(BuildCommand::new()));
    supported_cmds.insert("clean", Box::new(CleanCommand::new()));
    supported_cmds.insert("list", Box::new(ListCommand::new()));

    // Add more commands as needed

    supported_cmds
}

pub use build::BuildCommand;
use clap::ArgMatches;
pub use clean::CleanCommand;
pub use list::ListCommand;
pub use handler::CmdHandler;