use std::env;
use std::path::PathBuf;

use crate::commands::{BCommand, BError, BBaseCommand};
use crate::executers::{DockerImage, Docker, Executer};
use crate::configs::WorkspaceSettings;
use crate::workspace::Settings;
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
        // TODO: we need to handle the configs and workspace in some other way
        // for now let's keep it like this
        let json_test_str = r#"
        {
            "version": "4"
        }"#;
        let result: Result<WorkspaceSettings, BError> = WorkspaceSettings::from_str(json_test_str);
        let config: WorkspaceSettings;
        match result {
            Ok(rsettings) => {
                config = rsettings;
            }
            Err(e) => {
                eprintln!("Error parsing JSON: {}", e);
                panic!();
            } 
        }
        let work_dir: PathBuf = PathBuf::from("/workspace");
        let workspace: Workspace = Workspace::new(Some(work_dir), config);
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
