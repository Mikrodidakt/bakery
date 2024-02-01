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

    pub fn expand_ctx(&mut self, ctx: &Context) -> Result<(), BError> {
        self.cmd = ctx.expand_str(&self.cmd)?;
        self.docker = ctx.expand_str(&self.docker)?;
        Ok(())
    }

    pub fn cmd(&self) -> &String {
        &self.cmd
    }
}

#[cfg(test)]
mod tests {
    use crate::data::WsDeployData;
    use crate::configs::Context;
    use indexmap::{IndexMap, indexmap};

    #[test]
    fn test_ws_deploy_data_default() {
        let json_build_config = r#"
        {
        }"#;
        let data: WsDeployData = WsDeployData::from_str(json_build_config).expect("Failed to parse config data");
        assert_eq!(data.cmd(), "NA");
    }

    #[test]
    fn test_ws_deploy_cmd() {
        let json_build_config = r#"
        {
            "cmd": "/path/to/deploy/script.sh arg1 arg2 arg3"
        }"#;
        let data: WsDeployData = WsDeployData::from_str(json_build_config).expect("Failed to parse config data");
        assert_eq!(data.cmd(), "/path/to/deploy/script.sh arg1 arg2 arg3");
    }

    #[test]
    fn test_ws_deploy_cmd_ctx() {
        let variables: IndexMap<String, String> = indexmap! {
            "ARG1".to_string() => "arg1".to_string(),
            "ARG2".to_string() => "arg2".to_string(),
            "ARG3".to_string() => "arg3".to_string(),
            "SCRIPTS_DIR".to_string() => "/path/to/deploy".to_string()
        };
        let ctx: Context = Context::new(&variables);
        let json_build_config = r#"
        {
            "cmd": "$#[SCRIPTS_DIR]/script.sh $#[ARG1] $#[ARG2] $#[ARG3]"
        }"#;
        let mut data: WsDeployData = WsDeployData::from_str(json_build_config).expect("Failed to parse config data");
        assert_eq!(data.cmd(), "$#[SCRIPTS_DIR]/script.sh $#[ARG1] $#[ARG2] $#[ARG3]");
        data.expand_ctx(&ctx).unwrap();
        assert_eq!(data.cmd(), "/path/to/deploy/script.sh arg1 arg2 arg3");
    }
}
