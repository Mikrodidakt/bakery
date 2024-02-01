use crate::cli::{BLogger, Cli};
use crate::commands::BCommand;
use crate::configs::WsConfigFileHandler;
use crate::error::BError;
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
                .version(env!("CARGO_PKG_VERSION"))
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

    pub fn match_or_exit<T>(&self, result: Result<T, BError>) -> T {
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
        let settings: WsSettingsHandler = self.match_or_exit::<WsSettingsHandler>(cfg_handler.ws_settings());
        let cmd_name: &str = self.cli.get_args().subcommand_name().unwrap();
        let cmd_result: Result<&Box<dyn BCommand>, BError> = self.cli.get_command(cmd_name);

        match cmd_result {
            Ok(command) => {
                let config: WsBuildConfigHandler = self.match_or_exit(cfg_handler.build_config(&command.get_config_name(&self.cli), &settings));
                let mut workspace: Workspace = self.match_or_exit::<Workspace>(Workspace::new(Some(work_dir), Some(settings), Some(config)));
                let _res: () = self.match_or_exit::<()>(command.execute(&self.cli, &mut workspace));
            }
            Err(err) => {
                self.cli.error(format!("{}", err.to_string()));
                std::process::exit(1);
            }
        }
        std::process::exit(0);
    }
}
