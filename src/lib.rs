#![feature(struct_variant, globs, macro_rules, phase)]

#[phase(plugin, link)] extern crate log;

use std::io;

pub use result::*;
pub use tokens::*;

pub mod tokens;
pub mod result;

#[repr(u8)]
#[deriving(FromPrimitive)]
enum SetextHeaderLevel {
    StxFirst = b'=',
    StxSecond = b'-'
}

impl SetextHeaderLevel {
    #[inline]
    fn to_numeric(self) -> uint {
        match self {
            StxFirst => 1,
            StxSecond => 2
        }
    }
}

trait CharOps {
    fn is_emphasis(self) -> bool;
}

impl CharOps for u8 {
    #[inline]
    fn is_emphasis(self) -> bool {
        self == b'*' || self == b'_'
    }
}

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
        debug!(">>> setting mark at {}", self.buf.pos);
        self.buf.marks.push(self.buf.pos);
    }

    fn unset(&mut self) {
        assert!(self.buf.marks.len() > 0);
        debug!(">>> unsetting previously set mark at {}", *self.buf.marks.last().unwrap());
        self.buf.marks.pop();
    }

    fn reset(&mut self) {
        assert!(self.buf.marks.len() > 0);
        debug!(">>> resetting to previously set mark at {}", *self.buf.marks.last().unwrap());
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
        if self.data.len() == self.pos {
            self.data.clear();
        } else {
            self.data = self.data.slice_from(self.pos).to_vec();
        }
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
    fn new() -> BufStack { BufStack { bufs: vec![Buf::new()] } }

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
    stack: BufStack,
    event_queue: Vec<MarkdownResult<Block>>
}

