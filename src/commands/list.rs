use indexmap::IndexMap;

use crate::cli::Cli;
use crate::commands::{BBaseCommand, BCommand, BError};
use crate::workspace::Workspace;

//use clap::{ArgMatches, value_parser};

static BCOMMAND: &str = "list";
static BCOMMAND_ABOUT: &str =
    "List all builds configs or all tasks available for a specific build config.";
pub struct ListCommand {
    cmd: BBaseCommand,
    // Your struct fields and methods here
}

impl BCommand for ListCommand {
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
        let ctx: bool = self.get_arg_flag(cli, "ctx", BCOMMAND)?;
        if config == "NA" {
            // default value if not specified
            // If no config is specified then we will list all supported build configs
            cli.stdout(format!("{:<25} {:<52}", "NAME", "DESCRIPTION"));
            workspace
                .build_configs()
                .iter()
                .for_each(|(path, description)| {
                    cli.stdout(format!(
                        "{:<25} - {:<50}",
                        path.file_stem().unwrap().to_string_lossy(),
                        description
                    ));
                });
        } else {
            // List all tasks for a build config
            if workspace.valid_config(config.as_str()) {
                workspace.expand_ctx()?;
                cli.stdout(format!(
                    "name: {}\narch: {}\nmachine: NA\ndescription: {}\n",
                    workspace.config().build_data().name(),
                    workspace.config().build_data().product().arch(),
                    workspace.config().build_data().product().description()
                ));

                if ctx {
                    let variables: IndexMap<String, String> = workspace.context()?;
                    cli.stdout("Context variables:".to_string());
                    variables.iter().for_each(|(key, value)| {
                        cli.stdout(format!("{}={}", key.to_ascii_uppercase(), value));
                    });
                } else {
                    cli.stdout(format!(
                        "{:<15} {:<52} {}",
                        "NAME", "DESCRIPTION", "ENABLED/DISABLED"
                    ));
                    workspace.config().tasks().iter().for_each(|(_name, task)| {
                        cli.stdout(format!(
                            "{:<15} - {:<50} [{}]",
                            task.data().name(),
                            task.data().description(),
                            if task.data().disabled() {
                                "disabled"
                            } else {
                                "enabled"
                            }
                        ));
                    });
               }
            } else {
                return Err(BError::CliError(format!(
                    "Unsupported build config '{}'",
                    config
                )));
            }
        }
        Ok(())
    }
}

