#![feature(struct_variant, globs, macro_rules, phase)]

#[phase(plugin, link)] extern crate log;
extern crate collections;

use std::io;
use std::mem;

pub use result::*;
pub use tokens::*;
pub use parser::MarkdownParser;

pub mod tokens;
pub mod result;
pub mod parser;

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

impl<R: Reader> MarkdownParser<R> {
    
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

    fn skip_spaces(&mut self) -> ParseResult<()> {
        loop {
            match iotry_err!(self.read_byte()).unwrap() {
                b' ' => {},
                _ => { self.unread_byte(); return Success(()) }
            }
        }
    }

    fn parse_error<T>(&mut self, what: &str) -> ParseResult<T> {
        parse_error!("unable to parse {}; current buffer contents: {}", what, self.stack.peek().data)
    }
}
