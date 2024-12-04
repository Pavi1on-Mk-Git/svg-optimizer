use clap::Parser;
use svg_optimizer::Optimizer;

fn main() {
    let optimizer = Optimizer::parse();
    optimizer.optimize();
}
