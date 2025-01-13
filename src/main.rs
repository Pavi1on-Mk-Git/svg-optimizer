mod errors;
mod node;
mod optimizations;
mod optimizer;
mod parser;
mod writer;

use clap::Parser;
use optimizer::Optimizer;
use std::process::exit;

fn main() {
    let optimizer = Optimizer::parse();
    if let Err(opt_error) = optimizer.optimize() {
        println!("{}", opt_error);
        exit(1);
    }
}
