use serde_json::Value;

use crate::configs::Context;
use crate::error::BError;
use crate::configs::Config;

pub struct WsTaskCmdData {
    name: String,
    cmd: String,
    docker: String,
}

impl Config for WsTaskCmdData {
}

impl WsTaskCmdData {
    pub fn from_str(name: &str, json_string: &str) -> Result<Self, BError> {
      let data: Value = Self::parse(json_string)?;
      Self::from_value(name, &data)
    }

    pub fn from_value(name: &str, data: &Value) -> Result<Self, BError> {
        let mut task_data: &Value = data;
        match task_data.get(name) {
            Some(value) => {
                task_data = value;
            }
            None => {}
        }
        Self::new(name, task_data)
    }

    pub fn new(name: &str, data: &Value) -> Result<Self, BError> {
        let cmd: String = Self::get_str_value("cmd", data, Some(format!("echo \"INFO: currently no '{}' task defined\"", name)))?;
        let docker: String = Self::get_str_value("docker", data, Some(String::from("NA")))?;

        Ok(WsTaskCmdData {
            name: String::from(name),
            cmd,
            docker,
        })
    }

    pub fn expand_ctx(&mut self, ctx: &Context) -> Result<(), BError> {
        self.cmd = ctx.expand_str(&self.cmd)?;
        self.docker = ctx.expand_str(&self.docker)?;
        Ok(())
    }

    pub fn setup_cmd(&self) -> &String {
        &self.cmd
    }
}
