use crate::node::Node;
use crate::optimizations::*;
use crate::parser::Parser;
use crate::writer::SVGWriter;
use anyhow::Error;
use anyhow::Result;
use std::ffi::OsString;
use std::fs::File;
use std::io::BufReader;
use std::path::Path;
use std::path::PathBuf;

/// Program that optimizes the size of SVG files.
#[derive(clap::Parser)]
#[command(version)]
pub struct Optimizer {
    /// Remove all comments.
    #[arg(long)]
    remove_comments: bool,

    /// Remove useless groups.
    ///
    /// A group is considered useless if it contains a single node or no nodes.
    #[arg(long)]
    remove_useless_groups: bool,

    /// Convert ellipses to circles if their `rx` and `ry` are equal.
    #[arg(long)]
    ellipsis_to_circles: bool,

    /// Shorten id names.
    ///
    /// Convert id names to be as short as possible. New names will always be created from latin alphabet letters and digits.
    #[arg(long)]
    shorten_ids: bool,

    /// Remove excess whitespace from attributes.
    #[arg(long)]
    remove_attr_whitespace: bool,

    /// Names of the files to optimize.
    #[arg(num_args = 1..)]
    file_names: Vec<PathBuf>,

    /// Names of the output files.
    ///
    /// If given, must have the same length as file_names. Default output file names are opt_{original_filename}.
    #[arg(short, long, num_args = 1..)]
    output_file_names: Vec<PathBuf>,
}

impl Optimizer {
    fn apply_optimizations_on_nodes(&self, nodes: Vec<Node>) -> Result<Vec<Node>> {
        let mut nodes = nodes;

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

        Ok(nodes)
    }

    fn get_output_path(input_path: &Path, output_path_arg: Option<&Path>) -> PathBuf {
        match output_path_arg {
            Some(path) => path.to_path_buf(),
            None => {
                let mut output_file_name = OsString::from("opt_");
                output_file_name.push(input_path.file_name().unwrap());
                input_path.with_file_name(output_file_name)
            }
        }
    }

    fn optimize_file(&self, input_path: &Path, output_path_arg: Option<&Path>) -> Result<()> {
        let file = File::open(input_path)?;
        let file = BufReader::new(file);
        let mut parser = Parser::new(file)?;

        let nodes = parser.parse_document()?;
        let optimized = self.apply_optimizations_on_nodes(nodes)?;

        let output_path = Self::get_output_path(input_path, output_path_arg);
        let output_file = File::create(output_path)?;
        SVGWriter::new(output_file).write(optimized)?;

        Ok(())
    }

    fn validate_args(&self) -> Result<()> {
        if self.file_names.is_empty() {
            return Err(Error::msg("There must be at least one input file path"));
        }

        if !self.output_file_names.is_empty()
            && self.output_file_names.len() != self.file_names.len()
        {
            return Err(Error::msg(
                "There must be the same amount of output file paths and input file paths",
            ));
        }
        Ok(())
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
            "--remove-attr-whitespace",
            "somedir/xd.svg",
            "--remove-useless-groups",
            "abcd.svg",
        ])?;

        assert_equal(
            optimizer.file_names.iter().map(|file| file.as_os_str()),
            vec!["abc.svg", "somedir/xd.svg", "abcd.svg"],
        );
        assert!(optimizer.output_file_names.is_empty());
        assert!(!optimizer.remove_comments);
        assert!(optimizer.remove_attr_whitespace);
        assert!(optimizer.remove_useless_groups);

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
            "--remove-comments",
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
        assert!(optimizer.remove_comments);
        assert!(!optimizer.remove_attr_whitespace);
        assert!(!optimizer.remove_useless_groups);

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
            "--remove-comments",
            "abcd.svg",
        ])?;

        let result = optimizer.optimize();

        assert!(result.is_err());
        Ok(())
    }
}
