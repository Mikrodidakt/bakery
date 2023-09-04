use crate::commands::{CmdHandler, BCommand};
use crate::error::BError;

use clap::Command;
pub struct Bakery {
    _cli: clap::Command,
    _cli_matches: clap::ArgMatches,
    _cmd_handler: CmdHandler,
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
        let cmd_handler = CmdHandler::new();
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
        //cli = clap_autocomplete::add_subcommand(cli);
        let matches = cmd_handler.build_cli(cli.clone()).get_matches();
        Bakery {
            _cli: cli,
            _cli_matches: matches,
            _cmd_handler: cmd_handler,
        }
    }

    pub fn bake(&self) {
        /*
        We need to revisit this later seems like the clap_autocomplete is not fully working.
        It is adding a "complete" subcommand and then it should have added a "--print" flag
        but for some reason it is not added so when trying to generate the shell autocomplete
        file it fails because it cannot find the "--print" flag. Might be they way I am calling
        it because I cannot call it exactly as in the example.
        // Setup autocompletion in the shell for bakery
        if let Some(result) = clap_autocomplete::test_subcommand(&self._cli_matches, self._cli.clone()) {
            if let Err(err) = result {
                eprintln!("Insufficient permissions: {err}");
                std::process::exit(1);
            } else {
                std::process::exit(0);
            }
        // Execute the bakery subcommand
        } else {
        */
            let cmd_name = self._cli_matches.subcommand_name();
            let cmd: Result<&Box<dyn BCommand>, BError> = self._cmd_handler.get_cmd(cmd_name.unwrap());
    
            match cmd {
                Ok(command) => {
                    let error: Result<(), BError> = command.execute(&self._cli_matches);
                    match error {
                        Err(err) => {
                            eprintln!("{}", err);
                            std::process::exit(1);
                        }
                        Ok(()) => {}
                    }
                }
                Err(err) => {
                    eprintln!("{}", err);
                    std::process::exit(1);
                }
            }
            std::process::exit(0);
        //}
    }
}