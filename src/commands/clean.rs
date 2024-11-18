use indexmap::IndexMap;
use std::collections::HashMap;

use crate::cli::Cli;
use crate::commands::{BBaseCommand, BCommand};
use crate::data::WsContextData;
use crate::error::BError;
use crate::executers::Docker;
use crate::workspace::{Workspace, WsTaskHandler};

static BCOMMAND: &str = "clean";
static BCOMMAND_ABOUT: &str = "Clean one or all tasks defined in a build config.";
pub struct CleanCommand {
    cmd: BBaseCommand,
    // Your struct fields and methods here
}

impl BCommand for CleanCommand {
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
        let tasks: Vec<String> = self.get_arg_many(cli, "tasks", BCOMMAND)?;
        let args_context: IndexMap<String, String> = self.setup_context(ctx);
        let context: WsContextData = WsContextData::new(&args_context)?;

        if !workspace.valid_config(config.as_str()) {
            return Err(BError::CliError(format!(
                "Unsupported build config '{}'",
                config
            )));
        }

        /*
         * If docker is enabled in the workspace settings then bakery will be boottraped into a docker container
         * with a bakery inside and all the baking will be done inside that docker container. Not all commands should
         * be run inside of docker and if we are already inside docker we should not try and bootstrap into a
         * second docker container.
         */
        if !workspace.settings().docker_disabled()
            && self.is_docker_required()
            && !Docker::inside_docker()
        {
            return self.bootstrap(
                &cli.get_cmd_line(),
                cli,
                workspace,
                &vec![],
                self.cmd.interactive,
            );
        }

        /*
         * We will update the context with the variables from the cli
         * and then expand the context variables in the config
         */
        workspace.update_ctx(&context)?;

        /*
         * TODO: we should handle env variables but for now we will just
         * send in an empty list to the cleaning task
         */
        let env_variables: HashMap<String, String> = HashMap::new();

        if tasks.len() > 1 {
            // More then one task was specified on the command line
            for t_name in tasks {
                let task: &WsTaskHandler = workspace.config().task(&t_name)?;
                task.clean(cli, &workspace.config().build_data(), &env_variables)?;
            }
        } else {
            // One task was specified on the command line or default was used
            let task: &String = tasks.get(0).unwrap();
            if task == "all" {
                // The alias "all" was specified on the command line or it none was specified and "all" was used
                for (_t_name, task) in workspace.config().tasks() {
                    task.clean(cli, &workspace.config().build_data(), &env_variables)?;
                }
            } else {
                // One task was specified on the command line
                let task: &WsTaskHandler = workspace.config().task(tasks.get(0).unwrap())?;
                task.clean(cli, &workspace.config().build_data(), &env_variables)?;
            }
        }
        Ok(())
    }
}

impl CleanCommand {
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
            clap::Arg::new("tasks")
                .short('t')
                .long("tasks")
                .value_name("tasks")
                .default_value("all")
                .value_delimiter(',')
                .help("The task(s) to clean."),
          )
          .arg(
            clap::Arg::new("ctx")
                .action(clap::ArgAction::Append)
                .short('x')
                .long("context")
                .value_name("KEY=VALUE")
                .help("Adding variable to the context. Any KEY that already exists in the context will be overwriten."),
          );
        // Initialize and return a new BuildCommand instance
        CleanCommand {
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

#[cfg(test)]
mod tests {
    use std::collections::HashMap;
    use std::path::PathBuf;
    use tempdir::TempDir;

    use crate::cli::*;
    use crate::commands::{BCommand, CleanCommand};
    use crate::error::BError;
    use crate::workspace::{Workspace, WsBuildConfigHandler, WsSettingsHandler};

    fn helper_test_clean_subcommand(
        json_ws_settings: &str,
        json_build_config: &str,
        work_dir: &PathBuf,
        logger: Box<dyn Logger>,
        system: Box<dyn System>,
        cmd_line: Vec<&str>,
    ) -> Result<(), BError> {
        let settings: WsSettingsHandler = WsSettingsHandler::from_str(work_dir, json_ws_settings)?;
        let config: WsBuildConfigHandler =
            WsBuildConfigHandler::from_str(json_build_config, &settings)?;
        let mut workspace: Workspace =
            Workspace::new(Some(work_dir.to_owned()), Some(settings), Some(config))?;
        let cli: Cli = Cli::new(logger, system, clap::Command::new("bakery"), Some(cmd_line));
        let cmd: CleanCommand = CleanCommand::new();
        cmd.execute(&cli, &mut workspace)
    }

    #[test]
    fn test_cmd_clean_nonbitbake() {
        let json_ws_settings: &str = r#"
        {
            "version": "5",
            "builds": {
                "supported": [
                    "default"
                ]
            },
            "docker": {
                "disabled": "true"
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
                "task-name": {
                    "index": "1",
                    "name": "task-name",
                    "type": "non-hlos",
                    "builddir": "test-dir",
                    "build": "test.sh",
                    "clean": "rm -rf dir-to-delete"
                }
            }
        }
        "#;
        let temp_dir: TempDir =
            TempDir::new("bakery-test-dir").expect("Failed to create temp directory");
        let work_dir: PathBuf = temp_dir.into_path();
        let build_dir: PathBuf = work_dir.join("test-dir");
        let mut mocked_system: MockSystem = MockSystem::new();
        mocked_system
            .expect_check_call()
            .with(mockall::predicate::eq(CallParams {
                cmd_line: vec![
                    "cd",
                    &build_dir.to_string_lossy().to_string(),
                    "&&",
                    "rm",
                    "-rf",
                    "dir-to-delete",
                ]
                .iter()
                .map(|s| s.to_string())
                .collect(),
                env: HashMap::new(),
                shell: true,
            }))
            .once()
            .returning(|_x| Ok(()));
        let _result: Result<(), BError> = helper_test_clean_subcommand(
            json_ws_settings,
            json_build_config,
            &work_dir,
            Box::new(BLogger::new()),
            Box::new(mocked_system),
            vec!["bakery", "clean", "--config", "default"],
        );
    }
}
