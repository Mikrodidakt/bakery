use crate::commands::{CmdHandler, Command};
use std::process;
pub struct Bakery {}

impl Bakery {
    pub fn bake(args: Vec<String>) {
        let cmd_handler: CmdHandler = CmdHandler::new();
        let cmd: Result<&Box<dyn Command>, &'static str> = cmd_handler.get_cmd("build");

        match cmd {
            Ok(command) => {
                // Use the command object as needed
                command.execute();
            }
            Err(err_msg) => {
                println!("Error: {}", err_msg);
            }
        }
        process::exit(0);
    }
}