impl ListCommand {
    pub fn new() -> Self {
        let subcmd: clap::Command = clap::Command::new(BCOMMAND)
            .about(BCOMMAND_ABOUT)
            .arg(
                clap::Arg::new("config")
                    .short('c')
                    .long("config")
                    .help("The build config defining all the components for the full build")
                    .value_name("name")
                    .default_value("NA"),
            )
            .arg(
                clap::Arg::new("verbose")
                    .action(clap::ArgAction::SetTrue)
                    .long("verbose")
                    .help("Set verbose level."),
            )
            .arg(
                clap::Arg::new("ctx")
                    .action(clap::ArgAction::SetTrue)
                    .long("ctx")
                    .help("List the context variables for a build config"),
            );
        // Initialize and return a new BuildCommand instance
        ListCommand {
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
    use indexmap::{indexmap, IndexMap};
    use std::path::PathBuf;
    use tempdir::TempDir;

    use crate::cli::*;
    use crate::commands::{BCommand, ListCommand};
    use crate::error::BError;
    use crate::workspace::{Workspace, WsBuildConfigHandler, WsSettingsHandler};

    fn helper_test_list_subcommand(
        work_dir: &PathBuf,
        json_ws_settings: &str,
        json_build_config: &str,
        mlogger: MockLogger,
        msystem: MockSystem,
        cmd_line: Vec<&str>,
    ) -> Result<(), BError> {
        let settings: WsSettingsHandler = WsSettingsHandler::from_str(work_dir, json_ws_settings)?;
        let config: WsBuildConfigHandler =
            WsBuildConfigHandler::from_str(json_build_config, &settings)?;
        let mut workspace: Workspace =
            Workspace::new(Some(work_dir.to_owned()), Some(settings), Some(config))?;
        let cli: Cli = Cli::new(
            Box::new(mlogger),
            Box::new(msystem),
            clap::Command::new("yaab"),
            Some(cmd_line),
        );
        let cmd: ListCommand = ListCommand::new();
        cmd.execute(&cli, &mut workspace)
    }

    #[test]
    fn test_cmd_list_build_config() {
        let temp_dir: TempDir =
            TempDir::new("yaab-test-dir").expect("Failed to create temp directory");
        let work_dir: PathBuf = temp_dir.into_path();
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
            "tasks": {
                "task1": {
                    "index": "1",
                    "name": "task1",
                    "type": "non-hlos"
                },
                "task2": {
                    "index": "2",
                    "name": "task2",
                    "disabled": "true",
                    "description": "test",
                    "type": "non-hlos"
                }
            }
        }
        "#;
        let mut mocked_logger: MockLogger = MockLogger::new();
        mocked_logger
            .expect_stdout()
            .with(mockall::predicate::eq(
                "name: default\narch: test-arch\nmachine: NA\ndescription: Test Description\n"
                    .to_string(),
            ))
            .once()
            .returning(|_x| ());
        mocked_logger
            .expect_stdout()
            .with(mockall::predicate::eq(format!(
                "{:<15} {:<52} {}",
                "NAME", "DESCRIPTION", "ENABLED/DISABLED"
            )))
            .once()
            .returning(|_x| ());
        mocked_logger
            .expect_stdout()
            .with(mockall::predicate::eq(format!(
                "{:<15} - {:<50} [{}]",
                "task1", "NA", "enabled"
            )))
            .once()
            .returning(|_x| ());
        mocked_logger
            .expect_stdout()
            .with(mockall::predicate::eq(format!(
                "{:<15} - {:<50} [{}]",
                "task2", "test", "disabled"
            )))
            .once()
            .returning(|_x| ());
        let _result: Result<(), BError> = helper_test_list_subcommand(
            &work_dir,
            json_ws_settings,
            json_build_config,
            mocked_logger,
            MockSystem::new(),
            vec!["yaab", "list", "--config", "default"],
        );
    }

    #[test]
    fn test_cmd_list_invalid_build_config() {
        let temp_dir: TempDir =
            TempDir::new("yaab-test-dir").expect("Failed to create temp directory");
        let work_dir: PathBuf = temp_dir.into_path();
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
            "tasks": {
                "task1": {
                    "index": "1",
                    "name": "task1",
                    "type": "non-hlos"
                },
                "task2": {
                    "index": "2",
                    "name": "task2",
                    "type": "non-hlos"
                }
            }
        }
        "#;
        let result: Result<(), BError> = helper_test_list_subcommand(
            &work_dir,
            json_ws_settings,
            json_build_config,
            MockLogger::new(),
            MockSystem::new(),
            vec!["yaab", "list", "--config", "invalid"],
        );
        match result {
            Ok(_status) => {
                panic!("We should have recived an error because the config is invalid!");
            }
            Err(e) => {
                assert_eq!(
                    e.to_string(),
                    "Unsupported build config 'invalid'".to_string()
                );
            }
        }
    }

/*
    #[test]
    fn test_cmd_list_ctx() {
        let temp_dir: TempDir =
            TempDir::new("bakery-test-dir").expect("Failed to create temp directory");
        let work_dir: PathBuf = temp_dir.into_path();
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
            "context": [
                "PLATFORM_VERSION=x.y.z",
                "BUILD_ID=abcdef",
                "BUILD_VARIANT=test"
            ]
        }
        "#;
        let mut mocked_logger: MockLogger = MockLogger::new();
        mocked_logger
            .expect_stdout()
            .with(mockall::predicate::eq("name: default\narch: test-arch\nmachine: test-machine\ndescription: Test Description\n".to_string()))
            .once()
            .returning(|_x| ());
        mocked_logger
            .expect_stdout()
            .with(mockall::predicate::eq("Context variables:".to_string()))
            .once()
            .returning(|_x| ());
        let ctx_variables: IndexMap<String, String> = indexmap! {
            "MACHINE".to_string() => "test-machine".to_string(),
            "ARCH".to_string() => "test-arch".to_string(),
            "DISTRO".to_string() => "test-distro".to_string(),
            "PRODUCT_NAME".to_string() => "default".to_string(),
            "NAME".to_string() => "default".to_string(),
            "PRODUCT_NAME".to_string() => "default".to_string(),
            "PROJECT_NAME".to_string() => "default".to_string(),
            "BB_BUILD_DIR".to_string() => format!("{}", work_dir.join(PathBuf::from("builds/default")).display()),
            "BB_DEPLOY_DIR".to_string() => format!("{}", work_dir.join(PathBuf::from("builds/default/tmp/deploy/images")).display()),
            "ARTIFACTS_DIR".to_string() => format!("{}", work_dir.join(PathBuf::from("artifacts")).display()),
            "LAYERS_DIR".to_string() => format!("{}", work_dir.join(PathBuf::from("layers")).display()),
            "SCRIPTS_DIR".to_string() => format!("{}", work_dir.join(PathBuf::from("scripts")).display()),
            "BUILDS_DIR".to_string() => format!("{}", work_dir.join(PathBuf::from("builds")).display()),
            "WORK_DIR".to_string() => format!("{}", work_dir.display()),
            "PLATFORM_VERSION".to_string() => "x.y.z".to_string(),
            "BUILD_ID".to_string() => "abcdef".to_string(),
            "PLATFORM_RELEASE".to_string() => "".to_string(),
            "BUILD_SHA".to_string() => "".to_string(),
            "RELEASE_BUILD".to_string() => "".to_string(),
            "BUILD_VARIANT".to_string() => "test".to_string(),
            "ARCHIVER".to_string() => "".to_string(),
            "DEBUG_SYMBOLS".to_string() => "".to_string(),
            "DEVICE".to_string() => "".to_string(),
            "IMAGE".to_string() => "".to_string(),
            "DATE".to_string() => chrono::offset::Local::now().format("%Y-%m-%d").to_string(),
            "TIME".to_string() => chrono::offset::Local::now().format("%H:%M").to_string(),
        };
        ctx_variables.iter().for_each(|(key, value)| {
            mocked_logger
                .expect_stdout()
                .with(mockall::predicate::eq(format!("{}={}", key, value)))
                .once()
                .returning(|_x| ());
        });

        let _result: Result<(), BError> = helper_test_list_subcommand(
            &work_dir,
            json_ws_settings,
            json_build_config,
            mocked_logger,
            MockSystem::new(),
            vec!["bakery", "list", "--config", "default", "--ctx"],
        );
    }
*/
}
