use crate::error::BError;
use crate::executers::{UploadExecuter, TaskExecuter};
use crate::fs::JsonFileReader;
use crate::configs::Context;
use crate::data::WsUploadData;
use crate::cli::Cli;

use serde_json::Value;
use std::collections::HashMap;

pub struct WsUploadHandler {
    data: WsUploadData,
}

impl WsUploadHandler {
    pub fn from_str(json_config: &str) -> Result<Self, BError> {
        let data: Value = JsonFileReader::parse(json_config)?;
        Self::new(&data)
    }

    pub fn new(data: &Value) -> Result<Self, BError> {
        let upload_data: WsUploadData = WsUploadData::from_value(data)?;

        Ok(WsUploadHandler {
          data: upload_data,
        })
    }

    pub fn expand_ctx(&mut self, ctx: &Context) -> Result<(), BError> {
        self.data.expand_ctx(ctx)?;
        Ok(())
    }

    pub fn run<'a>(&self, cli: &'a Cli, env_variables: &HashMap<String, String>, dry_run: bool, interactive: bool) -> Result<(), BError> {
        let executer: Box<dyn TaskExecuter> = Box::new(UploadExecuter::new(cli, &self.data));
        executer.exec(env_variables, dry_run, interactive)
    }
}