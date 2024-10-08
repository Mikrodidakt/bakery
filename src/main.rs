mod cli;
mod collector;
mod commands;
mod configs;
mod data;
mod error;
mod executers;
mod fs;
mod helper;
mod workspace;

use crate::cli::bakery::Bakery;

fn main() {
    let bakery: Bakery = Bakery::new();
    bakery.bake();
}
