use crate::error::BError;
use crate::executers::{SubCmdExecuter, TaskExecuter};
use crate::fs::ConfigFileReader;
use crate::configs::Context;
use crate::data::WsSubCmdData;
use crate::cli::Cli;

use serde_json::Value;
use std::collections::HashMap;

pub struct WsSubCmdHandler {
    data: WsSubCmdData,
}

impl WsSubCmdHandler {
    pub fn from_str(name: &str, json_config: &str) -> Result<Self, BError> {
        let data: Value = ConfigFileReader::parse(json_config)?;
        Self::new(name, &data)
    }

    pub fn new(name: &str, data: &Value) -> Result<Self, BError> {
        let taskcmd_data: WsSubCmdData = WsSubCmdData::from_value(name, data)?;

        Ok(WsSubCmdHandler {
          data: taskcmd_data,
        })
    }

    pub fn expand_ctx(&mut self, ctx: &Context) -> Result<(), BError> {
        self.data.expand_ctx(ctx)?;
        Ok(())
    }

    pub fn run<'a>(&self, cli: &'a Cli, env_variables: &HashMap<String, String>, dry_run: bool, interactive: bool) -> Result<(), BError> {
        let executer: Box<dyn TaskExecuter> = Box::new(SubCmdExecuter::new(cli, &self.data));
        executer.exec(env_variables, dry_run, interactive)
    }

    pub fn data(&self) -> &WsSubCmdData {
        &self.data
    }
}

#[cfg(test)]
mod tests {
    use crate::workspace::WsSubCmdHandler;

    use std::collections::HashMap;
    use crate::cli::*;

    #[test]
    fn test_ws_deploy_handler() {
        let json_build_config = r#"
        {
            "cmd": "$#[SCRIPTS_DIR]/script.sh $#[ARG1] $#[ARG2] $#[ARG3]"
        }"#;
        let handler: WsSubCmdHandler = WsSubCmdHandler::from_str("deploy", json_build_config).expect("Failed to parse build config");
        let mut mocked_system: MockSystem = MockSystem::new();
        mocked_system
            .expect_check_call()
            .with(mockall::predicate::eq(CallParams {
                cmd_line: vec!["$#[SCRIPTS_DIR]/script.sh", "$#[ARG1]", "$#[ARG2]", "$#[ARG3]"]
                    .iter()
                    .map(|s| s.to_string())
                    .collect(),
                env: HashMap::new(),
                shell: true,
            }))
            .once()
            .returning(|_x| Ok(()));
        let cli: Cli = Cli::new(
            Box::new(BLogger::new()),
            Box::new(mocked_system),
            clap::Command::new("bakery"),
            Some(vec!["bakery"]),
        );
        handler.run(&cli, &HashMap::new(), false, true).expect("Failed to run handler");
    }
}