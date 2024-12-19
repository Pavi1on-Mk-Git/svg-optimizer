use std::error::Error;
use std::fmt;
use std::io;
use xml::reader;
use xml::writer;

#[derive(Debug)]
pub enum ParserError {
    IO(io::Error),
    FileReading(reader::Error),
    FileWriting(writer::Error),
}

impl Error for ParserError {}

impl fmt::Display for ParserError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Self::IO(error) => {
                write!(f, "An IO Error has happened. Reason: {}", error)
            }
            Self::FileReading(error) => {
                write!(f, "A file reading error has happened. Reason: {}", error)
            }
            Self::FileWriting(error) => {
                write!(f, "A file writing error has happened. Reason: {}", error)
            }
        }
    }
}

impl From<io::Error> for ParserError {
    fn from(value: io::Error) -> Self {
        Self::IO(value)
    }
}

impl From<reader::Error> for ParserError {
    fn from(value: reader::Error) -> Self {
        Self::FileReading(value)
    }
}

impl From<writer::Error> for ParserError {
    fn from(value: writer::Error) -> Self {
        Self::FileWriting(value)
    }
}
