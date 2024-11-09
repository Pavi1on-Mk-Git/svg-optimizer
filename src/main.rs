use clap::Parser;
use svg_optimizer::Optimizer;
fn main() {
    let optimizer = Optimizer::parse();
    optimizer.optimize();
}

#[test]
fn testt() {
    assert_eq!(1, 1);
}
