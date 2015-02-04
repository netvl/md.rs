use parser::{MarkdownParser, ParseResult, Success, End};
use tokens::*;

pub trait EscapeParser {
    fn parse_escape(&self) -> ParseResult<Option<Inline>>;
}

impl<'a> EscapeParser for MarkdownParser<'a> {
    fn parse_escape(&self) -> ParseResult<Option<Inline>> {
        const ESCAPE_CHARS: &'static [u8] = b"\\`*_{}[]()#+-.!:|&<>^~";

        match self.cur.next_byte() {
            Some(c) if ESCAPE_CHARS.contains(&c) => 
                Success(Some(Chunk(String::from_utf8(vec![c]).unwrap()))),
            Some(_) => Success(None),
            None => End
        }
    }
}
