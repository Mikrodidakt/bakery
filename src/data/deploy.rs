use serde_json::Value;

use crate::configs::Context;
use crate::error::BError;
use crate::configs::Config;

pub struct WsDeployData {
    cmd: String,
    docker: String,
}

impl Config for WsDeployData {
}

impl WsDeployData {
    pub fn from_str(json_string: &str) -> Result<Self, BError> {
      let data: Value = Self::parse(json_string)?;
      Self::from_value(&data)
    }

    pub fn from_value(data: &Value) -> Result<Self, BError> {
        let mut deploy_data: &Value = data;
        match deploy_data.get("deploy") {
            Some(value) => {
                deploy_data = value;
            }
            None => {}
        }
        Self::new(deploy_data)
    }

    pub fn new(data: &Value) -> Result<Self, BError> {
        let cmd: String = Self::get_str_value("cmd", data, Some(String::from("NA")))?;
        let docker: String = Self::get_str_value("docker", data, Some(String::from("NA")))?;

        Ok(WsDeployData {
            cmd,
            docker,
        })
    }

    pub fn expand_ctx(&mut self, ctx: &Context) {
        self.cmd = ctx.expand_str(&self.cmd);
        self.docker = ctx.expand_str(&self.docker);
    }

    pub fn deploy_cmd(&self) -> &String {
        &self.cmd
    }
}
