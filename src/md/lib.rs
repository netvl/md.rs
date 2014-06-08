#![crate_id = "md#0.1"]
#![crate_type = "rlib"]
#![feature(struct_variant, globs, macro_rules)]

use std::io;

pub use result::*;
pub use tokens::*;

pub mod tokens;
pub mod result;

pub struct MarkdownParser<B> {
    source: B,

    buf: Vec<char>,
    pos: uint
}

impl<B: Buffer> MarkdownParser<B> {
    pub fn new(buffer: B) -> MarkdownParser<B> {
        MarkdownParser {
            source: buffer,

            buf: Vec::new(),
            pos: 0
        }
    }
    
    pub fn tokens(self) -> MarkdownTokens<B> {
        MarkdownTokens { parser: self }
    }
}

pub struct MarkdownTokens<B> {
    parser: MarkdownParser<B>
}

impl<B: Buffer> Iterator<tokens::Block> for MarkdownTokens<B> {
    #[inline]
    fn next(&mut self) -> Option<tokens::Block> {
        self.parser.next().to_result().ok()
    }
}

macro_rules! try_parse(
    ($f:ident) => (self.$f().unwrap());
    ($f:ident $($more:ident)+) => (
        match self.$f() {
            Some(result) => result,
            None => try_parse!($($more)+)
        }
    )
)

impl<B: Buffer> MarkdownParser<B> {
    pub fn read_while_possible(&mut self) -> (Document, Option<MarkdownError>) {
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
                Failure(IoError(ref cause)) if cause.kind == io::EndOfFile => break,
                Failure(err) => {
                    error = Some(err);
                    break;
                }
            }
        }
        (result, error)
    }

    pub fn next(&mut self) -> MarkdownResult<Block> {
        self.parse_block()
    }

    fn read_char(&mut self) -> io::IoResult<()> {
        self.source.read_char().map(|c| self.buf.push(c))
    }
    
    fn parse_block(&mut self) -> MarkdownResult<Block> {
        try_parse! {
            block_quote
            block_code
            horizontal_rule
            heading
            ordered_list
            unordered_list
            paragraph
            parse_error
        }
    }

    fn block_quote(&mut self) -> Option<MarkdownResult<Block>> {
        None
    }


    fn block_code(&mut self) -> Option<MarkdownResult<Block>> {
        None
    }


    fn horizontal_rule(&mut self) -> Option<MarkdownResult<Block>> {
        None
    }


    fn heading(&mut self) -> Option<MarkdownResult<Block>> {
        None
    }


    fn ordered_list(&mut self) -> Option<MarkdownResult<Block>> {
        None
    }


    fn unordered_list(&mut self) -> Option<MarkdownResult<Block>> {
        None
    }


    fn paragraph(&mut self) -> Option<MarkdownResult<Block>> {
        None
    }


    fn parse_error(&mut self) -> Option<MarkdownResult<Block>> {
        None
    }
}
