use std::collections::HashMap;
use std::path::PathBuf;

use crate::cli::Cli;
use crate::commands::{BBaseCommand, BCommand, BError};
use crate::workspace::Workspace;
use crate::executers::{Docker, DockerImage};

static BCOMMAND: &str = "shell";
static BCOMMAND_ABOUT: &str = "Start a shell inside docker or run a bitbake command inside or outside of docker";
pub struct ShellCommand {
    cmd: BBaseCommand,
    // Your struct fields and methods here
}

impl BCommand for ShellCommand {
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
        let docker: String = self.get_arg_str(cli, "docker", BCOMMAND)?;
        let volumes: Vec<String> = self.get_arg_many(cli, "volume", BCOMMAND)?;
        let env: Vec<String> = self.get_arg_many(cli, "env", BCOMMAND)?;

        /*
         * If docker is enabled in the workspace settings then bakery will be boottraped into a docker container
         * with a bakery inside and all the baking will be done inside that docker container. Not all commands should
         * be run inside of docker and if we are already inside docker we should not try and bootstrap into a
         * second docker container.
         */
        if workspace.settings().docker_enabled() && self.is_docker_required() && !Docker::inside_docker() {
            return self.bootstrap(cli, workspace, &volumes, true);
        }

        if config == "NA" {
            return self.run_shell(cli, workspace, &docker);
        } else {
            if !workspace.valid_config(config.as_str()) {
                return Err(BError::CliError(format!("Unsupported build config '{}'", config)));
            }

            if docker.is_empty() {
                return self.run_bitbake_shell(cli, workspace, &self.setup_env(env));
            }
        }
        println!("{}", config);
        Ok(())
    }
}

impl ShellCommand {
    pub fn new() -> Self {
        let subcmd: clap::Command = clap::Command::new(BCOMMAND)
        .about(BCOMMAND_ABOUT)
        .arg(
            clap::Arg::new("config")
                .short('c')
                .long("config")
                .help("Setup bitbake build environment if no task specified drop into shell")
                .value_name("name")
                .default_value("NA"),
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
            clap::Arg::new("env")
                .action(clap::ArgAction::Append)
                .short('e')
                .long("env")
                .value_name("KEY=VALUE")
                .help("Extra variables to add to build env for bitbake."),
        )
        .arg(
            clap::Arg::new("docker")
                .short('d')
                .long("docker")
                .value_name("registry/image:tag")
                .default_value("")
                .help("Use a custome docker image when creating a shell"),
        );
        // Initialize and return a new BuildCommand instance
        ShellCommand {
            // Initialize fields if any
            cmd: BBaseCommand {
                cmd_str: String::from(BCOMMAND),
                sub_cmd: subcmd,
                interactive: true,
                require_docker: true,
            },
        }
    }

    fn setup_env(&self, env: Vec<String>) -> HashMap<String, String> {
        let variables: HashMap<String, String> = env.iter().map(|e|{
            let v: Vec<&str> = e.split('=').collect();
            (v[0].to_string(), v[1].to_string())
        }).collect();
        variables
    }

    fn bb_build_env(&self, cli: &Cli, workspace: &Workspace, args_env_variables: &HashMap<String, String>) -> Result<HashMap<String, String>, BError> {
        let init_env: PathBuf = workspace.config().build_data().bitbake().init_env_file();

        // Env variables priority are
        // 1. Cli env variables
        // 2. System env variables

        // Sourcing the init env file and returning all the env variables available including from the shell
        cli.info(format!("source init env file {}", init_env.display()));
        let mut env: HashMap<String, String> = cli.source_init_env(&init_env, &workspace.config().build_data().bitbake().build_dir())?;
        // Any variable that should be able to passthrough into bitbake needs to be defined as part of the bb passthrough variable
        // we define some defaults that should always be possible to passthrough
        let mut bb_env_passthrough_additions: String = String::from("SSTATE_DIR DL_DIR TMPDIR");

        // Process the env variables from the cli
        args_env_variables.iter().for_each(|(key, value)| {
            env.insert(key.clone(), value.clone());
            // Any variable comming from the cli should not by default be added to the passthrough
            // list. The only way to get it through is if this variable is already defined as part
            // of the task build config env
        });

        if env.contains_key("BB_ENV_PASSTHROUGH_ADDITIONS") {
            bb_env_passthrough_additions.push_str(env.get("BB_ENV_PASSTHROUGH_ADDITIONS").unwrap_or(&String::from("")));
        }

        env.insert(String::from("BB_ENV_PASSTHROUGH_ADDITIONS"), bb_env_passthrough_additions);
        Ok(env)
    }

    pub fn run_bitbake_shell(&self, cli: &Cli, workspace: &Workspace, args_env_variables: &HashMap<String, String>) -> Result<(), BError> {
        let cmd_line: Vec<String> = vec![
            String::from("/bin/bash"),
            String::from("-i")
        ];
        let env: HashMap<String, String> = self.bb_build_env(cli, workspace, args_env_variables)?;
        cli.check_call(&cmd_line, &env, true)
    }

    pub fn run_shell(&self, cli: &Cli, workspace: &Workspace, docker: &String) -> Result<(), BError> {
        let cmd_line: Vec<String> = vec![
            String::from("/bin/bash"),
            String::from("-i")
        ];
        
        if !docker.is_empty() {
            let image: DockerImage = DockerImage::new(&docker);
            let executer: Docker = Docker::new(image, true);
            return executer.run_cmd(&cmd_line, &HashMap::new(), &workspace.settings().work_dir(), cli);
        }

        cli.check_call(&cmd_line, &HashMap::new(), true)
    }
}