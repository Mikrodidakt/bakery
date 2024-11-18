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

use crate::cli::yaab::Yaab;

fn main() {
    let bakery: Yaab = Yaab::new();
    bakery.bake();
}
