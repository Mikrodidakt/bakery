use crate::cli::{BLogger, Cli};
use crate::commands::BCommand;
use crate::error::BError;

use clap::Command;

pub struct Bakery {
    cli: Cli,
    cli_matches: clap::ArgMatches,
}

impl Bakery {
    pub fn new() -> Self {
        let cli: Cli = Cli::new(Box::new(BLogger::new()));
        /*
            TODO: We should try and use command! macro in clap so
            the about, author and version can be read out from the
            Cargo.toml
        */
        let cmd = Command::new("bakery")
            .version("0.0.1")
            .subcommand_required(true)
            .arg_required_else_help(true)
            .about("Build engine for the Yocto/OE")
            .author("bakery by Mikrodidakt(mikro.io)");
        let matches = cli.build_cli(&cmd);
        Bakery {
            cli: cli,
            cli_matches: matches,
        }
    }

    pub fn bake(&self) {
        let cmd_name = self.cli_matches.subcommand_name();
        let cmd: Result<&Box<dyn BCommand>, BError> = self.cli.get_command(String::from(cmd_name.unwrap()));

        match cmd {
            Ok(command) => {
                let error: Result<(), BError> = command.execute(&self.cli);
                match error {
                    Err(err) => {
                        self.cli.error(format!("{}", err.message));
                        std::process::exit(1);
                    }
                    Ok(()) => {}
                }
            }
            Err(err) => {
                self.cli.error(format!("{}", err.message));
                std::process::exit(1);
            }
        }
        std::process::exit(0);
    }
}
