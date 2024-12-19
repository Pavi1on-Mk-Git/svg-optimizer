mod errors;
mod node;
mod optimizations;
mod optimizer;
mod parser;

use clap::Parser;
use optimizer::Optimizer;

fn main() {
    let optimizer = Optimizer::parse();
    optimizer.optimize();
}
