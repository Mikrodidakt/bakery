use clap::{ArgMatches, Arg};

use crate::commands::{CmdHandler, BCommand};
use crate::error::BError;
use crate::cli::Logger;

pub struct Cli {
    args: ArgMatches,
    cmd_handler: CmdHandler,
    pub logger: Box<dyn Logger>,
}

impl Cli {
    pub fn new(logger: Box<dyn Logger>, cmd: clap::Command, cmd_line: Option<Vec<&str>>) -> Self {
        let cmd_handler: CmdHandler = CmdHandler::new();
        let args: ArgMatches;
        match cmd_line {
            Some(cline) => {
                args = cmd_handler.build_cli(cmd).get_matches_from(cline);
            },
            None => {
                args = cmd_handler.build_cli(cmd).get_matches();
            }
        }
        Cli {
            args,
            cmd_handler,
            logger,
        } 
    }

    pub fn get_command(&self, name: &str) -> Result<&Box<dyn BCommand>, BError> {
        self.cmd_handler.get_cmd(&name)
    }

    pub fn get_args(&self) -> &ArgMatches {
        &self.args
    }

    //pub fn build_cli(&self, cmd: &clap::Command) -> clap::ArgMatches {
    //    self.cmd_handler.build_cli(cmd.clone()).get_matches()
    //}

    pub fn get_logger(&self) -> &Box<dyn Logger> {
        &self.logger
    }

    pub fn info(&self, message: String) {
        (*self.logger).info(message);
    }

    pub fn warn(&self, message: String) {
        (*self.logger).warn(message);
    }

    pub fn error(&self, message: String) {
        (*self.logger).error(message);
    }

    pub fn stdout(&self, message: String) {
        (*self.logger).stdout(message);
    }
}