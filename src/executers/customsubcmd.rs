use crate::cli::Cli;
use crate::error::BError;
use crate::data::WsCustomSubCmdData;
use crate::executers::TaskExecuter;

use std::collections::HashMap;

pub struct CustomSubCmdExecuter<'a> {
    cli: &'a Cli,
    data: &'a WsCustomSubCmdData,
}

impl<'a> TaskExecuter for CustomSubCmdExecuter<'a> {
    fn exec(&self, env_variables: &HashMap<String, String>, dry_run: bool, _interactive: bool) -> Result<(), BError> {
        let cmd: Vec<String> = self.data.cmd().split(' ').map(|c| c.to_string()).collect();

        if dry_run {
            self.cli.info("Dry run. Skipping deploy!".to_string());
            return Ok(());
        }

        self.cli.check_call(&cmd, env_variables, true)
    }
}

impl<'a> CustomSubCmdExecuter<'a> {
    pub fn new(cli: &'a Cli, data: &'a WsCustomSubCmdData) -> Self {
        CustomSubCmdExecuter {
            cli,
            data,
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::data::WsCustomSubCmdData;
    use crate::executers::{CustomSubCmdExecuter, TaskExecuter};

    use std::collections::HashMap;
    use crate::cli::*;

    #[test]
    fn test_ws_deploy_executer() {
        let json_build_config = r#"
        {
            "cmd": "$#[SCRIPTS_DIR]/script.sh $#[ARG1] $#[ARG2] $#[ARG3]"
        }"#;
        let data: WsCustomSubCmdData = WsCustomSubCmdData::from_str("deploy", json_build_config).expect("Failed to parse config data");
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
        let executer: CustomSubCmdExecuter = CustomSubCmdExecuter::new(&cli, &data);
        executer.exec(&HashMap::new(), false, true).expect("Failed to execute deploy");
    }
}