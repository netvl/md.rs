use parser::{MarkdownParser, ParseResult, Success, End, NoParse};
use tokens::*;

pub trait BlockCodeParser {
    fn parse_block_code(&self) -> ParseResult<Block>;
}

trait Ops {
    fn block_code_prefix(&self) -> ParseResult<()>;
}

impl<'a> Ops for MarkdownParser<'a> {
    fn block_code_prefix(&self) -> ParseResult<()> {
        let mut n = 0u;
        let m = self.cur.mark();
        loop {
            if n == 4 { break };
            parse_or_ret!(self.try_read_char(b' '));
            n += 1
        }
        m.cancel();
        Success(())
    }
}

impl<'a> BlockCodeParser for MarkdownParser<'a> {
    fn parse_block_code(&self) -> ParseResult<Block> {
        debug!(">> trying code block");

        let m = self.cur.mark();
        parse_or_ret!(self.block_code_prefix());
        m.reset();

        let mut buf = Vec::new();
        loop {
            match self.block_code_prefix() {
                NoParse => {  // no prefix, check for emptiness
                    let m = self.cur.mark();
                    match self.try_parse_empty_line() {
                        // non-empty line without prefix or end of buffer
                        NoParse | End => break,
                        // empty line without prefix, add newline to the result
                        _ => { m.cancel(); buf.push(b'\n'); }
                    }
                }
                End => break,
                // prefix is ok, read everything else
                _ => { parse_or_break!(self.read_line_to(&mut buf)); }
            }
        }

        // TODO: handle UTF-8 decoding error
        Success(BlockCode { tag: None, content: String::from_utf8(buf).unwrap() })
    }
}
