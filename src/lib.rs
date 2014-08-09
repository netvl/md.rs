#![feature(struct_variant, globs, macro_rules)]

use std::io;

pub use result::*;
pub use tokens::*;

pub mod tokens;
pub mod result;

struct Buf {
    data: Vec<u8>,
    marks: Vec<uint>,
    pos: uint,
    sized: bool
}

struct Mark<'a> {
    buf: &'a mut Buf
}

impl<'a> Mark<'a> {
    fn set(&mut self) {
        self.buf.marks.push(self.buf.pos);
    }

    fn unset(&mut self) {
        assert!(self.buf.marks.len() > 0);
        self.buf.marks.pop();
    }

    fn reset(&mut self) {
        assert!(self.buf.marks.len() > 0);
        self.buf.pos = self.buf.marks.pop().unwrap();
    }
}

impl Buf {
    #[inline]
    fn new_existing(data: Vec<u8>) -> Buf {
        Buf { data: data, marks: Vec::new(), pos: 0, sized: true }
    }

    #[inline]
    fn new() -> Buf {
        Buf { data: Vec::new(), marks: Vec::new(), pos: 0, sized: false }
    }

    fn read_from<R: Reader>(&mut self, source: &mut R) -> io::IoResult<u8> {
        if self.pos < self.data.len() {
            let ch = self.data[self.pos];
            self.pos += 1;
            Ok(ch)
        } else if self.sized {
            Err(io::IoError { kind: io::EndOfFile, desc: "buffer exhausted", detail: None })
        } else {
            source.read_byte().map(|c| { 
                self.data.push(c);
                self.pos += 1;
                c
            })
        }
    }

    #[inline]
    fn unread(&mut self, n: uint) {
        assert!(n <= self.pos);
        self.pos -= n;
    }

    #[inline]
    fn rewind(&mut self) {
        self.pos = 0;
    }
    
    #[inline]
    fn consume(&mut self) {
        self.data.clear();
        self.marks.clear();
        self.pos = 0;
    }

    #[inline]
    fn mark(&mut self) -> Mark {
        Mark { buf: self }
    }
}

struct BufStack {
    bufs: Vec<Buf>
}

impl BufStack {
    #[inline]
    fn new() -> BufStack { BufStack { bufs: Vec::new() } }

    #[inline]
    fn push(&mut self, data: Vec<u8>) {
        self.bufs.push(Buf::new_existing(data))
    }

    #[inline]
    fn push_new(&mut self) {
        self.bufs.push(Buf::new());
    }

