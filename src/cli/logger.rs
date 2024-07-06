use mockall::*;

#[automock]
pub trait Logger {
    fn info(&self, message: String);

    fn warn(&self, message: String);

    fn error(&self, message: String);

    fn stdout(&self, message: String);

    fn debug(&self, message: String);
}

pub struct BLogger {}

impl BLogger {
    pub fn new() -> Self {
        BLogger {}
    }
}

impl Logger for BLogger {
    fn info(&self, message: String) {
        self.stdout(format!("INFO: {}", message));
    }

    fn warn(&self, message: String) {
        println!("WARN: {}", message);
    }

    fn error(&self, message: String) {
        eprintln!("ERROR: {}", message);
    }

    fn debug(&self, message: String) {
        println!("DEBUG: {}", message);
    }

    fn stdout(&self, message: String) {
        println!("{}", message);
    }
}



