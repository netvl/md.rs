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

    fn read_from<R: Reader>(&mut self, source: &mut R) -> io::IoResult<u8> {
        if self.pos < self.data.len() {
            let ch = *self.data.get(self.pos);
            self.pos += 1;
            Ok(ch)
        } else {
            source.read_byte().map(|c| { 
                self.data.push(c);
                self.pos += 1;
                c
            })
        }
    }

    fn unread(&mut self, n: uint) {
        assert!(n <= self.pos);
        self.pos -= n;
    }

    fn rewind(&mut self) {
        self.pos = 0;
    }
    
    fn consume(&mut self) {
        self.data.clear();
        self.pos = 0;
    }
}

struct BufStack {
    bufs: Vec<Buf>
}

impl BufStack {
    #[inline]
    fn new() -> BufStack { BufStack { bufs: Vec::new() } }

    fn push_new(&mut self) {
        self.bufs.push(Buf::new());
    }

    #[inline]
    fn peek<'a>(&'a mut self) -> &'a mut Buf {
        assert!(self.bufs.len() > 0);
        self.bufs.mut_last().unwrap()
    }

    fn pop(&mut self) {
        assert!(self.bufs.len() > 0);
        self.bufs.pop();
    }

    fn rewind(&mut self) {
        assert!(self.bufs.len() > 0);
        self.bufs.mut_last().unwrap().rewind();
    }

    fn consume(&mut self) {
        assert!(self.bufs.len() > 0);
        self.bufs.mut_last().unwrap().consume();
    }

    fn read_from<R: Reader>(&mut self, source: &mut R) -> io::IoResult<u8> {
        assert!(self.bufs.len() > 0);
        self.bufs.mut_last().unwrap().read_from(source)
    }

    fn unread(&mut self, n: uint) {
        assert!(self.bufs.len() > 0);
        self.bufs.mut_last().unwrap().unread(n);
    }
}

pub struct MarkdownParser<R> {
    source: R,
    stack: BufStack
}

impl<R: Reader> MarkdownParser<R> {
    pub fn new(r: R) -> MarkdownParser<R> {
        MarkdownParser {
            source: r,
            stack: BufStack::new()
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
    ($e:expr or $f:expr $(or $more:expr)*) => (
        $e.or_else(|| first_of!($f $(or $more)*))
    )
)

macro_rules! try_bt(
    ($_self:ident; $e:expr) => (
        match $e {
            Some(result) => Some(result),
            None => { $_self.backtrack(); return None }
        }
    )
)

impl<R: Reader> MarkdownParser<R> {
    pub fn read_while_possible(&mut self) -> (Document, Option<MarkdownError>) {
        let mut result = Vec::new();
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

    #[inline]
    pub fn next(&mut self) -> MarkdownResult<Block> { self.parse_block() }
    
    #[inline]
    fn push_buf(&mut self) { self.stack.push_new(); }
    
    #[inline]
    fn pop_buf(&mut self) { self.stack.pop(); }

    #[inline]
    fn read_byte(&mut self) -> io::IoResult<u8> { self.stack.read_from(&mut self.source) }

    #[inline]
    fn read_byte_mr(&mut self) -> MarkdownResult<u8> { result::from_io(self.read_byte()) }
    
    #[inline]
    fn unread_bytes(&mut self, n: uint) { self.stack.unread(n); }
    
    #[inline]
    fn unread_byte(&mut self) { self.unread_bytes(1); }

    #[inline]
    fn consume(&mut self) { self.stack.consume(); }

    #[inline]
    fn backtrack(&mut self) { self.stack.rewind(); }
    
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
        block.unwrap_or_else(|| self.parse_error("a block"))
    }

    fn block_quote(&mut self) -> Option<MarkdownResult<Block>> {
        //let mut blocks = Vec::new();
        //let mut lines = Vec::new();
        loop {
            try_bt!(self; self.skip_initial_spaces());
        }
    }

    fn skip_initial_spaces(&mut self) -> Option<MarkdownResult<()>> {
        let mut n: u8 = 0;
        loop {
            if n >= 4 {
                return None;
            }
            match self.read_byte() {
                Ok(b' ') => n += 1,  // increase counter and continue
                Ok(_) => { self.unread_byte(); break },   // not a space and less than 4 spaces
                Err(e) => return Some(Failure(MarkdownError::from_io(e)))
            }
        }
        Some(Success(()))
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


    fn parse_error(&mut self, what: &str) -> MarkdownResult<Block> {
        parse_error!("unable to parse {}; current buffer contents: {}", what, self.stack.peek().data)
    }
}
