use std::error::Error;
use std::fmt;
use std::io;

#[derive(Debug)]
pub enum ParserError {
    MissingSVGStart,
    MissingEndTag { tag_type: String },
    IOError,
}

impl Error for ParserError {}

impl fmt::Display for ParserError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Self::MissingSVGStart => write!(f, "Missing SVG start tag"),
            Self::MissingEndTag { tag_type } => write!(f, "Missing {} end tag", tag_type),
            Self::IOError => write!(f, "An IO Error has happened"),
        }
    }
}

impl From<io::Error> for ParserError {
    fn from(_value: io::Error) -> Self {
        Self::IOError
    }
}
