use std::io;
use std::fmt;
use std::str::SendStr;

pub type MarkdownResult<T> = Result<T, MarkdownError>;

pub enum MarkdownError {
    ParseError(SendStr),
    EndOfDocument
}

impl fmt::Show for MarkdownError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            ParseError(ref msg) => write!(f, "parse error: {}", msg.as_slice())
        }
    }
}
