//use std::path::PathBuf;
use std::env;

use clap::ArgMatches;

use crate::cli::Cli;
use crate::commands::{BBaseCommand, BCommand, BError};
use crate::executers::{Docker, DockerImage, Executer};
use crate::workspace::Workspace;

//use clap::{ArgMatches, value_parser};

static BCOMMAND: &str = "list";
static BCOMMAND_ABOUT: &str = "List all builds or the tasks available for one build";
pub struct ListCommand {
    cmd: BBaseCommand,
    // Your struct fields and methods here
}

impl BCommand for ListCommand {
    fn cmd_str(&self) -> &str {
        &self.cmd.cmd_str
    }

    fn subcommand(&self) -> &clap::Command {
        &self.cmd.sub_cmd
    }

    fn is_docker_required(&self) -> bool {
        self.cmd.require_docker
    }

    fn execute(&self, cli: &Cli, workspace: &Workspace) -> Result<(), BError> {
        let config: String = self.get_args_config(cli, BCOMMAND)?;
        if config == "all" { // default value if not specified
            // If no config is specified then we will list all supported build configs
            workspace
                .build_configs()
                .iter()
                .for_each(|(path, description)| {
                    cli.stdout(format!(
                        "{}: {}",
                        path.file_name().unwrap().to_string_lossy(),
                        description
                    ));
                });
        } else {
            // List all tasks for a build config
            if workspace.valid_config(config.as_str()) {
                workspace.config().tasks().iter().for_each(|(_name, task)| {
                    cli.stdout(format!("{}", task.name()));
                });
            } else {
                return Err(BError::CliError(format!("Unsupported build config '{}'", config)));
            }
        }
        Ok(())
    }
}

impl ListCommand {
    pub fn new() -> Self {
        let subcmd: clap::Command = clap::Command::new(BCOMMAND).about(BCOMMAND_ABOUT).arg(
            clap::Arg::new("config")
                .short('c')
                .long("config")
                .help("The build config defining all the components for the full build")
                .value_name("name")
                .default_value("all"),
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
    use std::path::{Path, PathBuf};
    use tempdir::TempDir;

    use crate::cli::*;
    use crate::commands::{BCommand, ListCommand};
    use crate::error::BError;
    use crate::workspace::{Workspace, WsBuildConfigHandler, WsSettingsHandler};

    fn helper_test_list_subcommand(
        json_ws_settings: &str,
        json_build_config: &str,
        mlogger: MockLogger,
        msystem: MockSystem,
        cmd_line: Vec<&str>,
    ) -> Result<(), BError> {
        let temp_dir: TempDir =
            TempDir::new("bakery-test-dir").expect("Failed to create temp directory");
        let work_dir: &PathBuf = &temp_dir.into_path();
        let settings: WsSettingsHandler = WsSettingsHandler::from_str(work_dir, json_ws_settings)?;
        let config: WsBuildConfigHandler =
            WsBuildConfigHandler::from_str(json_build_config, &settings)?;
        let workspace: Workspace =
            Workspace::new(Some(work_dir.to_owned()), Some(settings), Some(config))?;
        let cli: Cli = Cli::new(
            Box::new(mlogger),
            Box::new(msystem),
            clap::Command::new("bakery"),
            Some(cmd_line),
        );
        let cmd: ListCommand = ListCommand::new();
        cmd.execute(&cli, &workspace)
    }

    #[test]
    fn test_cmd_list_no_build_config() {
        let json_ws_settings: &str = r#"
        {
            "version": "4",
            "builds": {
                "supported": [
                    "default"
                ]
            }
        }"#;
        let json_build_config: &str = r#"
        {
            "version": "4",
            "name": "default",
            "description": "Test Description",
            "arch": "test-arch",
            "bb": {}
        }
        "#;
        let mut mocked_logger: MockLogger = MockLogger::new();
        mocked_logger
            .expect_stdout()
            .with(mockall::predicate::eq(
                "default.json: Test Description".to_string(),
            ))
            .once()
            .returning(|_x| ());
        let _result: Result<(), BError> = helper_test_list_subcommand(
            json_ws_settings,
            json_build_config,
            mocked_logger,
            MockSystem::new(),
            vec!["bakery", "list"],
        );
    }

    #[test]
    fn test_cmd_list_build_config() {
        let json_ws_settings: &str = r#"
        {
            "version": "4",
            "builds": {
                "supported": [
                    "default"
                ]
            }
        }"#;
        let json_build_config: &str = r#"
        {
            "version": "4",
            "name": "default",
            "description": "Test Description",
            "arch": "test-arch",
            "bb": {},
            "tasks": { 
                "task1": {
                    "index": "1",
                    "name": "task1",
                    "type": "non-bitbake"
                },
                "task2": {
                    "index": "2",
                    "name": "task2",
                    "type": "non-bitbake"
                }
            }
        }
        "#;
        let mut mocked_logger: MockLogger = MockLogger::new();
        mocked_logger
            .expect_stdout()
            .with(mockall::predicate::eq("task1".to_string()))
            .once()
            .returning(|_x| ());
        mocked_logger
            .expect_stdout()
            .with(mockall::predicate::eq("task2".to_string()))
            .once()
            .returning(|_x| ());
        let _result: Result<(), BError> = helper_test_list_subcommand(
            json_ws_settings,
            json_build_config,
            mocked_logger,
            MockSystem::new(),
            vec!["bakery", "list", "--config", "default"],
        );
    }

    #[test]
    fn test_cmd_list_invalid_build_config() {
        let json_ws_settings: &str = r#"
        {
            "version": "4",
            "builds": {
                "supported": [
                    "default"
                ]
            }
        }"#;
        let json_build_config: &str = r#"
        {
            "version": "4",
            "name": "default",
            "description": "Test Description",
            "arch": "test-arch",
            "bb": {},
            "tasks": { 
                "task1": {
                    "index": "1",
                    "name": "task1",
                    "type": "non-bitbake"
                },
                "task2": {
                    "index": "2",
                    "name": "task2",
                    "type": "non-bitbake"
                }
            }
        }
        "#;
        let result: Result<(), BError> = helper_test_list_subcommand(
            json_ws_settings,
            json_build_config,
            MockLogger::new(),
            MockSystem::new(),
            vec!["bakery", "list", "--config", "invalid"],
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
}
