use crate::cli::Cli;
use crate::error::BError;
use crate::data::WsDeployData;
use crate::executers::{
    TaskExecuter,
    Docker,
    DockerImage,
};

use std::collections::HashMap;

pub struct DeployExecuter<'a> {
    cli: &'a Cli,
    data: &'a WsDeployData,
}

impl<'a> TaskExecuter for DeployExecuter<'a> {
    fn exec(&self, env_variables: &HashMap<String, String>, dry_run: bool, interactive: bool) -> Result<(), BError> {
        let cmd: Vec<String> = self.data.deploy_cmd().split(' ').map(|c| c.to_string()).collect();

        if dry_run {
            self.cli.info("Dry run. Skipping deploy!".to_string());
            return Ok(());
        }

        self.cli.check_call(&cmd, env_variables, true)
    }
}

impl<'a> DeployExecuter<'a> {
    pub fn new(cli: &'a Cli, data: &'a WsDeployData) -> Self {
        DeployExecuter {
            cli,
            data,
        }
    }
}