    #[inline]
    fn peek(&mut self) -> &mut Buf {
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

    fn mark(&mut self) -> Mark {
        assert!(self.bufs.len() > 0);
        self.bufs.mut_last().unwrap().mark()
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

macro_rules! try_reset(
    ($_self:ident; $e:expr) => ({
        $_self.mark().set();
        match $e {
            NoParse => { $_self.mark().reset(); return NoParse }
            r => { $_self.mark().unset(); r }
        }
    })
)

macro_rules! check_reset(
    ($_self:ident; $e:expr) => ({
        $_self.mark().set();
        match $e {
            NoParse => { $_self.mark().reset(); NoParse }
            r => { $_self.mark().unset(); r }
        }
    })
)

macro_rules! try_parse(
    ($e:expr) => (
        match $e {
            NoParse => return NoParse,
            r => r
        }
    )
)

macro_rules! try_err(
    ($e:expr) => (
        match $e {
            Failure(e) => return Failure(e),
            PartialSuccess(r, e) => return PartialSuccess(r, e),
            r => r
        }
    )
)

macro_rules! try_err_f(
    ($e:expr) => (
        match $e {
            Failure(e) => return Failure(e),
            PartialSuccess(_, e) => return Failure(e),
            r => r
        }
    )
)

macro_rules! iotry_err(
    ($e:expr) => (
        match $e {
            Ok(r) => Success(r),
            Err(e) => return Failure(MarkdownError::from_io(e))
        }
    )
)

macro_rules! break_err(
    ($e:expr) => (
        match $e {
            Failure(_) => break,
            PartialSuccess(_, _) => break,
            o => o
        }
    );
    ($e:expr; -> $err:ident) => (
        match $e {
            Failure(e) => { $err = Some(e); break }
            PartialSuccess(_, e) => { $err = Some(e); break }
            o => o
        }
    )
)

macro_rules! attempt(
    ($e:expr) => (
        match $e {
            NoParse => Success(()),
            o => o
        }
    )
)

enum ParseResult<T> {
    NoParse,
    Success(T),
    PartialSuccess(T, MarkdownError),
    Failure(MarkdownError)
}

impl<T> ParseResult<T> {
    #[inline]
    fn from_io(r: io::IoResult<T>) -> ParseResult<T> {
        match r {
            Ok(r) => Success(r),
            Err(e) => Failure(MarkdownError::from_io(e))
        }
    }

    #[inline]
    fn is_np(&self) -> bool {
        match *self {
            NoParse => true,
            _ => false
        }
    }

    #[inline]
    fn unwrap(self) -> T {
        match self {
            Success(r) => r,
            PartialSuccess(r, _) => r,
            Failure(e) => fail!("unwrapping failure: {}", e),
            NoParse => fail!("unwrapping NoParse")
        }
    }

    #[inline]
    fn or_else(self, f: || -> ParseResult<T>) -> ParseResult<T> {
        match self {
            NoParse => f(),
            r => r
        }
    }

    fn to_md_result(self) -> MarkdownResult<T> {
        match self {
            Success(r) => result::Success(r),
            PartialSuccess(r, e) => result::PartialSuccess(r, e),
            Failure(e) => result::Failure(e),
            NoParse => fail!("NoParse is converted to MarkdownResult")
        }
    }
}

macro_rules! parse_error(
    ($s:expr) => (Failure(ParseError(std::str::Slice($s))));
    ($s:expr, $($arg:expr),+) => (Failure(ParseError(std::str::Owned(format!($s, $($arg),+)))))
)

impl<R: Reader> MarkdownParser<R> {
    pub fn read_while_possible(&mut self) -> (Document, Option<MarkdownError>) {
        let mut result = Vec::new();
        loop {
            match self.parse_block() {
                Success(token) => result.push(token),
                PartialSuccess(token, err) => {
                    result.push(token);
                    match err {
                        IoError(ref cause) if cause.kind == io::EndOfFile => break,
                        err => return (result, Some(err))
                    }
                }
                Failure(IoError(ref cause)) if cause.kind == io::EndOfFile => break,
                Failure(err) => return (result, Some(err)),
                NoParse => fail!("unexpected NoParse")
            }
        }
        (result, None)
    }

    #[inline]
    pub fn next(&mut self) -> MarkdownResult<Block> { self.parse_block().to_md_result() }
    
    #[inline]
    fn push_existing(&mut self, buf: Vec<u8>) { self.stack.push(buf); }

    #[inline]
    fn push_buf(&mut self) { self.stack.push_new(); }
    
    #[inline]
    fn pop_buf(&mut self) { self.stack.pop(); }

    #[inline]
    fn read_byte(&mut self) -> io::IoResult<u8> { self.stack.read_from(&mut self.source) }

    #[inline]
    fn read_byte_pr(&mut self) -> ParseResult<u8> { ParseResult::from_io(self.read_byte()) }

    fn read_line_pr(&mut self, target: &mut Vec<u8>) -> ParseResult<()> {
        loop {
            let b = iotry_err!(self.read_byte()).unwrap();
            target.push(b);
            if b == b'\n' { break; }
        }
        Success(())
    }

    #[inline]
    fn unread_bytes(&mut self, n: uint) { self.stack.unread(n); }
    
    #[inline]
    fn unread_byte(&mut self) { self.unread_bytes(1); }

    #[inline]
    fn consume(&mut self) { self.stack.consume(); }

    #[inline]
    fn backtrack(&mut self) { self.stack.rewind(); }

    #[inline]
    fn mark(&mut self) -> Mark { self.stack.mark() }

    fn read_char(&mut self, ch: u8) -> ParseResult<()> {
        match self.read_byte() {
            Ok(c) if c == ch => Success(()),
            Ok(_) => NoParse,
            Err(e) => Failure(MarkdownError::from_io(e))
        }
    }

    fn try_read_char(&mut self, ch: u8) -> ParseResult<()> {
        match self.read_byte() {
            Ok(c) if c == ch => Success(()),
            Ok(_) => { self.unread_byte(); NoParse }
            Err(e) => Failure(MarkdownError::from_io(e))
        }
    }
    
    fn parse_block(&mut self) -> ParseResult<Block> {
        first_of! {
            self.block_quote() or
            self.block_code() or
            self.horizontal_rule() or
            self.heading() or
            self.ordered_list() or
            self.unordered_list() or
            self.paragraph() or
            self.parse_error("a block")
        }
    }

    fn block_quote(&mut self) -> ParseResult<Block> {
        fn block_quote_prefix<R: Reader>(this: &mut MarkdownParser<R>) -> ParseResult<()> {
            try_reset!(this; this.skip_initial_spaces());
            try_reset!(this; this.read_char(b'>'));
            attempt!(this.try_read_char(b' ')) // optional space after >
        }

        try_parse!(try_err_f!(block_quote_prefix(self)));
        self.backtrack();

        let mut buf = Vec::new();
        let mut err0 = None;
        loop {
            break_err!(block_quote_prefix(self); -> err0);
            break_err!(self.read_line_pr(&mut buf); -> err0);

            // break if there is an empty line followed by non-quote line after this line
            match break_err!(self.try_parse_empty_line(); -> err0) {
                Success(_) => {
                    self.mark().set();
                    match break_err!(block_quote_prefix(self); -> err0) {
                        NoParse => { self.mark().reset(); break }
                        _ => self.mark().reset()
                    }
                }
                _ => {}
            }
        }

        self.push_existing(buf);
        let (result, err) = self.read_while_possible();
        self.pop_buf();
        self.consume();

        // TODO: validate this table and errors priority
        match (err0, err, result.is_empty()) {
            (_,       Some(e), true) => Failure(e),
            (_,       Some(e), false) => PartialSuccess(BlockQuote(result), e),
            (Some(e), None,    _) => PartialSuccess(BlockQuote(result), e),
            (None,    None,    _) => Success(BlockQuote(result))
        }
    }

    fn block_code(&mut self) -> ParseResult<Block> {
        fn block_code_prefix<R: Reader>(this: &mut MarkdownParser<R>) -> ParseResult<()> {
            try_reset!(this; this.skip_initial_spaces());
            try_reset!(this; this.read_char(b'>'));
            Success(())
        }

        try_parse!(try_err_f!(block_code_prefix(self)));
        self.backtrack();

        let mut buf = Vec::new();
        let mut err = None;
        loop {
            match break_err!(block_code_prefix(self); -> err) {
                NoParse => {  // no prefix, check for emptiness
                    self.mark().set();
                    match break_err!(self.try_parse_empty_line(); -> err) {
                        // non-empty line without prefix
                        NoParse => { self.mark().reset(); break }
                        // empty line without prefix, add newline
                        _ => { self.mark().unset(); buf.push(b'\n') }
                    }
                }
                // prefix is ok, read everything else
                _ => { break_err!(self.read_line_pr(&mut buf); -> err); }
            }
        }

        self.consume();

        // TODO: handle UTF-8 decoding error
        let result = BlockCode { tag: None, content: String::from_utf8(buf).unwrap() };
        match err {
            Some(e) => PartialSuccess(result, e),
            None => Success(result)
        }
    }

    fn try_parse_empty_line(&mut self) -> ParseResult<()> {
        self.mark().set();
        loop {
            // TODO: is it important that the mark can be left set?
            match iotry_err!(self.read_byte()).unwrap() {
                b' ' => {},
                b'\n' => { self.mark().unset(); return Success(()) }
                _ => { self.mark().reset(); return NoParse }
            }
        }
    }

    fn skip_initial_spaces(&mut self) -> ParseResult<()> {
        let mut n: u8 = 0;
        loop {
            if n >= 4 {
                return NoParse;
            }
            match self.read_byte() {
                Ok(b' ') => n += 1,  // increase counter and continue
                Ok(_) => { self.unread_byte(); break },   // not a space and less than 4 spaces
                Err(e) => return Failure(MarkdownError::from_io(e))
            }
        }
        Success(())
    }


    fn horizontal_rule(&mut self) -> ParseResult<Block> {
        NoParse
    }


    fn heading(&mut self) -> ParseResult<Block> {
        NoParse
    }


    fn ordered_list(&mut self) -> ParseResult<Block> {
        NoParse
    }


    fn unordered_list(&mut self) -> ParseResult<Block> {
        NoParse
    }


    fn paragraph(&mut self) -> ParseResult<Block> {
        NoParse
    }


    fn parse_error<T>(&mut self, what: &str) -> ParseResult<T> {
        parse_error!("unable to parse {}; current buffer contents: {}", what, self.stack.peek().data)
    }
}
