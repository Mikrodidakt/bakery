use crate::commands::{CmdHandler, BCommand};
use crate::error::BError;
use crate::cli::Logger;

pub struct Cli {
    _cmd_handler: CmdHandler,
    pub logger: Box<dyn Logger>,
}

impl Cli {
    pub fn new(logger: Box<dyn Logger>) -> Self {
        let cmd_handler = CmdHandler::new();
        Cli {
            _cmd_handler: cmd_handler,
            logger: logger,
        } 
    }

    pub fn get_command(&self, cmd_name: String) -> Result<&Box<dyn BCommand>, BError> {
        self._cmd_handler.get_cmd(&cmd_name)
    }

    pub fn build_cli(&self, cmd: &clap::Command) -> clap::ArgMatches {
        self._cmd_handler.build_cli(cmd.clone()).get_matches()
    }

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
}