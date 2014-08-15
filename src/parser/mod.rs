use collections::ringbuf::RingBuf;

pub use tokens::*;

use util::CellOps;

mod block;
mod inline;
mod result;

macro_rules! first_of(
    ($e:expr) => ($e);
    ($e:expr or $f:expr $(or $more:expr)*) => (
        $e.or_else(|| first_of!($f $(or $more)*))
    )
)

macro_rules! opt_ret_end(
    ($e:expr) => (
        match $e {
            None => return End,
            Some(c) => c
        }
    )
)

macro_rules! cancel(
    ($m:ident; $r:expr) => ({$m.cancel(); $r})
)

macro_rules! parse_error(
    ($s:expr) => (Failure(ParseError(std::str::Slice($s))));
    ($s:expr, $($arg:expr),+) => (Failure(ParseError(std::str::Owned(format!($s, $($arg),+)))))
)

macro_rules! try_reset(
    ($_self:ident; $e:expr) => ({
        let m = $_self.cur().mark();
        match $e {
            NoParse => return NoParse,
            End => cancel!(m; return End),
            Success(r) => cancel!(m; r),
            Failure(e) => cancel!(m; return Failure(e))
        }
    })
)

struct Cursor<'a> {
    buf: &'a [u8],
    pos: Cell<uint>
}

impl Deref<u8> for Cursor {
    #[inline]
    fn deref(&self) -> &u8 {
        &self.buf[self.pos]
    }
}

impl Cursor {
    fn new(buf: &[u8]) -> Cursor {
        Cursor {
            buf: buf,
            pos: Cell::new(0)
        }
    }

    #[inline]
    fn available(&self) -> bool { self.pos.get() < self.buf.len() }
    
    #[inline]
    fn advance(&mut self, n: uint) { self.pos.modify(|p| p + n); }

    #[inline]
    fn retract(&mut self, n: uint) { self.pos.modify(|p| if n > p { 0 } else { p - n }); }

    #[inline]
    fn next(&mut self) -> Option<u8> { 
        if self.available() {
            let r = **self; 
            self.advance(1);
            Some(r)
        } else {
            None
        }
    }

    #[inline]
    fn prev(&mut self) -> u8 { self.retract(1); **self }

    #[inline]
    fn reset(&mut self) { self.pos.set(0); }

    #[inline]
    fn mark(&self) -> Mark { Mark { cur: self, pos: self.pos, cancelled: false } }
}

struct Mark<'b, 'a> {
    cur: &'b Cursor<'a>,
    pos: uint,
    cancelled: bool
}

impl Drop for Mark<'b, 'a> {
    fn drop(&mut self) {
        if !cancelled {
            self.cur.pos.set(self.pos);
        }
    }
}

impl Mark {
    #[inline]
    fn cancel(&mut self) { self.cancelled = true; }
}

pub struct MarkdownParser<'a> {
    cur_stack: Vec<Cursor<'a>>,
    event_queue: RingBuf<MarkdownResult<Block>>
}

impl MarkdownParser {
    pub fn new(buffer: &[u8]) -> MarkdownParser {
        MarkdownParser {
            cur_stack: vec![Cursor::new(buffer)],
            event_queue: RingBuf::new()
        }
    }

    pub fn tokens(self) -> MarkdownTokens<R> {
        MarkdownTokens { parser: self }
    }

    #[inline]
    pub fn next(&mut self) -> MarkdownResult<Block> { 
        self.event_queue.pop_front().unwrap_or_else(|| {
            let result = self.parse_block().to_md_result();
            if self.event_queue.len() > 0 { // recent parse has added elements to the queue
                self.event_queue.push(result);
                self.event_queue.pop_front().unwrap()
            } else {
                result
            }
        })
    }

    #[inline]
    fn cur(&self) -> Cursor {
        self.cur_stack.last().expect("impossible happened, empty cursor stack")
    }

    fn read_while_possible(&mut self) -> (Document, Option<MarkdownError>) {
        let mut result = Vec::new();
        loop {
            match self.parse_block() {
                Success(token) => result.push(token),
                Failure(err) => return (result, Some(err)),
                End => return (result, None),
                NoParse => fail!("unexpected NoParse"),
            }
        }
    }

    fn try_parse_empty_line(&mut self) -> ParseResult<()> {
        let mut m = self.cur().mark();
        loop {
            match opt_ret_end!(self.cur().next()) {
                b' ' => {}
                b'\n' => { m.cancel(); return Success(()) }
                _ => return NoParse
            }
        }
    }

    fn try_skip_initial_spaces(&mut self) -> ParseResult<()> {
        let mut n: u8 = 0;
        let mut m = self.cur().mark();
        while self.cur().available() {
            if n >= 4 {
                return NoParse;
            }
            match *self.cur() {
                b' ' => { n += 1; self.cur().next(); }  // increase counter and continue
                _ => { m.cancel(); return Success(()); },  // not a space and less than 4 spaces
            }
        }
    }

    fn skip_spaces(&mut self) -> ParseResult<()> {
        while self.cur().available() {
            match *self.cur() {
                b' ' => { self.cur.next(); }
                _ => 
            }
        }
    }
}

pub struct MarkdownTokens<'a> {
    parser: MarkdownParser<'a>
}

impl<'a> Iterator<Block> for MarkdownTokens<'a> {
    #[inline]
    fn next(&mut self) -> Option<Block> {
        self.parser.next().ok()
    }
}

trait CharOps {
    fn is_emphasis(self) -> bool;
    fn is_code(self) -> bool;
}

impl CharOps for u8 {
    #[inline]
    fn is_emphasis(self) -> bool {
        self == b'*' || self == b'_'
    }

    #[inline]
    fn is_code(self) -> bool {
        self == b'`' || self == b'`'
    }
}

enum ParseResult<T> {
    Success(T),
    Failure(MarkdownError),
    NoParse,
    End
}

impl<T> ParseResult<T> {
    fn to_result(self) -> MarkdownResult<T> {
        match self {
            Success(r) => Ok(r),
            Failure(e) => Err(e),
            NoParse | End => fail!("programming error, End/NoParse is converted to result")
        }
    }
}
