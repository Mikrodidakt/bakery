use serde_json::Value;

use crate::configs::Context;
use crate::error::BError;
use crate::configs::Config;

pub struct WsUploadData {
    cmd: String,
    docker: String,
}

impl Config for WsUploadData {
}

impl WsUploadData {
    pub fn from_str(json_string: &str) -> Result<Self, BError> {
      let data: Value = Self::parse(json_string)?;
      Self::from_value(&data)
    }

    pub fn from_value(data: &Value) -> Result<Self, BError> {
        let mut upload_data: &Value = data;
        match upload_data.get("upload") {
            Some(value) => {
                upload_data = value;
            }
            None => {}
        }
        Self::new(upload_data)
    }

    pub fn new(data: &Value) -> Result<Self, BError> {
        let cmd: String = Self::get_str_value("cmd", data, Some(String::from("NA")))?;
        let docker: String = Self::get_str_value("docker", data, Some(String::from("NA")))?;

        Ok(WsUploadData {
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
