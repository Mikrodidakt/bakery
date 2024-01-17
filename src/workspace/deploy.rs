use crate::error::BError;
use crate::executers::{DeployExecuter, TaskExecuter};
use crate::fs::JsonFileReader;
use crate::configs::Context;
use crate::data::WsDeployData;
use crate::cli::Cli;

use serde_json::Value;
use std::collections::HashMap;

pub struct WsDeployHandler {
    data: WsDeployData,
}

impl WsDeployHandler {
    pub fn from_str(json_config: &str) -> Result<Self, BError> {
        let data: Value = JsonFileReader::parse(json_config)?;
        Self::new(&data)
    }

    pub fn new(data: &Value) -> Result<Self, BError> {
        let deploy_data: WsDeployData = WsDeployData::from_value(data)?;

        Ok(WsDeployHandler {
          data: deploy_data,
        })
    }

    pub fn expand_ctx(&mut self, ctx: &Context) {
        self.data.expand_ctx(ctx);
    }

    pub fn run<'a>(&self, cli: &'a Cli, env_variables: &HashMap<String, String>, dry_run: bool, interactive: bool) -> Result<(), BError> {
        let executer: Box<dyn TaskExecuter> = Box::new(DeployExecuter::new(cli, &self.data));
        executer.exec(env_variables, dry_run, interactive)
    }
}