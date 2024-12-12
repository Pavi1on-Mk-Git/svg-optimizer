use crate::errors::ParserError;
use crate::parser::{Parser, ParserResult};
use std::ffi::OsString;
use std::fs::File;
use std::io::Write;
use std::path::{Path, PathBuf};

/// SVG file optimizer. Currently, saves the output files as opt_{original_filename}.
#[derive(clap::Parser)]
#[command(version)]
pub struct Optimizer {
    /// Names of the files to optimize
    file_names: Vec<PathBuf>,
}

impl Optimizer {
    fn apply_optimizations(&self, file_path: &Path) -> Result<(), ParserError> {
        let mut read_buffer = String::new();
        let svg_source = svg::open(file_path, &mut read_buffer)?;

        let mut parser = Parser::new(svg_source);
        let ParserResult(document, pre, post) = parser.parse_document()?;

        let new_file_name = {
            let mut new_file_string = OsString::from("opt_");
            new_file_string.push(file_path.file_name().unwrap());
            new_file_string
        };

        let mut file = File::create(new_file_name)?;

        for string in pre {
            file.write_fmt(format_args!("{}\n", string))?;
        }
        file.write_fmt(format_args!("{}\n", &document.to_string()))?;
        for string in post {
            file.write_fmt(format_args!("{}\n", string))?;
        }

        Ok(())
    }

    pub fn optimize(&self) {
        for file_name in self.file_names.iter() {
            if let Err(opt_error) = self.apply_optimizations(file_name) {
                println!("An error has occurred: {}", opt_error);
            }
        }
    }
}
