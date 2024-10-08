use indexmap::{indexmap, IndexMap};
use std::collections::HashMap;

use crate::cli::Cli;
use crate::commands::{BBaseCommand, BCommand, BError};
use crate::data::{WsContextData, CTX_KEY_DEVICE, CTX_KEY_IMAGE};
use crate::workspace::{Workspace, WsCustomSubCmdHandler};

static BCOMMAND: &str = "deploy";
static BCOMMAND_ABOUT: &str = "Deploy artifacts to the target.";
pub struct DeployCommand {
    cmd: BBaseCommand,
    // Your struct fields and methods here
}

impl BCommand for DeployCommand {
    fn get_config_name(&self, cli: &Cli) -> String {
        if let Some(sub_matches) = cli.get_args().subcommand_matches(BCOMMAND) {
            if sub_matches.contains_id("config") {
                if let Some(value) = sub_matches.get_one::<String>("config") {
                    return value.clone();
                }
            }
        }

        return String::from("default");
    }

    fn cmd_str(&self) -> &str {
        &self.cmd.cmd_str
    }

    fn subcommand(&self) -> &clap::Command {
        &self.cmd.sub_cmd
    }

    fn is_docker_required(&self) -> bool {
        self.cmd.require_docker
    }

    fn execute(&self, cli: &Cli, workspace: &mut Workspace) -> Result<(), BError> {
        let config: String = self.get_arg_str(cli, "config", BCOMMAND)?;
        let ctx: Vec<String> = self.get_arg_many(cli, "ctx", BCOMMAND)?;
        let device: String = self.get_arg_str(cli, "device", BCOMMAND)?;
        let image: String = self.get_arg_str(cli, "image", BCOMMAND)?;
        let args_context: IndexMap<String, String> = self.setup_context(ctx);
        let mut context: WsContextData = WsContextData::new(&args_context)?;

        if device != String::from("NA") {
            context.update(&indexmap! {
                CTX_KEY_DEVICE.to_string() => device,
            });
        }

        if image != String::from("NA") {
            context.update(&indexmap! {
                CTX_KEY_IMAGE.to_string() => image,
            });
        }

        if !workspace.valid_config(config.as_str()) {
            return Err(BError::CliError(format!(
                "Unsupported build config '{}'",
                config
            )));
        }

        /*
         * We will update the context with the variables from the cli
         * and then expand the context variables in the config
         */
        workspace.update_ctx(&context)?;

        let deploy: &WsCustomSubCmdHandler = workspace.config().deploy();
        deploy.run(cli, &cli.env(), false, self.cmd.interactive)
    }
}

impl DeployCommand {
    pub fn new() -> Self {
        let subcmd: clap::Command = clap::Command::new(BCOMMAND)
        .about(BCOMMAND_ABOUT)
        .arg(
          clap::Arg::new("config")
              .short('c')
              .long("config")
              .help("The build config defining deploy task")
              .value_name("name")
              .required(true),
        )
        .arg(
            clap::Arg::new("verbose")
                .action(clap::ArgAction::SetTrue)
                .long("verbose")
                .help("Set verbose level."),
        )
        .arg(
          clap::Arg::new("ctx")
              .action(clap::ArgAction::Append)
              .short('x')
              .long("context")
              .value_name("KEY=VALUE")
              .help("Adding variable to the context. Any KEY that already exists in the context will be overwriten."),
        )
        .arg(
            clap::Arg::new("device")
                .action(clap::ArgAction::Append)
                .short('d')
                .long("device")
                .value_name("device")
                .default_value("NA")
                .help("The device can either be an IP or a device file like /dev/ttyUSB. Will be exposed as a context variable $#[DEVICE]"),
        )
        .arg(
            clap::Arg::new("image")
                .action(clap::ArgAction::Append)
                .short('i')
                .long("image")
                .value_name("image")
                .default_value("NA")
                .help("The image will be exposed as a context variable $#[IMAGE]"),
          );
        // Initialize and return a new DeployCommand instance
        DeployCommand {
            // Initialize fields if any
            cmd: BBaseCommand {
                cmd_str: String::from(BCOMMAND),
                sub_cmd: subcmd,
                interactive: true,
                require_docker: false,
            },
        }
    }
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;
    use std::path::PathBuf;
    use tempdir::TempDir;

