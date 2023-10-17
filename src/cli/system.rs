use mockall::*;
use std::collections::HashMap;

use crate::error::BError;

/*
Tried using "withf" and closure when mocking the check_call for testing 
but it did not work when using multiple expectations not sure why but suspect
that it had something todo with that it could not determine which
closure to use because it would always use the first closure which always
resulted in a failed test. Instead we switched to "with" and predicate::eq.
Using the predicate::eq instead required that we use a struct wrapper
for the params that inherits the PartialEq trait. Not ideal but it works
so for now we will use this solution.
*/
#[derive(Debug, PartialEq)]
pub struct CallParams {
    pub cmd_line: Vec<String>,
    pub env: HashMap<String, String>,
    pub shell: bool,
}

#[automock]
pub trait System {
    fn check_call(&self, params: &CallParams) -> Result<(), BError>;
}

pub struct BSystem {}

impl BSystem {
    pub fn new() -> Self {
        BSystem {}
    }
}

impl System for BSystem {
    fn check_call(&self, params: &CallParams) -> Result<(), BError> {
        Ok(())
    }
}