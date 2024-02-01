use crate::cli::Cli;
use crate::error::BError;
use crate::data::WsUploadData;
use crate::executers::{
    TaskExecuter,
};

use std::collections::HashMap;

pub struct UploadExecuter<'a> {
    cli: &'a Cli,
    data: &'a WsUploadData,
}

impl<'a> TaskExecuter for UploadExecuter<'a> {
    fn exec(&self, env_variables: &HashMap<String, String>, dry_run: bool, _interactive: bool) -> Result<(), BError> {
        let cmd: Vec<String> = self.data.deploy_cmd().split(' ').map(|c| c.to_string()).collect();

        if dry_run {
            self.cli.info("Dry run. Skipping deploy!".to_string());
            return Ok(());
        }

        self.cli.check_call(&cmd, env_variables, true)
    }
}

impl<'a> UploadExecuter<'a> {
    pub fn new(cli: &'a Cli, data: &'a WsUploadData) -> Self {
        UploadExecuter {
            cli,
            data,
        }
    }
}