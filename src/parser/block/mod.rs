use super::MarkdownParser;

mod block_quote;
mod block_code;
mod atx_heading;
mod lists;
mod misc;

impl MarkdownParser {
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
}
