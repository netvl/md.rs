use parser::{MarkdownParser, ParseResult, Success, End, NoParse};
use tokens::*;

pub trait BlockQuoteParser {
    fn parse_block_quote(&self) -> ParseResult<Block>;
}

trait Ops {
    fn block_quote_prefix(&self) -> ParseResult<()>;
}

impl<'a> Ops for MarkdownParser<'a> {
    fn block_quote_prefix(&self) -> ParseResult<()> {
        parse_or_ret!(self.try_skip_initial_spaces());
        parse_or_ret!(self.try_read_char(b'>'));
        self.try_read_char(b' ');
        Success(())
    }
}

impl<'a> BlockQuoteParser for MarkdownParser<'a> {
    fn parse_block_quote(&self) -> ParseResult<Block> {
        debug!(">> trying blockquote");

        let m = self.cur.mark();
        parse_or_ret!(self.block_quote_prefix());
        m.reset();

        let mut buf = Vec::new();
        loop {
            break_on_end!(self.block_quote_prefix());
            parse_or_break!(self.read_line_to(&mut buf));

            // break if there is an empty line followed by non-quote line after this line
            match self.try_parse_empty_line() {
                Success(_) => {
                    let mut _m = self.cur.mark();
                    match self.block_quote_prefix() {
                        NoParse | End => break,
                        _ => {}
                    }
                }
                End => break,
                _ => {}
            }
        }

        let subp = self.fork(buf.as_slice());
        let result = self.fix_links(subp.read_all());

        Success(BlockQuote(result))
    }

}
