mod cli;
mod commands;

use crate::cli::bakery::Bakery;
use std::env;

fn main() {
    Bakery::bake(env::args().collect())
}