    use crate::cli::*;
    use crate::commands::{BCommand, DeployCommand};
    use crate::error::BError;
    use crate::workspace::{Workspace, WsBuildConfigHandler, WsSettingsHandler};

    #[test]
    fn test_cmd_deploy() {
        let temp_dir: TempDir =
            TempDir::new("bakery-test-dir").expect("Failed to create temp directory");
        let work_dir: &PathBuf = &temp_dir.into_path();
        let json_ws_settings: &str = r#"
        {
            "version": "5",
            "builds": {
                "supported": [
                    "default"
                ]
            }
        }"#;
        let json_build_config: &str = r#"
        {
            "version": "5",
            "name": "default",
            "description": "Test Description",
            "arch": "test-arch",
            "bb": {},
            "context": [
                "ARG1=arg1",
                "ARG2=arg2",
                "ARG3=arg3"
            ],
            "deploy": {
                "cmd": "$#[SCRIPTS_DIR]/script.sh $#[ARG1] $#[ARG2] $#[ARG3]"
            }
        }
        "#;
        let mut mocked_system: MockSystem = MockSystem::new();
        mocked_system
            .expect_check_call()
            .with(mockall::predicate::eq(CallParams {
                cmd_line: vec![
                    &format!("{}/scripts/script.sh", work_dir.display()),
                    "arg1",
                    "arg2",
                    "arg3",
                ]
                .iter()
                .map(|s| s.to_string())
                .collect(),
                env: HashMap::new(),
                shell: true,
            }))
            .once()
            .returning(|_x| Ok(()));
        mocked_system.expect_env().returning(|| HashMap::new());
        let settings: WsSettingsHandler = WsSettingsHandler::from_str(work_dir, json_ws_settings)
            .expect("Failed to parse settings");
        let config: WsBuildConfigHandler =
            WsBuildConfigHandler::from_str(json_build_config, &settings)
                .expect("Failed to parse build config");
        let mut workspace: Workspace =
            Workspace::new(Some(work_dir.to_owned()), Some(settings), Some(config))
                .expect("Failed to setup workspace");
        let cli: Cli = Cli::new(
            Box::new(BLogger::new()),
            Box::new(mocked_system),
            clap::Command::new("bakery"),
            Some(vec!["bakery", "deploy", "-c", "default"]),
        );
        let cmd: DeployCommand = DeployCommand::new();
        let _result: Result<(), BError> = cmd.execute(&cli, &mut workspace);
    }

    #[test]
    fn test_cmd_deploy_ctx() {
        let temp_dir: TempDir =
            TempDir::new("bakery-test-dir").expect("Failed to create temp directory");
        let work_dir: &PathBuf = &temp_dir.into_path();
        let json_ws_settings: &str = r#"
        {
            "version": "5",
            "builds": {
                "supported": [
                    "default"
                ]
            }
        }"#;
        let json_build_config: &str = r#"
        {
            "version": "5",
            "name": "default",
            "description": "Test Description",
            "arch": "test-arch",
            "bb": {},
            "context": [
                "ARG1=arg1",
                "ARG2=arg2",
                "ARG3=arg3"
            ],
            "deploy": {
                "cmd": "$#[SCRIPTS_DIR]/script.sh $#[ARG1] $#[ARG2] $#[ARG3]"
            }
        }
        "#;
        let mut mocked_system: MockSystem = MockSystem::new();
        mocked_system
            .expect_check_call()
            .with(mockall::predicate::eq(CallParams {
                cmd_line: vec![
                    &format!("{}/scripts/script.sh", work_dir.display()),
                    "arg1",
                    "arg2",
                    "arg4",
                ]
                .iter()
                .map(|s| s.to_string())
                .collect(),
                env: HashMap::new(),
                shell: true,
            }))
            .once()
            .returning(|_x| Ok(()));
        mocked_system.expect_env().returning(|| HashMap::new());
        let settings: WsSettingsHandler = WsSettingsHandler::from_str(work_dir, json_ws_settings)
            .expect("Failed to parse settings");
        let config: WsBuildConfigHandler =
            WsBuildConfigHandler::from_str(json_build_config, &settings)
                .expect("Failed to parse build config");
        let mut workspace: Workspace =
            Workspace::new(Some(work_dir.to_owned()), Some(settings), Some(config))
                .expect("Failed to setup workspace");
        let cli: Cli = Cli::new(
            Box::new(BLogger::new()),
            Box::new(mocked_system),
            clap::Command::new("bakery"),
            Some(vec![
                "bakery",
                "deploy",
                "-c",
                "default",
                "--context",
                "ARG3=arg4",
            ]),
        );
        let cmd: DeployCommand = DeployCommand::new();
        let _result: Result<(), BError> = cmd.execute(&cli, &mut workspace);
    }

