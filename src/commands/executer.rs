use std::env::Vars;

use crate::error::BError;
use crate::docker::{Docker, DockerImage};
use crate::workspace::Workspace;
use crate::logger::BLogger;

pub struct Executer<'a> {
    _workspace: &'a Workspace,
}

impl<'a> Executer<'a> {
    pub fn new(workspace: &'a Workspace) -> Self {
        Executer {
            _workspace: workspace,
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
                cmd_line = format!("cd {} && {}", self._workspace._work_dir, cmd_line);
                exec_dir = self._workspace._work_dir.clone();
            }
        }

        match docker {
            Some(docker) => {
                docker.run_cmd(cmd_line, exec_dir)?;
            },
            None => {
                BLogger::info(format_args!("Execute '{}'", cmd_line));
            }  
        }

        Ok(())
    }
}