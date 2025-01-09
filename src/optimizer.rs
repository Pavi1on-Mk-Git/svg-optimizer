use crate::errors::ParserError;
use crate::optimizations::*;
use crate::parser::Parser;
use std::ffi::OsString;
use std::fs::File;
use std::io::BufReader;
use std::path::{Path, PathBuf};
use xml::EventWriter;

/// SVG file optimizer. Currently, saves the output files as opt_{original_filename}.
#[derive(clap::Parser)]
#[command(version)]
pub struct Optimizer {
    /// Remove all comments from files
    #[arg(long)]
    remove_comments: bool,

    /// Remove useless groups
    /// A group is considered useless if it contains a single node
    #[arg(long)]
    remove_useless_groups: bool,

    /// Convert ellipsis to circles if their `rx` and `ry` are equal
    #[arg(long)]
    ellipsis_to_circles: bool,

    /// Convert ids to shortest possible
    #[arg(long)]
    shorten_ids: bool,

    /// Names of the files to optimize
    file_names: Vec<PathBuf>,
}

impl Optimizer {
    fn apply_optimizations(&self, file_path: &Path) -> Result<(), ParserError> {
        let file = File::open(file_path)?;
        let file = BufReader::new(file);
        let mut parser = Parser::new(file)?;

        let mut nodes = parser.parse_document()?;

        if self.remove_comments {
            nodes = remove_comments(nodes);
        }

        if self.remove_useless_groups {
            nodes = remove_useless_groups(nodes);
        }

        if self.ellipsis_to_circles {
            nodes = ellipsis_to_circles(nodes);
        }

        if self.shorten_ids {
            nodes = shorten_ids(nodes);
        }

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