    #[test]
    fn test_cmd_deploy_device() {
        let temp_dir: TempDir =
            TempDir::new("bakery-test-dir").expect("Failed to create temp directory");
        let work_dir: &PathBuf = &temp_dir.into_path();
        let json_ws_settings: &str = r#"
        {
            "version": "5",
            "builds": {
                "supported": [
                    "default"
                ]
            }
        }"#;
        let json_build_config: &str = r#"
        {
            "version": "5",
            "name": "default",
            "description": "Test Description",
            "arch": "test-arch",
            "bb": {},
            "context": [
                "ARG2=arg2",
                "ARG3=arg3"
            ],
            "deploy": {
                "cmd": "$#[SCRIPTS_DIR]/script.sh $#[DEVICE] $#[ARG2] $#[ARG3]"
            }
        }
        "#;
        let mut mocked_system: MockSystem = MockSystem::new();
        mocked_system
            .expect_check_call()
            .with(mockall::predicate::eq(CallParams {
                cmd_line: vec![
                    &format!("{}/scripts/script.sh", work_dir.display()),
                    "192.168.1.90",
                    "arg2",
                    "arg3",
                ]
                .iter()
                .map(|s| s.to_string())
                .collect(),
                env: HashMap::new(),
                shell: true,
            }))
            .once()
            .returning(|_x| Ok(()));
        mocked_system.expect_env().returning(|| HashMap::new());
        let settings: WsSettingsHandler = WsSettingsHandler::from_str(work_dir, json_ws_settings)
            .expect("Failed to parse settings");
        let config: WsBuildConfigHandler =
            WsBuildConfigHandler::from_str(json_build_config, &settings)
                .expect("Failed to parse build config");
        let mut workspace: Workspace =
            Workspace::new(Some(work_dir.to_owned()), Some(settings), Some(config))
                .expect("Failed to setup workspace");
        let cli: Cli = Cli::new(
            Box::new(BLogger::new()),
            Box::new(mocked_system),
            clap::Command::new("bakery"),
            Some(vec![
                "bakery",
                "deploy",
                "-c",
                "default",
                "--device",
                "192.168.1.90",
            ]),
        );
        let cmd: DeployCommand = DeployCommand::new();
        let _result: Result<(), BError> = cmd.execute(&cli, &mut workspace);
    }

