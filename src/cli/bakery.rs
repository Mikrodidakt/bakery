use crate::commands::{CmdHandler, BCommand};
use std::process;
use clap::Command;
pub struct Bakery {
    _cli_matches: clap::ArgMatches,
    _cmdhandler: CmdHandler,
}

impl Bakery {
    pub fn new() -> Self {
        /*
            TODO: We should try and use command! macro in clap so
            the about, author and version can be read out from the
            Cargo.toml
        */
        let cli = Command::new("bakery")
            .version("0.0.1")
            .subcommand_required(true)
            .arg_required_else_help(true)
            .about("Build engine for the Yocto/OE")
            .author("bakery by Mikrodidakt(mikro.io)");
        let cmdhandler = CmdHandler::new();
        /*
            We clone the cli because it is owned by the bakery.
            I am not sure if this is the right way have not fully
            figured out how you are supposed to do things yet in Rust
            feels a bit inefficiant but I cannot find a nother way.
            I am using clap a bit differently because I wanted the
            individual bakery commands to handle their own subcommand
            in clap. The build_cli will go over all supported commands
            and get the clap::Command clone it and add it to the cli by
            calling clap::Command subcommand method. The clap::Command
            subcommand method can be found here
            https://docs.rs/clap_builder/4.4.2/src/clap_builder/builder/command.rs.html#440
            and it will take ownership of the clap::Command
            pub fn subcommand(self, subcmd: impl Into<Command>) -> Self {...}
            and return Self. Because it takes ownership of the clap::Command
            ,in our case the cli, we have to clone cli and then we call
            the get_matches() on it.
        */
        let matches = cmdhandler.build_cli(cli.clone()).get_matches();
        Bakery {
            _cli_matches: matches,
            _cmdhandler: cmdhandler,
        }
    }

    pub fn bake(&self) {
        let cmd_name = self._cli_matches.subcommand_name();
        let cmd: Result<&Box<dyn BCommand>, &'static str> = self._cmdhandler.get_cmd(cmd_name.unwrap());

        match cmd {
            Ok(command) => {
                // Use the command object as needed
                command.execute();
            }
            Err(err_msg) => {
                println!("Error: {}", err_msg);
            }
        }
        /*let cmd_handler: CmdHandler = CmdHandler::new();
        let cmd: Result<&Box<dyn BCommand>, &'static str> = cmd_handler.get_cmd("build");

        match cmd {
            Ok(command) => {
                // Use the command object as needed
                command.execute();
            }
            Err(err_msg) => {
                println!("Error: {}", err_msg);
            }
        }*/
        process::exit(0);
    }
}