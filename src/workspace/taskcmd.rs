use crate::error::BError;
use crate::executers::{TaskCmdExecuter, TaskExecuter};
use crate::fs::JsonFileReader;
use crate::configs::Context;
use crate::data::WsTaskCmdData;
use crate::cli::Cli;

use serde_json::Value;
use std::collections::HashMap;

pub struct WsTaskCmdHandler {
    data: WsTaskCmdData,
}

impl WsTaskCmdHandler {
    pub fn from_str(name: &str, json_config: &str) -> Result<Self, BError> {
        let data: Value = JsonFileReader::parse(json_config)?;
        Self::new(name, &data)
    }

    pub fn new(name: &str, data: &Value) -> Result<Self, BError> {
        let taskcmd_data: WsTaskCmdData = WsTaskCmdData::from_value(name, data)?;

        Ok(WsTaskCmdHandler {
          data: taskcmd_data,
        })
    }

    pub fn expand_ctx(&mut self, ctx: &Context) -> Result<(), BError> {
        self.data.expand_ctx(ctx)?;
        Ok(())
    }

    pub fn run<'a>(&self, cli: &'a Cli, env_variables: &HashMap<String, String>, dry_run: bool, interactive: bool) -> Result<(), BError> {
        let executer: Box<dyn TaskExecuter> = Box::new(TaskCmdExecuter::new(cli, &self.data));
        executer.exec(env_variables, dry_run, interactive)
    }
}