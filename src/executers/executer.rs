use std::env::Vars;

use crate::error::BError;
use crate::executers::Docker;
use crate::workspace::Workspace;
use crate::cli::Cli;

pub struct Executer<'a> {
    workspace: &'a Workspace,
    cli: &'a Cli,
}

impl<'a> Executer<'a> {
    pub fn new(workspace: &'a Workspace, cli: &'a Cli) -> Self {
        Executer {
            workspace: workspace,
            cli: cli,
        }
    }

    pub fn execute(&self, cmd: &str, _env: Vars, dir: Option<String>, docker: Option<Docker>, interactive: bool) -> Result<(), BError> {
        let mut cmd_line: String = String::from(cmd);
        let exec_dir: String;

        // If no directory is specified we should use the Workspace work directory
        // as the directory to execute the command from
        match dir {
            Some(directory) => {
                cmd_line = format!("cd {} && {}", directory, cmd_line);
                exec_dir = directory;
            },
            None => {
                cmd_line = format!("cd {} && {}", self.workspace.settings().work_dir().to_str().unwrap(), cmd_line);
                exec_dir = self.workspace.settings().work_dir().to_str().unwrap().to_string();
            }
        }

        match docker {
            Some(docker) => {
                docker.run_cmd(cmd_line, exec_dir, &self.cli)?;
            },
            None => {
                self.cli.info(format!("Execute '{}'", cmd_line));
            }  
        }

        Ok(())
    }
}