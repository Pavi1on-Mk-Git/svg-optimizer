use std::error::Error;
use std::fmt;
use std::io;
use svg::parser;

#[derive(Debug)]
pub enum ParserError {
    MissingSVGStart,
    MissingEndTag { tag_type: String },
    UnexpectedText,
    IOError { description: String },
    FileFormatError { description: String },
}

impl Error for ParserError {}

impl fmt::Display for ParserError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Self::MissingSVGStart => write!(f, "Missing SVG start tag"),
            Self::MissingEndTag { tag_type } => write!(f, "Missing {} end tag", tag_type),
            Self::UnexpectedText => write!(f, "Unexpected text before SVG tag"),
            Self::IOError { description } => {
                write!(f, "An IO Error has happened. Reason: {}", description)
            }
            Self::FileFormatError { description } => write!(
                f,
                "A file format error has happened. Reason: {}",
                description
            ),
        }
    }
}

impl From<io::Error> for ParserError {
    fn from(value: io::Error) -> Self {
        Self::IOError {
            description: value.to_string(),
        }
    }
}

impl From<&parser::Error> for ParserError {
    fn from(value: &parser::Error) -> Self {
        Self::FileFormatError {
            description: value.to_string(),
        }
    }
}
