use std::collections::HashMap;
use crate::commands::Command;

use super::get_supported_cmds;

pub struct CmdHandler {
    _cmds: HashMap<&'static str, Box<dyn Command>>,
}

impl CmdHandler {
    pub fn new() -> Self {
        CmdHandler {
            _cmds: get_supported_cmds(),
        }
    }

    pub fn get_cmd(&self, cmd_str: &str) -> Result<&Box<dyn Command>, &'static str> {
        match self._cmds.get(cmd_str) {
            Some(command) => Ok(command),
            None => Err("Invalid command"),
        }
    }
}