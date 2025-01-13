use std::error::Error;
use std::fmt;
use xml::common::TextPosition;

#[derive(Debug)]
pub struct ErrorWithPosition {
    pub message: String,
    pub position: TextPosition,
}

impl Error for ErrorWithPosition {}

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
    pub fn _new(message: String, position: TextPosition) -> Self {
        Self { message, position }
    }
}
