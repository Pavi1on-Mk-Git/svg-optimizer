pub mod errors;
pub mod parser;

use errors::ParserError;

/// SVG file optimizer
#[derive(clap::Parser)]
#[command(version, about)]
pub struct Optimizer {
    /// Names of the files to optimize
    file_names: Vec<String>,
}

impl Optimizer {
    fn apply_optimizations(&self, file_name: &str) -> Result<(), ParserError> {
        let mut file = String::new();
        let svg_source = svg::open(file_name, &mut file)?;

        let mut parser = parser::Parser::new(svg_source);
        let document = parser.parse_document()?;

        svg::save(file_name, &document)?;
        Ok(())
    }

    pub fn optimize(&self) {
        for file_name in self.file_names.iter() {
            if let Err(opt_error) = self.apply_optimizations(file_name.as_str()) {
                println!("An error has occurred: {}", opt_error);
            }
        }
    }
}
