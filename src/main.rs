mod errors;
mod node;
mod optimizations;
mod optimizer;
mod parser;
mod writer;

use anyhow::Result;
use clap::Parser;
use optimizer::Optimizer;

fn main() -> Result<()> {
    Optimizer::parse().optimize()?;
    Ok(())
}