    #[test]
    fn test_cmd_deploy_device_ctx() {
        let temp_dir: TempDir =
            TempDir::new("bakery-test-dir").expect("Failed to create temp directory");
        let work_dir: &PathBuf = &temp_dir.into_path();
        let json_ws_settings: &str = r#"
        {
            "version": "5",
            "builds": {
                "supported": [
                    "default"
                ]
            }
        }"#;
        let json_build_config: &str = r#"
        {
            "version": "5",
            "name": "default",
            "description": "Test Description",
            "arch": "test-arch",
            "bb": {},
            "context": [
                "DEVICE=192.168.253.90"
            ],
            "deploy": {
                "cmd": "$#[SCRIPTS_DIR]/script.sh $#[DEVICE]"
            }
        }
        "#;
        let mut mocked_system: MockSystem = MockSystem::new();
        mocked_system
            .expect_check_call()
            .with(mockall::predicate::eq(CallParams {
                cmd_line: vec![
                    &format!("{}/scripts/script.sh", work_dir.display()),
                    "192.168.253.90",
                ]
                .iter()
                .map(|s| s.to_string())
                .collect(),
                env: HashMap::new(),
                shell: true,
            }))
            .once()
            .returning(|_x| Ok(()));
        mocked_system.expect_env().returning(|| HashMap::new());
        let settings: WsSettingsHandler = WsSettingsHandler::from_str(work_dir, json_ws_settings)
            .expect("Failed to parse settings");
        let config: WsBuildConfigHandler =
            WsBuildConfigHandler::from_str(json_build_config, &settings)
                .expect("Failed to parse build config");
        let mut workspace: Workspace =
            Workspace::new(Some(work_dir.to_owned()), Some(settings), Some(config))
                .expect("Failed to setup workspace");
        let cli: Cli = Cli::new(
            Box::new(BLogger::new()),
            Box::new(mocked_system),
            clap::Command::new("bakery"),
            Some(vec!["bakery", "deploy", "-c", "default"]),
        );
        let cmd: DeployCommand = DeployCommand::new();
        let _result: Result<(), BError> = cmd.execute(&cli, &mut workspace);
    }

    #[test]
    fn test_cmd_deploy_device_arg() {
        let temp_dir: TempDir =
            TempDir::new("bakery-test-dir").expect("Failed to create temp directory");
        let work_dir: &PathBuf = &temp_dir.into_path();
        let json_ws_settings: &str = r#"
        {
            "version": "5",
            "builds": {
                "supported": [
                    "default"
                ]
            }
        }"#;
        let json_build_config: &str = r#"
        {
            "version": "5",
            "name": "default",
            "description": "Test Description",
            "arch": "test-arch",
            "bb": {},
            "context": [
                "DEVICE=192.168.253.90"
            ],
            "deploy": {
                "cmd": "$#[SCRIPTS_DIR]/script.sh $#[DEVICE]"
            }
        }
        "#;
        let mut mocked_system: MockSystem = MockSystem::new();
        mocked_system
            .expect_check_call()
            .with(mockall::predicate::eq(CallParams {
                cmd_line: vec![
                    &format!("{}/scripts/script.sh", work_dir.display()),
                    "192.168.253.91",
                ]
                .iter()
                .map(|s| s.to_string())
                .collect(),
                env: HashMap::new(),
                shell: true,
            }))
            .once()
            .returning(|_x| Ok(()));
        mocked_system.expect_env().returning(|| HashMap::new());
        let settings: WsSettingsHandler = WsSettingsHandler::from_str(work_dir, json_ws_settings)
            .expect("Failed to parse settings");
        let config: WsBuildConfigHandler =
            WsBuildConfigHandler::from_str(json_build_config, &settings)
                .expect("Failed to parse build config");
        let mut workspace: Workspace =
            Workspace::new(Some(work_dir.to_owned()), Some(settings), Some(config))
                .expect("Failed to setup workspace");
        let cli: Cli = Cli::new(
            Box::new(BLogger::new()),
            Box::new(mocked_system),
            clap::Command::new("bakery"),
            Some(vec![
                "bakery",
                "deploy",
                "-c",
                "default",
                "-d",
                "192.168.253.91",
            ]),
        );
        let cmd: DeployCommand = DeployCommand::new();
        let _result: Result<(), BError> = cmd.execute(&cli, &mut workspace);
    }

