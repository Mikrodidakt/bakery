mod cli;
mod commands;
mod workspace;
mod error;
mod executers;
mod configs;
mod helper;

use crate::cli::bakery::Bakery;

fn main() {
    let bakery: Bakery = Bakery::new();
    bakery.bake()
}
