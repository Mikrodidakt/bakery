use indexmap::IndexMap;
use std::collections::HashMap;

use crate::cli::Cli;
use crate::commands::{BBaseCommand, BCommand, BError};
use crate::workspace::Workspace;
use crate::data::WsContextData;
use crate::workspace::WsTaskCmdHandler;
use crate::executers::Docker;

static BCOMMAND: &str = "upload";
static BCOMMAND_ABOUT: &str = "Upload artifacts to the Artifactory server.";
pub struct UploadCommand {
    cmd: BBaseCommand,
    // Your struct fields and methods here
}

impl BCommand for UploadCommand {
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
      let args_context: IndexMap<String, String> = self.setup_context(ctx);
      let context: WsContextData = WsContextData::new(&args_context)?;
      let volumes: Vec<String> = self.get_arg_many(cli, "volume", BCOMMAND)?;

      if !workspace.valid_config(config.as_str()) {
          return Err(BError::CliError(format!("Unsupported build config '{}'", config)));
      }

      /*
       * If docker is enabled in the workspace settings then bakery will be boottraped into a docker container
       * with a bakery inside and all the baking will be done inside that docker container. Not all commands should
       * be run inside of docker and if we are already inside docker we should not try and bootstrap into a
       * second docker container.
       */
      if !workspace.settings().docker_disabled() && self.is_docker_required() && !Docker::inside_docker() {
          return self.bootstrap(&cli.get_cmd_line(), cli, workspace, &volumes, self.cmd.interactive);
      }

      workspace.update_ctx(&context)?;

      let upload: &WsTaskCmdHandler = workspace.config().upload();
      upload.run(cli, &HashMap::new(), false, self.cmd.interactive)
  }
}

impl UploadCommand {
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
        clap::Arg::new("volume")
            .action(clap::ArgAction::Append)
            .short('v')
            .long("docker-volume")
            .value_name("path:path")
            .help("Docker volume to mount bind when boot strapping into docker."),
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
      UploadCommand {
          // Initialize fields if any
          cmd: BBaseCommand {
              cmd_str: String::from(BCOMMAND),
              sub_cmd: subcmd,
              interactive: true,
              require_docker: true,
          },
      }
  }
}