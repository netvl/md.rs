#![crate_id = "md#0.1"]
#![crate_type = "rlib"]
#![feature(struct_variant, globs, macro_rules)]

use std::io;

pub use result::*;
pub use tokens::*;

pub mod tokens;
pub mod result;

macro_rules! parse_error(
    ($s:expr) => (Failure(ParseError(std::str::Slice($s))));
    ($s:expr, $($arg:expr),+) => (Failure(ParseError(std::str::Owned(format!($s, $($arg),+)))))
)

struct Buf {
    data: Vec<u8>,
    pos: uint
}

impl Buf {
    #[inline]
    fn new() -> Buf {
        Buf { data: Vec::new(), pos: 0 }
    }

    fn read_from<R: Reader>(&mut self, source: R) -> io::IoResult<u8> {
        if self.pos < self.data.len() {
            let ch = *self.buf.get(self.pos);
            self.pos += 1;
            Ok(ch)
        } else {
            source.read_byte().map(|c| { 
                self.buf.push(c);
                self.pos += 1;
                c
            })
        }
    }

    fn rewind(&mut self) {
        self.pos = 0;
    }
    
    fn consume(&mut self) {
        self.buf.clear();
        self.pos = 0;
    }
}

struct BufferStack {
    bufs: Vec<Buf>
}

impl Buf {
    #[inline]
    fn new() -> BufferStack { BufferStack { bufs: Vec::new() } }

    fn push_new(&mut self) {
        self.bufs.push(Buf::new());
    }

    fn pop(&mut self) {
        assert!(self.bufs.len() > 0);
        self.bufs.pop();
    }

    fn rewind(&mut self) {
        assert!(self.bufs.len() > 0);
        self.bufs.last_mut().unwrap().rewind();
    }

    fn consume(&mut self) {
        assert!(self.bufs.len() > 0);
        self.bufs.last_mut().unwrap().consume();
    }

    fn read_from<R: Reader>(&mut self, source: R) -> io::IoResult<u8> {
        assert!(self.bufs.len() > 0);
        self.bufs.last_mut().unwrap().read_from(source)
    }
}

pub struct MarkdownParser<R> {
    source: R,
    stack: BufferStack
}

impl<R: Reader> MarkdownParser<R> {
    pub fn new(re: B) -> MarkdownParser<B> {
        MarkdownParser {
            source: buffer,

            buf: Vec::new(),
            pos: 0
        }
    }
    
    pub fn tokens(self) -> MarkdownTokens<R> {
        MarkdownTokens { parser: self }
    }
}

pub struct MarkdownTokens<R> {
    parser: MarkdownParser<R>
}

impl<R: Reader> Iterator<Block> for MarkdownTokens<R> {
    #[inline]
    fn next(&mut self) -> Option<Block> {
        self.parser.next().to_result().ok()
    }
}

macro_rules! first_of(
    ($e:expr) => ($e);
    ($e:expr or $($more:expr),+) => (
        $e.or_else(|| try_parse!($($more),+))
    )
)

macro_rules! try_(
    ($e:expr) => (
        match $e {
            Some(result) => Some(result),
            None => { self.backtrack(); None }
        }
    )
)

impl<R: Reader> MarkdownParser<R> {
    pub fn read_while_possible(&mut self) -> (Document, Option<MarkdownError>) {
        let mut result = Vec::new();
        let mut error = None;
        loop {
            match self.next() {
                Success(token) => result.push(token),
                PartialSuccess(token, err) => {
                    result.push(token);
                    return (result, Some(err));
                }
                Failure(IoError(ref cause)) if cause.kind == io::EndOfFile => break,
                Failure(err) => return (result, Some(err))
            }
        }
        (result, None)
    }

    pub fn next(&mut self) -> MarkdownResult<Block> {
        self.parse_block()
    }
    
    fn push_buf(&mut self) {
        self.stack.push_new();
    }
    
    fn pop_buf(&mut self) {
        self.stack.pop();
    }

    fn read_byte(&mut self) -> io::IoResult<char> {
        self.stack.read_from(&mut self.source)
    }

    fn consume(&mut self) {
        self.stack.consume();
    }

    fn backtrack(&mut self) {
        self.stack.backtrac();
    }
    
    fn parse_block(&mut self) -> MarkdownResult<Block> {
        let block = first_of! {
            self.block_quote() or
            self.block_code() or
            self.horizontal_rule() or
            self.heading() or
            self.ordered_list() or
            self.unordered_list() or
            self.paragraph()
        };
        block.or(|| parse_error("a block"))
    }

    fn block_quote(&mut self) -> Option<MarkdownResult<Block>> {
        let mut blocks = Vec::new();
        let mut lines = Vec::new();
        loop {
            try_!(self.skip_initial_spaces());
            match self.skip_char('>') {
            }
        }
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


    fn parse_error(&mut self, what: &str) -> Option<MarkdownResult<Block>> {
        Some(parse_error!("unable to parse {}, buffer contents: {}", what, self.buf))
    }
}
