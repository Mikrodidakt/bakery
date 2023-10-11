use crate::cli::{BLogger, Cli};
use crate::commands::BCommand;
use crate::error::BError;
use crate::executers::Docker;
use crate::workspace::{WsSettingsHandler, Workspace, WsBuildConfigHandler};

use clap::Command;
use std::path::PathBuf;

use super::BSystem;

pub struct Bakery {
    cli: Cli,
}

impl Bakery {
    pub fn new() -> Self {
        /*
            TODO: We should try and use command! macro in clap so
            the about, author and version can be read out from the
            Cargo.toml
        */
        let cli: Cli = Cli::new(Box::new(BLogger::new()), 
            Box::new(BSystem::new()),
            Command::new("bakery")
                .version("0.0.1")
                .subcommand_required(true)
                .arg_required_else_help(true)
                .about("Build engine for the Yocto/OE")
                .author("bakery by Mikrodidakt(mikro.io)"),
            None
        );
        Bakery {
            cli: cli,
        }
    }

    pub fn bake(&self) {
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
        let work_dir: PathBuf = PathBuf::from("/workspace");
        let mut settings: WsSettingsHandler = WsSettingsHandler::from_str(&work_dir, json_ws_settings).expect("Failed to parse settings.json");
        let config: WsBuildConfigHandler = WsBuildConfigHandler::from_str(json_build_config, &mut settings).expect("Failed to parse build config");
        let workspace: Workspace = Workspace::new(Some(work_dir), Some(settings), Some(config)).expect("Failed to setup workspace");

        let cmd_name: &str = self.cli.get_args().subcommand_name().unwrap();
        let cmd: Result<&Box<dyn BCommand>, BError> = self.cli.get_command(cmd_name);

        match cmd {
            Ok(command) => {
                if command.is_docker_required() && !Docker::inside_docker() {
                    self.cli.info(format!("Bootstrap bakery into docker"));
                    let docker: Docker = Docker::new(&workspace, workspace.settings().docker_image(), true);
                    let result: Result<(), BError> = docker.bootstrap_bakery(self.cli.get_args());
                    std::process::exit(0);
                }

                let error: Result<(), BError> = command.execute(&self.cli, &workspace);
                match error {
                    Err(err) => {
                        self.cli.error(format!("{}", err.to_string()));
                        std::process::exit(1);
                    }
                    Ok(()) => {}
                }
            }
            Err(err) => {
                self.cli.error(format!("{}", err.to_string()));
                std::process::exit(1);
            }
        }
        std::process::exit(0);
    }
}
