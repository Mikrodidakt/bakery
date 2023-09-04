mod cli;
mod commands;
mod docker;
mod workspace;
mod error;

use crate::cli::bakery::Bakery;

fn main() {
    let bakery: Bakery = Bakery::new();
    bakery.bake()
}
