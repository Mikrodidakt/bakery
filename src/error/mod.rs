use std::fmt;

#[derive(Debug, PartialEq)] // derive std::fmt::Debug on BError
pub struct BError {
    pub code: usize,
    pub message: String,
}

impl fmt::Display for BError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let err_msg = match self.code {
            1 => format!("Task failed trying to run '{}'", self.message),
            _ => format!("{}", self.message),
        };

        write!(f, "{}", err_msg)
    }
}