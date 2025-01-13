use std::error;
use std::fmt;
use xml::common::TextPosition;

#[derive(Debug)]
pub struct SimpleError {
    pub message: String,
}

impl error::Error for SimpleError {}

impl fmt::Display for SimpleError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Error: {}", self.message)?;
        Ok(())
    }
}

impl SimpleError {
    pub fn new(message: &str) -> SimpleError {
        SimpleError {
            message: message.into(),
        }
    }
}

#[derive(Debug)]
pub struct ErrorWithPosition {
    pub message: String,
    pub position: TextPosition,
}

impl error::Error for ErrorWithPosition {}

impl fmt::Display for ErrorWithPosition {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "Error: {}:{} {}", // consistent with xml-rs errors
            self.position.row, self.position.column, self.message
        )?;
        Ok(())
    }
}

impl ErrorWithPosition {
    pub fn _new(message: String, position: TextPosition) -> ErrorWithPosition {
        ErrorWithPosition { message, position }
    }
}