impl<R: Reader> MarkdownParser<R> {
    pub fn new(r: R) -> MarkdownParser<R> {
        MarkdownParser {
            source: r,
            stack: BufStack::new(),
            event_queue: Vec::new()
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
    ($e:expr; -> $err:ident) => (
        match $e {
            Failure(e) | PartialSuccess(_, e) => { 
                $err = Some(e); 
                break
            }
            o => o
        }
    );
    ($e:expr; -> $err:ident; $lt:ident) => (
        match $e {
            Failure(e) | PartialSuccess(_, e) => { 
                $err = Some(e); 
                break $lt
            }
            o => o
        }
    )
)

macro_rules! break_err_check(
    ($_self:expr; $e:expr; -> $err:ident) => ({
        $_self.mark().set();
        match $e {
            Failure(e) => { $err = Some(e); break }
            PartialSuccess(_, e) => { $err = Some(e); break }
            NoParse => { $_self.mark().reset(); NoParse }
            Success(r) => { $_self.mark().unset(); Success(r) }
        }
    })
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
    fn is_success(&self) -> bool {
        match *self {
            Success(_) => true,
            _ => false
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

    #[inline]
    fn map<U>(self, f: |T| -> U) -> ParseResult<U> {
        match self {
            Success(r) => Success(f(r)),
            PartialSuccess(r, e) => PartialSuccess(f(r), e),
            Failure(e) => Failure(e),
            NoParse => NoParse
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
    pub fn next(&mut self) -> MarkdownResult<Block> { 
        self.event_queue.shift().unwrap_or_else(|| {
            let result = self.parse_block().to_md_result();
            if self.event_queue.len() > 0 { // added elements to the queue
                self.event_queue.push(result);
                self.event_queue.shift().unwrap()
            } else {
                result
            }
        })
    }
    
    #[inline]
    fn push_existing(&mut self, buf: Vec<u8>) { self.stack.push(buf); }

    #[inline]
    fn push_buf(&mut self) { self.stack.push_new(); }
    
    #[inline]
    fn pop_buf(&mut self) { self.stack.pop(); }

    #[inline]
    fn read_byte(&mut self) -> io::IoResult<u8> {
        let b = self.stack.read_from(&mut self.source);
        match b {
            Ok(ref b) => debug!(">>> read byte: {} ~ {}", *b, (*b) as char),
            Err(ref e) => debug!(">>> error reading byte: {}", *e)
        }
        b
    }

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
    fn unread_bytes(&mut self, n: uint) { 
        debug!(">>> unreading {} bytes", n);
        self.stack.unread(n);
    }
    
    #[inline]
    fn unread_byte(&mut self) { self.unread_bytes(1); }

    #[inline]
    fn consume(&mut self) { 
        debug!(">>> consuming buffer");
        self.stack.consume();
    }

    #[inline]
    fn backtrack(&mut self) { 
        debug!(">>> rewinding buffer to initial position");
        self.stack.rewind();
    }

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
        debug!("--- parsing a block");
        // Skip empty lines
        loop {
            match try_err_f!(self.try_parse_empty_line()) {
                NoParse => break,
                _ => self.consume()
            }
        }
        first_of! {
            self.block_quote() or
            self.block_code() or
            self.horizontal_rule() or
            self.atx_heading() or
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

        debug!(">> trying blockquote");
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
            let mut n = 0u;
            this.mark().set();
            loop {
                if n == 4 { break };
                match try_err_f!(this.read_char(b' ')) {
                    NoParse => { this.mark().reset(); return NoParse },
                    _ => n += 1
                }
            }
            this.mark().unset();
            Success(())
        }

        debug!(">> trying code block");
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

    fn horizontal_rule(&mut self) -> ParseResult<Block> {
        debug!(">> trying hrule");
        try_reset!(self; self.skip_initial_spaces());

        let mut c = iotry_err!(self.read_byte()).unwrap();
        if c == b'-' || c == b'*' || c == b'_' {
            loop {
                match iotry_err!(self.read_byte()).unwrap() {
                    b'\n' => break,
                    b' ' => c = b' ',  // from now on everything should be spaces
                    cc if cc == c => {}
                    _ => { self.backtrack(); return NoParse }
                }
            }
            self.consume();
            Success(HorizontalRule)
        } else {
            self.backtrack();
            NoParse
        }
    }

    fn atx_heading(&mut self) -> ParseResult<Block> {
        debug!(">> trying atx header");
        try_parse!(check_reset!(self; try_err_f!(self.read_char(b'#'))));
        self.unread_byte();

        // read and count hashes
        let mut n = 0;
        let mut err = None;
        while n < 6 {
            match break_err!(self.read_byte_pr(); -> err).unwrap() {
                b'#' => n += 1,
                _ => { self.unread_byte(); break }
            }
        }

        let mut buf = Vec::new();

        // skip spaces after hashes
        err = err.or_else(|| match self.skip_spaces() {
            PartialSuccess(_, e) | Failure(e) => Some(e),
            _ => None
        // read the rest of the line
        }).or_else(|| match self.read_line_pr(&mut buf) {
            Failure(e) => Some(e),
            _ => { 
                buf.pop();  // remove newline
                None
            }
        });

        // if there were errors, return partial success with empty text
        match err {
            Some(e) => return PartialSuccess(Heading {
                level: n,
                content: Vec::new()
            }, e),
            None => {}
        }
         
        // skip hashes and spaces backwards
        while buf.len() > 0 {
            match *buf.last().unwrap() {
                b'#' => { buf.pop(); }
                _ => break
            }
        }
        while buf.len() > 0 {
            match *buf.last().unwrap() {
                b' ' => { buf.pop(); }
                _ => break
            }
        }

        // parse header contents
        self.push_existing(buf);
        let result = self.inline();
        self.pop_buf();
        self.consume();

        result.map(|r| Heading {
            level: n,
            content: r
        })
    }

    fn paragraph(&mut self) -> ParseResult<Block> {
        debug!(">> reading paragraph");

        let mut buf = Vec::new();
        let mut err0 = None;
        let mut level = None;
        loop {
            break_err!(self.read_line_pr(&mut buf); -> err0);

            // empty line means paragraph end
            match break_err!(self.try_parse_empty_line(); -> err0) {
                Success(_) => break,
                _ => {}
            }

            // header line means that the paragraph ended, and its last line
            // should be parsed as a heading
            match break_err!(self.try_parse_header_line(); -> err0) {
                Success(r) => { level = Some(r); break }
                _ => {}
            }

            // TODO: check for atx header, hrule or quote

            // TODO: lax spacing rules: check for list/html block/code fence
        }

        match level {
            // extract last line from the buffer
            Some(level) => {
                // unwrap is safe because buf will contain at least one
                // line in this case
                let nl_idx = buf.as_slice().rposition_elem(&b'\n').unwrap();
                let head_content = buf.slice_from(nl_idx+1).to_vec();
                buf.truncate(nl_idx+1);

                self.push_existing(head_content);
                let result = self.inline();
                self.pop_buf();

                let heading_result = result.map(|r| Heading {
                    level: level.to_numeric(),
                    content: r
                });
                self.event_queue.push(heading_result.to_md_result());
            }
            None => {}
        }

        self.push_existing(buf);
        let result = self.inline();
        self.pop_buf();
        self.consume();

        match (err0, result) {
            (Some(e), Success(r)) => PartialSuccess(Paragraph(r), e),
            (Some(e), PartialSuccess(r, _)) => PartialSuccess(Paragraph(r), e),
            (Some(e), _) => Failure(e),
            (None, r) => r.map(Paragraph)
        }
    }

    fn skip_spaces(&mut self) -> ParseResult<()> {
        loop {
            match iotry_err!(self.read_byte()).unwrap() {
                b' ' => {},
                _ => { self.unread_byte(); return Success(()) }
            }
        }
    }

    fn try_parse_header_line(&mut self) -> ParseResult<SetextHeaderLevel> {
        self.mark().set();

        // TODO: is it important that the mark can be left set?
        let mut cc = match iotry_err!(self.read_byte()).unwrap() {
            c if c == b'=' || c == b'-' => c,
            _ => { self.mark().reset(); return NoParse }
        };
        let level = FromPrimitive::from_u8(cc).unwrap();

        loop {
            match iotry_err!(self.read_byte()).unwrap() {
                c if c == cc => {},
                b' ' => cc = b' ',  // consume only spaces from now on
                b'\n' => { self.mark().unset(); return Success(level) },
                _ => { self.mark().reset(); return NoParse }
            }
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

    fn ordered_list(&mut self) -> ParseResult<Block> {
        NoParse
    }

    fn unordered_list(&mut self) -> ParseResult<Block> {
        NoParse
    }

    fn inline(&mut self) -> ParseResult<Text> {
        let mut tokens = Vec::new();
        let mut buf = Vec::new();
        let mut err;
        let mut escaping = false;
        let mut space = false;

        macro_rules! continue_with(
            ($($c:expr),+) => ({
                $(buf.push($c);)+
                continue
            })
        )

        fn find_emph_closing<R: Reader>(this: &mut MarkdownParser<R>, ec: u8, n: uint) 
            -> ParseResult<Vec<u8>> {
            assert!(n > 0);  // need at least one emphasis character

            let mut result = Vec::new();
            let mut err = None;
            let mut escaping = false;

            'outer: loop {
                match break_err!(this.read_byte_pr(); -> err).unwrap() {
                    // pass escaped characters as is
                    b'\\' => { result.push(b'\\'); escaping = true }
                    c if escaping => { result.push(c); escaping = false }

                    // found our emphasis character
                    c if c == ec => {
                        let mut rn = 1;
                        this.mark().set();
                        while break_err!(this.read_char(ec); -> err).is_success() {
                            rn += 1;
                        }

                        if rn == n { // this is our emphasis boundary, finish reading
                            this.unread_byte();
                            this.mark().unset();
                            break;
                            
                        } else { // otherwise reset to the character just after the one we just have read
                            this.mark().reset();
                            result.push(c);
                        }
                    }

                    // inline code block started, and we're not inline code block ourselves
                    // we need to pass through this block as is
                    c if c == b'`' => {  
                        // count `s
                        let mut sn = 1u;
                        while break_err!(this.read_char(b'`'); -> err).is_success() {
                            result.push(b'`');
                            sn += 1;
                        }
                        this.unread_byte();

                        let mut tn = 0;
                        while tn < sn {
                            match this.read_byte() {
                                Ok(b) => {
                                    result.push(b);
                                    match b {
                                        b'`' => tn += 1,
                                        _    => tn = 0
                                    }
                                },

                                Err(e) => {
                                    err = Some(IoError(e));
                                    break 'outer;
                                }
                            }
                        }
                    }

                    // TODO: skip hyperlinks

                    // just pass through any other character
                    c => result.push(c)
                }
            }

            match err {
                Some(e) => PartialSuccess(result, e),
                None => Success(result)
            }
        }

        loop {
            match break_err!(self.read_byte_pr(); -> err).unwrap() {
                b' ' => { space = true; continue_with!(b' ') }

                b'\\' => { escaping = true; continue }
                c if escaping => { escaping = false; continue_with!(c) }

                c if c.is_emphasis() => { 
                    if space {
                        space = false;
                        match break_err!(self.read_byte_pr(); -> err).unwrap() {
                            b' ' => { 
                                self.unread_byte();
                                continue_with!(c)  // this character is escaped
                            }
                            _ => self.unread_byte()
                        }
                    } 
                    
                    // one or two emphasis characters
                    let mut n = 1;
                    if break_err_check!(self; self.read_char(c); -> err).is_success() {
                        n += 1;
                    }

                    // read everything until closing emphasis bracket
                    let buf = break_err!(find_emph_closing(self, c, n); -> err).unwrap();
                    let result = if c == b'`' {  // code block
                        self.consume();
                        Code(String::from_utf8(buf).unwrap())  // TODO: handle UTF8 errors
                    } else {
                        self.push_existing(buf);
                        let result = break_err!(self.inline(); -> err).unwrap();
                        self.pop_buf();
                        self.consume();  // up to and including closing emphasis
                        match n {
                            1 => Emphasis(result),
                            2 => MoreEmphasis(result),
                            _ => unreachable!()
                        }
                    };

                    tokens.push(result)
                }

                // push plain characters to the buffer
                c => buf.push(c)
            }
        }

        match err {
            // TODO: handle UTF-8 decoding errors
            None => Success(tokens),
            Some(IoError(ref e)) if e.kind == io::EndOfFile => Success(tokens),
            Some(e) => PartialSuccess(tokens, e)
        }
    }

    fn parse_error<T>(&mut self, what: &str) -> ParseResult<T> {
        parse_error!("unable to parse {}; current buffer contents: {}", what, self.stack.peek().data)
    }
}
