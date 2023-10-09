//use std::path::PathBuf;
use std::env;

use clap::ArgMatches;

use crate::cli::Cli;
use crate::commands::{BBaseCommand, BCommand, BError};
use crate::workspace::Workspace;
use crate::executers::{DockerImage, Docker, Executer};

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

    fn execute(&self, cli: &Cli, workspace: &Workspace) -> Result<(), BError> {
        if let Some(sub_matches) = cli.get_args().subcommand_matches(BCOMMAND) {
            if let Some(config) = sub_matches.get_one::<String>("config") {
                if config == "all" {
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
                    return Ok(());
                } else {
                    if workspace.valid_config(config) {
                        workspace.config().tasks().iter().for_each(|(name, task)| {
                            cli.stdout(format!("{}", task.name()));
                        });
                        return Ok(());
                    }
                    return Err(BError {
                        code: 1, // You may set the appropriate error code
                        message: format!("Unsupported build config '{}'", config),
                    })
                }
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
                    .default_value("all"),
            );
        // Initialize and return a new BuildCommand instance
        ListCommand {
            // Initialize fields if any
            cmd: BBaseCommand {
                cmd_str: String::from(BCOMMAND),
                sub_cmd: subcmd,
                interactive: true,
            },
        }
    }
}
