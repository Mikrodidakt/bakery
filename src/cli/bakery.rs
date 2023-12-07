use crate::cli::{BLogger, Cli};
use crate::commands::BCommand;
use crate::configs::WsConfigFileHandler;
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

    pub fn match_and_exit<T>(&self, result: Result<T, BError>) -> T {
        match result {
            Ok(content) => {
                return content;
            },
            Err(err) => {
                self.cli.error(format!("{}", err.to_string()));
                std::process::exit(1);
            }
        }
    }

    pub fn bake(&self) {
        let work_dir: PathBuf = self.cli.get_curr_dir();
        let home_dir: PathBuf = self.cli.get_home_dir();
        let cfg_handler: WsConfigFileHandler = WsConfigFileHandler::new(&work_dir, &home_dir);
        let settings: WsSettingsHandler = self.match_and_exit::<WsSettingsHandler>(cfg_handler.ws_settings());
        let cmd_name: &str = self.cli.get_args().subcommand_name().unwrap();
        let cmd_result: Result<&Box<dyn BCommand>, BError> = self.cli.get_command(cmd_name);

        match cmd_result {
            Ok(command) => {
                let config: WsBuildConfigHandler = self.match_and_exit(cfg_handler.build_config(&command.get_config_name(&self.cli), &settings));
                let mut workspace: Workspace = self.match_and_exit::<Workspace>(Workspace::new(Some(work_dir), Some(settings), Some(config)));
                
                /*
                 * If docker is enabled in the workspace settings then bakery will be boottraped into a docker container
                 * with a bakery inside and all the baking will be done inside that docker container
                 */
                if workspace.settings().docker_enabled() {
                    /*
                     * Not all commands should be run inside of docker and if we are already inside docker
                     * we should not try and bootstrap into a second docker container.
                     */
                    if command.is_docker_required() && !Docker::inside_docker() {
                        self.cli.info(format!("Bootstrap bakery into docker"));
                        let docker: Docker = Docker::new(workspace.settings().docker_image(), true);
                        let _result: Result<(), BError> = docker.bootstrap_bakery(self.cli.get_args());
                        std::process::exit(0);
                    }
                }
                
                let _res: () = self.match_and_exit::<()>(command.execute(&self.cli, &mut workspace));
            }
            Err(err) => {
                self.cli.error(format!("{}", err.to_string()));
                std::process::exit(1);
            }
        }
        std::process::exit(0);
    }
}
