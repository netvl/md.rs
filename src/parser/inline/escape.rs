use parser::{MarkdownParser, ParseResult, Success, End, NoParse};
use tokens::*;

pub trait EscapeParser {
    fn parse_escape(&self) -> ParseResult<Option<Inline>>;
}

impl<'a> EscapeParser for MarkdownParser<'a> {
    fn parse_escape(&self) -> ParseResult<Option<Inline>> {
        static ESCAPE_CHARS: &'static [u8] = b"\\`*_{}[]()#+-.!:|&<>^~";

        match self.cur.next_byte() {
            Some(c) if ESCAPE_CHARS.contains(&c) => 
                Success(Some(Chunk([c.to_ascii()].as_str_ascii().to_string()))),
            Some(_) => Success(None),
            None => End
        }
    }
}
