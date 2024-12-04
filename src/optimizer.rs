use crate::errors::ParserError;
use crate::parser::Parser;
use std::path::Path;

/// SVG file optimizer. Currently, saves the output files as opt_{original_filename}.
#[derive(clap::Parser)]
#[command(version, about)]
pub struct Optimizer {
    /// Names of the files to optimize
    file_names: Vec<String>,
}

impl Optimizer {
    fn apply_optimizations(&self, file_path: &str) -> Result<(), ParserError> {
        let input_path = Path::new(file_path);

        let mut file = String::new();
        let svg_source = svg::open(input_path, &mut file)?;

        let mut parser = Parser::new(svg_source);
        let document = parser.parse_document()?;

        svg::save(
            format!("opt_{}", input_path.file_name().unwrap().to_str().unwrap()),
            &document,
        )?;
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
