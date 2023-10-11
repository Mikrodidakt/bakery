use mockall::*;
use std::collections::HashMap;

use crate::error::BError;

#[automock]
pub trait System {
    fn check_call(&self, cmd_line: &Vec<String>, env: &HashMap<String, String>, shell: bool) -> Result<(), BError>;
}

pub struct BSystem {}

impl BSystem {
    pub fn new() -> Self {
        BSystem {}
    }
}

impl System for BSystem {
    fn check_call(&self, cmd_line: &Vec<String>, env: &HashMap<String, String>, shell: bool) -> Result<(), BError> {
        Ok(())
    }
}