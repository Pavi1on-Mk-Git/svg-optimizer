use std::error::Error;
use std::fmt;
use std::io;

#[derive(Debug)]
pub struct ParserError {
    information: String,
}

impl ParserError {
    pub fn new(information: &str) -> Self {
        ParserError {
            information: information.to_owned(),
        }
    }
}

impl Error for ParserError {}

impl fmt::Display for ParserError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Parsing went wrong. Reason: {}", self.information)
    }
}

impl From<io::Error> for ParserError {
    fn from(_value: io::Error) -> Self {
        ParserError::new("An IO error has happened")
    }
}
