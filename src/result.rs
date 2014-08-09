use std::io;
use std::fmt;
use std::str::SendStr;

pub enum PartialResult<T, E1, E2> {
    Success(T),
    PartialSuccess(T, E1),
    Failure(E2)
}

impl<T, E1, E2> PartialResult<T, E1, E2> {
    pub fn to_result(self) -> Result<T, E2> {
        match self {
            Success(value) | PartialSuccess(value, _) => Ok(value),
            Failure(e) => Err(e)
        }
    }
}

pub type MarkdownResult<T> = PartialResult<T, MarkdownError, MarkdownError>;

pub enum MarkdownError {
    ParseError(SendStr),
    IoError(io::IoError)
}

impl fmt::Show for MarkdownError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            ParseError(ref msg) => write!(f, "parse error: {}", msg),
            IoError(ref err) => write!(f, "i/o error: {}", err)
        }
    }
}

impl MarkdownError {
    #[inline]
    pub fn from_io(err: io::IoError) -> MarkdownError {
         IoError(err)
    }
}

