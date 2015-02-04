use parser::{MarkdownParser, ParseResult, End};
use tokens::*;

use self::block_quote::BlockQuoteParser;
use self::block_code::BlockCodeParser;
use self::atx_heading::AtxHeadingParser;
use self::lists::ListsParser;
use self::misc::MiscParser;

mod block_quote;
mod block_code;
mod atx_heading;
mod lists;
mod misc;

pub trait BlockParser {
    fn parse_block(&self) -> ParseResult<Block>;
}

impl<'a> BlockParser for MarkdownParser<'a> {
    fn parse_block(&self) -> ParseResult<Block> {
        debug!("--- parsing a block");
        // Skip empty lines
        while ret_on_end!(self.try_parse_empty_line()).is_success() {}

        first_of! {
            self.parse_block_quote(),
            self.parse_block_code(),
            self.parse_horizontal_rule(),
            self.parse_atx_heading(),
            self.parse_list(),
            self.parse_paragraph(),
            panic!("programming error, parsing block failed")
        }
    }
}
