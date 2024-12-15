use std::error::Error;
use std::fmt;
use std::io;
use svg::parser;

#[derive(Debug)]
pub enum ParserError {
    MissingEndTag { tag_type: String },
    RepeatedOnceTag,
    UnexpectedContent,
    IOError { description: String },
    FileFormatError { description: String },
}

impl Error for ParserError {}

impl fmt::Display for ParserError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Self::MissingEndTag { tag_type } => write!(f, "Missing {} end tag", tag_type),
            Self::RepeatedOnceTag => write!(f, "Repeated a tag which can appear only once"),
            Self::UnexpectedContent => write!(f, "Unexpected content inside tag"),
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
