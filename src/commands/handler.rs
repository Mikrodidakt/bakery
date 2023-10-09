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
            None => Err(BError{ code: 0, message: String::from("Invalid command") }),
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