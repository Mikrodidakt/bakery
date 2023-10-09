use std::env;
use std::path::PathBuf;

use clap::ArgMatches;

use crate::commands::{BCommand, BError, BBaseCommand};
use crate::executers::{DockerImage, Docker, Executer};
use crate::workspace::{WsSettingsHandler, WsBuildConfigHandler, Workspace};
use crate::cli::Cli;

static BCOMMAND: &str = "build";
static BCOMMAND_ABOUT: &str = "Execute a build either a full build or a task of one of the builds";

pub struct BuildCommand {
    cmd: BBaseCommand,
    // Your struct fields and methods here
}

impl BCommand for BuildCommand {
    fn cmd_str(&self) -> &str {
        &self.cmd.cmd_str
    }

    fn subcommand(&self) -> &clap::Command {
        &self.cmd.sub_cmd
    }

    fn execute(&self, cli: &Cli, workspace: &Workspace) -> Result<(), BError> {
        let exec: Executer = Executer::new(&workspace, cli);
        let docker_image: DockerImage = DockerImage {
            registry: String::from("registry"),
            image: String::from("test"),
            tag: String::from("0.1"),
        };
        let docker: Docker = Docker::new(&workspace, &docker_image, true);
        exec.execute(self.cmd_str(), env::vars(), Some(String::from("test2")), Some(docker), self.cmd.interactive)?;
        Ok(())
    }
}

impl BuildCommand {
    pub fn new() -> Self {
        let subcmd: clap::Command = clap::Command::new(BCOMMAND)
            .about(BCOMMAND_ABOUT)
            .arg_required_else_help(true)
            .arg(
                clap::Arg::new("config")
                    .short('c')
                    .long("config")
                    .help("The build config defining all the components for the full build")
                    .value_name("name")
                    .required(true),
            )
            .arg(
                clap::Arg::new("task")
                    .short('t')
                    .long("task")
                    .value_name("task")
                    .default_value("all")
                    .value_delimiter(',')
                    .help("The task(s) to execute."),
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
                clap::Arg::new("volume")
                    .action(clap::ArgAction::Append)
                    .short('v')
                    .long("docker-volume")
                    .value_name("volume")
                    .help("Docker volume to mount bind when boot strapping into docker."),
            )
            .arg(
                clap::Arg::new("build_history")
                    .action(clap::ArgAction::SetTrue)
                    .long("build-history")
                    .help("Records information about each package and image and commits that information to a local Git repository where you can examine the information."),
            )
            .arg(
                clap::Arg::new("archiver")
                    .action(clap::ArgAction::SetTrue)
                    .long("archiver")
                    .help("Setting context variable ARCHIVER to 1 which will result in adding the archiver class to the local.conf. For more information see https://www.yoctoproject.org/docs/latest/mega-manual/mega-manual.html#ref-classes-archiver."),
            )
            .arg(
                clap::Arg::new("debug_symbols")
                    .action(clap::ArgAction::SetTrue)
                    .long("debug-symbols")
                    .help("Setting context variable DEBUG_SYMBOLS to 1 which will result in adding IMAGE_GEN_DEBUGFS=1 to the local.conf. For more information see https://www.yoctoproject.org/docs/latest/mega-manual/mega-manual.html#platdev-gdb-remotedebug."),
            )
            .arg(
                clap::Arg::new("dry_run")
                    .action(clap::ArgAction::SetTrue)
                    .long("dry-run")
                    .help("Only generates local.conf. To manually start the build run source ./layers/poky/oe-init-env-build <build-dir> followed by any bitbake command."),
            )
            .arg(
                clap::Arg::new("tar_balls")
                    .action(clap::ArgAction::SetTrue)
                    .long("tar-balls")
                    .help("This will add BB_GENERATE_MIRROR_TARBALLS=1 to the local.conf. For more information see https://www.yoctoproject.org/docs/latest/mega-manual/mega-manual.html#var-BB_GENERATE_MIRROR_TARBALLS."),
            )
            .arg(
                clap::Arg::new("platform_version")
                    .short('r')
                    .long("platform-version")
                    .value_name("x.y.z")
                    .default_value("0.0.0")
                    .help("Platform version number for the build."),
            )
            .arg(
                clap::Arg::new("build_sha")
                    .short('s')
                    .long("build-sha")
                    .value_name("sha")
                    .default_value("dev")
                    .help("Sha for the current build."),
            )
            .arg(
                clap::Arg::new("variant")
                    .short('a')
                    .long("variant")
                    .value_name("variant")
                    .default_value("dev")
                    .value_parser(["dev", "manufacturing", "release"])
                    .default_value("dev")
                    .help("Specify the variant of the build it can be one of release, dev or manufacturing."),
            )
            .arg(
                clap::Arg::new("interactive")
                    .short('i')
                    .long("interactive")
                    .value_name("interactive")
                    .default_value("true")
                    .value_parser(["true", "false"])
                    .help("Determines if a build inside docker should be interactive or not."),
            )
            .arg(
                clap::Arg::new("build_id")
                    .short('n')
                    .long("build-id")
                    .value_name("nbr")
                    .default_value("0")
                    .help("Build id number can be used if x.y.z is not enough for some reason."),
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
        BuildCommand {
            // Initialize fields if any
            cmd : BBaseCommand {
                cmd_str: String::from(BCOMMAND),
                sub_cmd: subcmd,
                interactive: true,
                require_docker: true,
            }
        }
    }
}
