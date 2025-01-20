use crate::optimizations::*;
use crate::parser::Parser;
use crate::writer::SVGWriter;
use anyhow::{Error, Result};
use std::ffi::OsString;
use std::fs::File;
use std::io::BufReader;
use std::path::{Path, PathBuf};

/// Program that optimizes the size of SVG files.
///
/// By default, all optimizations that do not take parameters are enabled.
/// See README for descriptions of each optimization.
#[derive(clap::Parser)]
#[command(version)]
pub struct Optimizer {
    /// Names of the files to optimize.
    #[arg(num_args = 1..)]
    file_names: Vec<PathBuf>,

    /// Names of the output files.
    ///
    /// If given, must have the same length as file_names. Default output file names are opt_{original_filename}.
    #[arg(short, long, num_args = 1..)]
    output_file_names: Vec<PathBuf>,

    /// Disable all optimizations by default.
    #[arg(short, long)]
    disable_by_default: bool,

    #[command(flatten)]
    optimizations: Optimizations,
}

impl Optimizer {
    fn get_output_path(input_path: &Path, output_path_arg: Option<&Path>) -> Result<PathBuf> {
        match output_path_arg {
            Some(path) => Ok(path.to_path_buf()),
            None => {
                let mut output_file_name = OsString::from("opt_");

                if let Some(file_name) = input_path.file_name() {
                    output_file_name.push(file_name);
                    Ok(input_path.with_file_name(output_file_name))
                } else {
                    Err(Error::msg(format!(
                        "Invalid input path given: {}",
                        input_path.as_os_str().to_str().unwrap_or("invalid unicode")
                    )))
                }
            }
        }
    }

    fn optimize_file(&self, input_path: &Path, output_path_arg: Option<&Path>) -> Result<()> {
        let file = File::open(input_path)?;
        let file = BufReader::new(file);
        let mut parser = Parser::new(file)?;

        let nodes = parser.parse_document()?;
        let optimized = self.optimizations.apply(nodes, !self.disable_by_default)?;

        let output_path = Self::get_output_path(input_path, output_path_arg)?;
        let output_file = File::create(output_path)?;
        SVGWriter::new(output_file).write(optimized)?;

        Ok(())
    }

    fn validate_args(&self) -> Result<()> {
        if self.file_names.is_empty() {
            Err(Error::msg("There must be at least one input file path"))
        } else if !self.output_file_names.is_empty()
            && self.output_file_names.len() != self.file_names.len()
        {
            Err(Error::msg(
                "There must be the same amount of output file paths and input file paths",
            ))
        } else {
            Ok(())
        }
    }

    pub fn optimize(&self) -> Result<()> {
        self.validate_args()?;

        if self.output_file_names.is_empty() {
            for input_path in self.file_names.iter() {
                self.optimize_file(input_path, None)?;
            }
        } else {
            for (input_path, output_path) in
                self.file_names.iter().zip(self.output_file_names.iter())
            {
                self.optimize_file(input_path, Some(output_path))?;
            }
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use clap::Parser;
    use itertools::assert_equal;

    #[test]
    fn test_parse_file_names_no_outputs() -> Result<()> {
        let optimizer = Optimizer::try_parse_from(vec![
            "main.exe",
            "abc.svg",
            "--no-remove-attribute-whitespace",
            "somedir/xd.svg",
            "--no-remove-useless-groups",
            "abcd.svg",
        ])?;

        assert_equal(
            optimizer.file_names.iter().map(|file| file.as_os_str()),
            vec!["abc.svg", "somedir/xd.svg", "abcd.svg"],
        );
        assert!(optimizer.output_file_names.is_empty());
        assert!(!optimizer.optimizations.no_remove_comments());
        assert!(optimizer.optimizations.no_remove_attribute_whitespace());
        assert!(optimizer.optimizations.no_remove_useless_groups());

        Ok(())
    }

    #[test]
    fn test_parse_file_names_with_outputs() -> Result<()> {
        let optimizer = Optimizer::try_parse_from(vec![
            "main.exe",
            "abc.svg",
            "somedir/xd.svg",
            "-o",
            "abc2.svg",
            "somedir_321/xd.svg",
            "abcd.svg",
            "--no-remove-comments",
            "abcd.svg",
        ])?;

        assert_equal(
            optimizer.file_names.iter().map(|file| file.as_os_str()),
            vec!["abc.svg", "somedir/xd.svg", "abcd.svg"],
        );
        assert_equal(
            optimizer
                .output_file_names
                .iter()
                .map(|file| file.as_os_str()),
            vec!["abc2.svg", "somedir_321/xd.svg", "abcd.svg"],
        );
        assert!(optimizer.optimizations.no_remove_comments());
        assert!(!optimizer.optimizations.no_remove_attribute_whitespace());
        assert!(!optimizer.optimizations.no_remove_useless_groups());

        Ok(())
    }

    #[test]
    fn test_no_input_files_validation_error() -> Result<()> {
        let optimizer = Optimizer::try_parse_from(vec!["main.exe"])?;

        let result = optimizer.optimize();

        assert!(result.is_err());
        Ok(())
    }

    #[test]
    fn test_parse_file_names_validation_error() -> Result<()> {
        let optimizer = Optimizer::try_parse_from(vec![
            "main.exe",
            "abc.svg",
            "somedir/xd.svg",
            "-o",
            "somedir_321/xd.svg",
            "abcd.svg",
            "--no-remove-comments",
            "abcd.svg",
        ])?;

        let result = optimizer.optimize();

        assert!(result.is_err());
        Ok(())
    }
}
