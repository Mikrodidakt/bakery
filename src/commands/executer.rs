use std::env::Vars;

use crate::commands::BError;

pub struct Workspace {

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

    pub fn execute(&self, _cmd: &str, _env: Vars, _directory: &str, _docker_imag: &str, _interactive: bool) -> Result<(), BError> {
        //check_call(cmd);
        Ok(())
    }
}