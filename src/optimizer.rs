use xml::EventWriter;

use crate::errors::ParserError;
use crate::parser::Parser;
use std::ffi::OsString;
use std::fs::File;
use std::io::BufReader;
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
        let file = File::open(file_path)?;
        let file = BufReader::new(file);
        let mut parser = Parser::new(file)?;

        let nodes = parser.parse_document()?;

        let new_file_name = {
            let mut new_file_string = OsString::from("opt_");
            new_file_string.push(file_path.file_name().unwrap());
            new_file_string
        };

        let new_file = File::create(new_file_name)?;
        let mut writer = EventWriter::new(new_file);
        nodes.into_iter().try_for_each(|node| {
            node.into_iter()
                .try_for_each(|event| writer.write(event.as_writer_event().unwrap()))
        })?;

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
