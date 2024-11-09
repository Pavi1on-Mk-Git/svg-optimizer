use clap::Parser;
use std::io::Error;
use svg::node::element::tag::{Type, SVG};
use svg::parser::Event;
use svg::Document;

/// SVG file optimizer
#[derive(Parser)]
#[command(version, about)]
pub struct Optimizer {
    /// Names of the files to optimize
    file_names: Vec<String>,
}

impl Optimizer {
    fn apply_optimizations(&self, file_name: &str) -> Result<(), Error> {
        let mut file = String::new();

        let mut document: Document = Document::new();

        for event in svg::open(file_name, &mut file)? {
            match event {
                Event::Tag(SVG, Type::Start, attributes) => {
                    for (name, val) in attributes {
                        document = document.set(name, val);
                    }
                }
                Event::Tag(SVG, Type::End, _) => {
                    break;
                }
                _ => {}
            }
        }

        svg::save(file_name, &document)?;

        Ok(())
    }

    pub fn optimize(&self) {
        for file_name in self.file_names.iter() {
            if let Err(opt_error) = self.apply_optimizations(file_name.as_str()) {
                println!("An error occurred: {:?}", opt_error);
            }
        }
    }
}
