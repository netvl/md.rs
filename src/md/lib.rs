#![crate_id = "md#0.1"]
#![crate_type = "rlib"]
#![feature(struct_variant)]

use std::io;

pub mod tokens;

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
    ParseError {
        message: &'static str
    },

    IoError {
        cause: io::IoError
    }
}

pub struct MarkdownParser<B> {
    buffer: B
}

impl<B: Buffer> MarkdownParser<B> {
    pub fn new(buffer: B) -> MarkdownParser<B> {
        MarkdownParser {
            buffer: buffer
        }
    }
    
    pub fn tokens(self) -> MarkdownTokens<B> {
        MarkdownTokens { parser: self }
    }
}

pub struct MarkdownTokens<B> {
    parser: MarkdownParser<B>
}

impl<B: Buffer> Iterator<tokens::TopLevel> for MarkdownTokens<B> {
    #[inline]
    fn next(&mut self) -> Option<tokens::TopLevel> {
        self.parser.next().to_result().ok()
    }
}

impl<B: Buffer> MarkdownParser<B> {
    pub fn read_while_possible(&mut self) -> (Vec<tokens::TopLevel>, Option<MarkdownError>) {
        let mut result = Vec::new();
        let mut error = None;
        loop {
            match self.next() {
                Success(token) => result.push(token),
                PartialSuccess(token, err) => {
                    result.push(token);
                    error = Some(err);
                    break;
                }
                Failure(IoError { ref cause }) if cause.kind == io::EndOfFile => break,
                Failure(err) => {
                    error = Some(err);
                    break;
                }
            }
        }
        (result, error)
    }

    pub fn next(&mut self) -> MarkdownResult<tokens::TopLevel> {
        unimplemented!()
    }
}
