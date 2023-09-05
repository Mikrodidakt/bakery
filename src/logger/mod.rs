use std::fmt::Arguments;

pub struct BLogger;

impl BLogger {
    pub fn info(args: Arguments) {
        println!("INFO: {}", args);
    }

    pub fn warn(args: Arguments) {
        println!("WARN: {}", args);
    }

    pub fn error(args: Arguments) {
        eprintln!("ERR: {}", args);
    }
}