use std::collections::HashMap;
use indexmap::IndexMap;

use crate::cli::Cli;
use crate::commands::{BBaseCommand, BCommand, BError};
use crate::workspace::{Workspace, WsDeployHandler};
use crate::data::WsContextData;

static BCOMMAND: &str = "deploy";
static BCOMMAND_ABOUT: &str = "Deploy artifact to target";
pub struct DeployCommand {
    cmd: BBaseCommand,
    // Your struct fields and methods here
}

impl BCommand for DeployCommand {
    fn get_config_name(&self, cli: &Cli) -> String {
        if let Some(sub_matches) = cli.get_args().subcommand_matches(BCOMMAND) {
            if sub_matches.contains_id("config") {
                if let Some(value) = sub_matches.get_one::<String>("config") {
                    return value.clone()
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
        let args_context: IndexMap<String, String> = self.setup_context(ctx);
        let context: WsContextData = WsContextData::new(&args_context)?;

        if !workspace.valid_config(config.as_str()) {
            return Err(BError::CliError(format!("Unsupported build config '{}'", config)));
        }

        workspace.update_ctx(&context);
        let deploy: &WsDeployHandler = workspace.config().deploy();
        deploy.run(cli, &HashMap::new(), false, self.cmd.interactive)
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
          clap::Arg::new("ctx")
              .action(clap::ArgAction::Append)
              .short('x')
              .long("context")
              .value_name("KEY=VALUE")
              .help("Adding variable to the context. Any KEY that already exists in the context will be overwriten."),
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