use crate::cli::Cli;
use crate::error::BError;
use crate::data::WsTaskCmdData;
use crate::executers::{
    TaskExecuter,
};

use std::collections::HashMap;

pub struct TaskCmdExecuter<'a> {
    cli: &'a Cli,
    data: &'a WsTaskCmdData,
}

impl<'a> TaskExecuter for TaskCmdExecuter<'a> {
    fn exec(&self, env_variables: &HashMap<String, String>, dry_run: bool, _interactive: bool) -> Result<(), BError> {
        let cmd: Vec<String> = self.data.setup_cmd().split(' ').map(|c| c.to_string()).collect();

        if dry_run {
            self.cli.info("Dry run. Skipping deploy!".to_string());
            return Ok(());
        }

        self.cli.check_call(&cmd, env_variables, true)
    }
}

impl<'a> TaskCmdExecuter<'a> {
    pub fn new(cli: &'a Cli, data: &'a WsTaskCmdData) -> Self {
        TaskCmdExecuter {
            cli,
            data,
        }
    }
}