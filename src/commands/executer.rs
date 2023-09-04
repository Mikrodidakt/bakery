use std::env::Vars;

use crate::commands::BError;

pub struct Workspace {
    pub _work_dir: String,
}

pub struct Executer {
    _workspace: Workspace,
}

impl Executer {
    pub fn new(workspace: Workspace) -> Self {
        Executer {
            _workspace: workspace,
        }
    }

    pub fn execute(&self, cmd: &str, _env: Vars, dir: Option<String>, docker_image: Option<String>, interactive: bool) -> Result<(), BError> {
        //check_call(cmd);
        let mut cmd_line: String = String::from(cmd);

        // If no directory is specified we should use the Workspace work directory
        // as the directory to execute the command from
        match dir {
            Some(directory) => {
                cmd_line = format!("cd {} && {}", directory, cmd_line);
            },
            None => {
                cmd_line = format!("cd {} && {}", self._workspace._work_dir, cmd_line);
            }
        }

        match docker_image {
            Some(image) => {
                if interactive {
                    println!("Execute inside docker image interactive=true'{} {}'", image, cmd_line);
                } else {
                    println!("Execute inside docker image '{} {}'", image, cmd_line);
                }
            },
            None => {
                println!("Execute '{}'", cmd_line);
            }  
        }

        Ok(())
    }
}