pub mod build;
pub mod clean;
pub mod list;
pub mod tests;
pub mod handler;
pub mod shell;

use std::collections::HashMap;

use crate::error::BError;
use crate::cli::Cli;
use crate::workspace::Workspace;
use crate::executers::docker::{Docker, DockerImage};

// Bakery SubCommand
pub trait BCommand {
    fn execute(&self, cli: &Cli, _workspace: &mut Workspace) -> Result<(), BError> {
        cli.info(format!("Execute command {}", self.cmd_str()));
        Ok(())
    }

    fn is_docker_required(&self) -> bool {
        false
    }

    fn bootstrap(&self, cli: &Cli, workspace: &Workspace, volumes: &Vec<String>, interactive: bool) -> Result<(), BError> {
        let mut docker: Docker = Docker::new(workspace.settings().docker_image(), interactive);
            
        if workspace.config().build_data().bitbake().docker_image() != "NA" {
            docker = Docker::new(DockerImage::new(workspace.config().build_data().bitbake().docker_image()), interactive);
        }

        cli.info(format!("Bootstrap bakery into docker"));
        return docker.bootstrap_bakery(cli, &workspace.settings().docker_top_dir(), &workspace.settings().work_dir(), workspace.settings().docker_args(), volumes);
    }

    fn get_config_name(&self, cli: &Cli) -> String {
        String::from("default")
    }
    
    fn get_arg_str(&self, cli: &Cli, id: &str, cmd: &str) -> Result<String, BError> {
        if let Some(sub_matches) = cli.get_args().subcommand_matches(cmd) {
            if sub_matches.contains_id(id) {
                if let Some(value) = sub_matches.get_one::<String>(id) {
                    return Ok(value.clone());
                }
            }
        }
        return Err(BError::CliError(format!("Failed to read arg {}", id)));
    }

    fn get_arg_flag(&self, cli: &Cli, id: &str, cmd: &str) -> Result<bool, BError> {
        if let Some(sub_matches) = cli.get_args().subcommand_matches(cmd) {
            if sub_matches.contains_id(id) {
                let flag: bool = sub_matches.get_flag(id);
                return Ok(flag);
            }
        }
        return Err(BError::CliError(format!("Failed to read arg {}", id)));
    }

    fn get_arg_many<'a>(&'a self, cli: &'a Cli, id: &str, cmd: &str) -> Result<Vec<String>, BError> {
        if let Some(sub_matches) = cli.get_args().subcommand_matches(cmd) {
            if sub_matches.contains_id(id) {
                let many: Vec<String> = sub_matches.get_many::<String>(id).unwrap_or_default().collect::<Vec<_>>().iter().map(|s| s.to_string()).collect();
                return Ok(many);
            }
            return Ok(Vec::new());
        }
        return Err(BError::CliError(format!("Failed to read arg {}", id)));
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
    supported_cmds.insert("shell", Box::new(ShellCommand::new()));

    // Add more commands as needed

    supported_cmds
}

pub use build::BuildCommand;
pub use clean::CleanCommand;
pub use list::ListCommand;
pub use shell::ShellCommand;
pub use handler::CmdHandler;