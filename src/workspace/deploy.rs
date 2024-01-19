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

#[cfg(test)]
mod tests {
    use crate::workspace::WsDeployHandler;

    use std::collections::HashMap;
    use crate::cli::*;

    #[test]
    fn test_ws_deploy_handler() {
        let json_build_config = r#"
        {
            "cmd": "${SCRIPTS_DIR}/script.sh ${ARG1} ${ARG2} ${ARG3}"
        }"#;
        let handler: WsDeployHandler = WsDeployHandler::from_str(json_build_config).expect("Failed to parse build config");
        let mut mocked_system: MockSystem = MockSystem::new();
        mocked_system
            .expect_check_call()
            .with(mockall::predicate::eq(CallParams {
                cmd_line: vec!["${SCRIPTS_DIR}/script.sh", "${ARG1}", "${ARG2}", "${ARG3}"]
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