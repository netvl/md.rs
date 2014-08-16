use parser::{MarkdownParser, ParseResult, Success, End, NoParse};
use tokens::*;

pub trait ListsParser {
    fn parse_ordered_list(&self) -> ParseResult<Block>;
    fn parse_unordered_list(&self) -> ParseResult<Block>;
}

impl<'a> ListsParser for MarkdownParser<'a> {
    fn parse_ordered_list(&self) -> ParseResult<Block> {
        NoParse
    }

    fn parse_unordered_list(&self) -> ParseResult<Block> {
        NoParse
    }
}
