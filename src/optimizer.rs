use crate::errors::ParserError;
use crate::optimizations::*;
use crate::parser::Parser;
use crate::writer::SVGWriter;
use std::fs::File;
use std::io::BufReader;
use std::path::Path;
use std::path::PathBuf;

/// Program that optimizes the size of SVG files.
#[derive(clap::Parser)]
#[command(version)]
pub struct Optimizer {
    /// Remove all comments
    #[arg(long)]
    remove_comments: bool,

    /// Remove useless groups; a group is considered useless if it contains a single node or no nodes
    #[arg(long)]
    remove_useless_groups: bool,

    /// Convert ellipsis to circles if their `rx` and `ry` are equal
    #[arg(long)]
    ellipsis_to_circles: bool,

    /// Convert id names to be as short as possible created from latin alphabet letters and digits
    #[arg(long)]
    shorten_ids: bool,

    /// Remove excess whitespace from attributes
    #[arg(long)]
    remove_attr_whitespace: bool,

    /// Names of the files to optimize
    file_names: Vec<PathBuf>,

    /// Names of the output files for each input file. Defaults are opt_{original_filename}
    #[arg(short, long)]
    output_file_names: Vec<PathBuf>,
}

impl Optimizer {
    fn optimize_file(&self, input_path: &Path, output_path: &Path) -> Result<(), ParserError> {
        let file = File::open(input_path)?;
        let file = BufReader::new(file);
        let mut parser = Parser::new(file)?;

        let mut nodes = parser.parse_document()?;

        macro_rules! apply_optimizations {
            ($($optimization:ident),*) => {
                $(
                    if self.$optimization {
                        nodes = $optimization(nodes);
                    }
                )*
            };
        }

        apply_optimizations!(
            remove_comments,
            remove_useless_groups,
            ellipsis_to_circles,
            shorten_ids,
            remove_attr_whitespace
        );

        let output_file = File::create(output_path)?;
        SVGWriter::new(output_file).write(nodes)?;

        Ok(())
    }

    pub fn optimize(&self) {
        for (input_path, output_path) in self.file_names.iter().zip(self.output_file_names.iter()) {
            if let Err(opt_error) = self.optimize_file(input_path, output_path) {
                println!("An error has occurred: {}", opt_error);
            }
        }
    }
}