    #[test]
    fn test_cmd_deploy_image_ctx() {
        let temp_dir: TempDir =
            TempDir::new("bakery-test-dir").expect("Failed to create temp directory");
        let work_dir: &PathBuf = &temp_dir.into_path();
        let json_ws_settings: &str = r#"
        {
            "version": "5",
            "builds": {
                "supported": [
                    "default"
                ]
            }
        }"#;
        let json_build_config: &str = r#"
        {
            "version": "5",
            "name": "default",
            "description": "Test Description",
            "arch": "test-arch",
            "bb": {},
            "context": [
                "IMAGE=ctx-test-image"
            ],
            "deploy": {
                "cmd": "$#[SCRIPTS_DIR]/script.sh $#[IMAGE]"
            }
        }
        "#;
        let mut mocked_system: MockSystem = MockSystem::new();
        mocked_system
            .expect_check_call()
            .with(mockall::predicate::eq(CallParams {
                cmd_line: vec![
                    &format!("{}/scripts/script.sh", work_dir.display()),
                    "ctx-test-image",
                ]
                .iter()
                .map(|s| s.to_string())
                .collect(),
                env: HashMap::new(),
                shell: true,
            }))
            .once()
            .returning(|_x| Ok(()));
        mocked_system.expect_env().returning(|| HashMap::new());
        let settings: WsSettingsHandler = WsSettingsHandler::from_str(work_dir, json_ws_settings)
            .expect("Failed to parse settings");
        let config: WsBuildConfigHandler =
            WsBuildConfigHandler::from_str(json_build_config, &settings)
                .expect("Failed to parse build config");
        let mut workspace: Workspace =
            Workspace::new(Some(work_dir.to_owned()), Some(settings), Some(config))
                .expect("Failed to setup workspace");
        let cli: Cli = Cli::new(
            Box::new(BLogger::new()),
            Box::new(mocked_system),
            clap::Command::new("bakery"),
            Some(vec!["bakery", "deploy", "-c", "default"]),
        );
        let cmd: DeployCommand = DeployCommand::new();
        let _result: Result<(), BError> = cmd.execute(&cli, &mut workspace);
    }

    #[test]
    fn test_cmd_deploy_image_arg() {
        let temp_dir: TempDir =
            TempDir::new("bakery-test-dir").expect("Failed to create temp directory");
        let work_dir: &PathBuf = &temp_dir.into_path();
        let json_ws_settings: &str = r#"
        {
            "version": "5",
            "builds": {
                "supported": [
                    "default"
                ]
            }
        }"#;
        let json_build_config: &str = r#"
        {
            "version": "5",
            "name": "default",
            "description": "Test Description",
            "arch": "test-arch",
            "bb": {},
            "context": [
                "IMAGE=ctx-test-image"
            ],
            "deploy": {
                "cmd": "$#[SCRIPTS_DIR]/script.sh $#[IMAGE]"
            }
        }
        "#;
        let mut mocked_system: MockSystem = MockSystem::new();
        mocked_system
            .expect_check_call()
            .with(mockall::predicate::eq(CallParams {
                cmd_line: vec![
                    &format!("{}/scripts/script.sh", work_dir.display()),
                    "arg-test-image",
                ]
                .iter()
                .map(|s| s.to_string())
                .collect(),
                env: HashMap::new(),
                shell: true,
            }))
            .once()
            .returning(|_x| Ok(()));
        mocked_system.expect_env().returning(|| HashMap::new());
        let settings: WsSettingsHandler = WsSettingsHandler::from_str(work_dir, json_ws_settings)
            .expect("Failed to parse settings");
        let config: WsBuildConfigHandler =
            WsBuildConfigHandler::from_str(json_build_config, &settings)
                .expect("Failed to parse build config");
        let mut workspace: Workspace =
            Workspace::new(Some(work_dir.to_owned()), Some(settings), Some(config))
                .expect("Failed to setup workspace");
        let cli: Cli = Cli::new(
            Box::new(BLogger::new()),
            Box::new(mocked_system),
            clap::Command::new("bakery"),
            Some(vec![
                "bakery",
                "deploy",
                "-c",
                "default",
                "-i",
                "arg-test-image",
            ]),
        );
        let cmd: DeployCommand = DeployCommand::new();
        let _result: Result<(), BError> = cmd.execute(&cli, &mut workspace);
    }
}
