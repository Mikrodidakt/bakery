use std::env;
use std::path::PathBuf;

use crate::commands::{BCommand, BError, BBaseCommand};
use crate::executers::{DockerImage, Docker, Executer};
use crate::workspace::{WsSettingsHandler, WsBuildConfigHandler};
use crate::workspace::Workspace;
use crate::cli::Cli;

static BCOMMAND: &str = "build";
static BCOMMAND_ABOUT: &str = "Build one of the components";

pub struct BuildCommand {
    cmd: BBaseCommand,
    // Your struct fields and methods here
}

impl BCommand for BuildCommand {
    fn cmd_str(&self) -> &str {
        &self.cmd._cmd_str
    }

    fn subcommand(&self) -> &clap::Command {
        &self.cmd._subcmd
    }

    fn execute(&self, cli: &Cli) -> Result<(), BError> {
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
            "name": "test-name",
            "description": "Test Description",
            "arch": "test-arch",
            "bb": {}
        }
        "#;
        let work_dir: PathBuf = PathBuf::from("/workspace");
        let mut settings: WsSettingsHandler = WsSettingsHandler::from_str(&work_dir, json_ws_settings).expect("Failed to parse settings.json");
        let config: WsBuildConfigHandler = WsBuildConfigHandler::from_str(json_build_config, &mut settings).expect("Failed to parse build config");
        let workspace: Workspace = Workspace::new(Some(work_dir), Some(settings), Some(config)).expect("Failed to setup workspace");
        let exec: Executer = Executer::new(&workspace, cli);
        let docker_image: DockerImage = DockerImage {
            registry: String::from("registry"),
            image: String::from("test"),
            tag: String::from("0.1"),
        };
        let docker: Docker = Docker::new(&workspace, &docker_image, true);
        exec.execute(self.cmd_str(), env::vars(), Some(String::from("test2")), Some(docker), self.cmd._interactive)?;
        Ok(())
    }
}

impl BuildCommand {
    pub fn new() -> Self {
        let subcmd: clap::Command = clap::Command::new(BCOMMAND)
            .about(BCOMMAND_ABOUT);
        /*
            .arg_required_else_help(true)
            .arg(
                clap::Arg::new("config")
                    .short('c')
                    .long("config")
                    .help("The build config defining all the components for the full build")
                    .value_name("path")
                    .required(true),
            )
        */
        // Initialize and return a new BuildCommand instance
        BuildCommand {
            // Initialize fields if any
            cmd : BBaseCommand {
                _cmd_str: String::from(BCOMMAND),
                _subcmd: subcmd,
                _interactive: true
            }
        }
    }
}
