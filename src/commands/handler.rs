use std::collections::HashMap;
use crate::commands::BCommand;
use crate::error::BError;

use super::get_supported_cmds;

pub struct CmdHandler {
    cmds: HashMap<&'static str, Box<dyn BCommand>>,
}

impl CmdHandler {
    pub fn new() -> Self {
        CmdHandler {
            cmds: get_supported_cmds(),
        }
    }

    pub fn get_cmd(&self, cmd_str: &str) -> Result<&Box<dyn BCommand>, BError> {
        match self.cmds.get(cmd_str) {
            Some(command) => Ok(command),
            None => Err(BError::CmdError(String::from("Invalid command"))),
        }
    }

    pub fn build_cli(&self, mut cli: clap::Command) -> clap::Command {
        for (_, value) in self.cmds.iter() {
            /*
                We clone the clap::Command owned by the bakery Command.
                And then we transfer the ownership to cli and once all
                subcommands have been added to the cli we return it.
            */
            cli = cli.subcommand(value.subcommand().clone());
        }
        cli
    }
}

#[cfg(test)]
mod tests {
    use crate::commands::{CmdHandler, BCommand};
    use crate::error::BError;

    #[test]
    fn test_get_build_command() {
        let cmd_handler: CmdHandler = CmdHandler::new();
        let cmd: Result<&Box<dyn BCommand>, BError> = cmd_handler.get_cmd("build");

        match cmd {
            Ok(command) => {
                assert_eq!(command.cmd_str(), "build");
            }
            Err(err_msg) => {
                assert!(false, "Expected OK result, but got an error '{}'", err_msg);
            }
        }
    }

    #[test]
    fn test_get_clean_command() {
        let cmd_handler: CmdHandler = CmdHandler::new();
        let cmd: Result<&Box<dyn BCommand>, BError> = cmd_handler.get_cmd("clean");

        match cmd {
            Ok(command) => {
                assert_eq!(command.cmd_str(), "clean");
            }
            Err(err) => {
                assert!(false, "Expected OK result, but got an error '{}'", err);
            }
        }
    }

    #[test]
    fn test_get_shell_command() {
        let cmd_handler: CmdHandler = CmdHandler::new();
        let cmd: Result<&Box<dyn BCommand>, BError> = cmd_handler.get_cmd("shell");

        match cmd {
            Ok(command) => {
                assert_eq!(command.cmd_str(), "shell");
            }
            Err(err) => {
                assert!(false, "Expected OK result, but got an error '{}'", err);
            }
        }
    }

    #[test]
    fn test_get_deploy_command() {
        let cmd_handler: CmdHandler = CmdHandler::new();
        let cmd: Result<&Box<dyn BCommand>, BError> = cmd_handler.get_cmd("deploy");

        match cmd {
            Ok(command) => {
                assert_eq!(command.cmd_str(), "deploy");
            }
            Err(err) => {
                assert!(false, "Expected OK result, but got an error '{}'", err);
            }
        }
    }

    #[test]
    fn test_get_upload_command() {
        let cmd_handler: CmdHandler = CmdHandler::new();
        let cmd: Result<&Box<dyn BCommand>, BError> = cmd_handler.get_cmd("upload");

        match cmd {
            Ok(command) => {
                assert_eq!(command.cmd_str(), "upload");
            }
            Err(err) => {
                assert!(false, "Expected OK result, but got an error '{}'", err);
            }
        }
    }

    #[test]
    fn test_get_list_command() {
        let cmd_handler: CmdHandler = CmdHandler::new();
        let cmd: Result<&Box<dyn BCommand>, BError> = cmd_handler.get_cmd("list");

        match cmd {
            Ok(command) => {
                assert_eq!(command.cmd_str(), "list");
            }
            Err(err) => {
                assert!(false, "Expected OK result, but got an error '{}'", err);
            }
        }
    }

    #[test]
    fn test_get_invalid_command() {
        let cmd_handler: CmdHandler = CmdHandler::new();
        let cmd: Result<&Box<dyn BCommand>, BError> = cmd_handler.get_cmd("invalid");

        match cmd {
            Ok(command) => {
                assert!(false, "Expected an error, but got an command '{}'", command.cmd_str());
            }
            Err(err) => {
                // TODO: we should make sure that BError is using PartialEq and Eq Traits
                assert_eq!("Invalid command", err.to_string());
            }
        }
    }
}