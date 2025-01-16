use std::error;
use std::fmt;
use xml::common::TextPosition;

#[derive(Debug)]
pub struct ErrorWithPosition {
    message: String,
    position: TextPosition,
}

impl error::Error for ErrorWithPosition {}

impl fmt::Display for ErrorWithPosition {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "{}:{} {}", // consistent with xml-rs errors
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
