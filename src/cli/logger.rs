use mockall::*;

#[automock]
pub trait Logger {
    fn info(&self, message: String);

    fn warn(&self, message: String);

    fn error(&self, message: String);
}

pub struct BLogger {}

impl BLogger {
    pub fn new() -> Self {
        BLogger {}
    }
}

impl Logger for BLogger {
    fn info(&self, message: String) {
        println!("INFO: {}", message);
    }

    fn warn(&self, message: String) {
        println!("WARN: {}", message);
    }

    fn error(&self, message: String) {
        eprintln!("ERR: {}", message);
    }    
}



