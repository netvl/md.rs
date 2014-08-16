use parser::{MarkdownParser, ParseResult, Success, End, NoParse};
use tokens::*;
use parser::inline::InlineParser;

pub trait AtxHeadingParser {
    fn parse_atx_heading(&self) -> ParseResult<Block>;
}

impl<'a> AtxHeadingParser for MarkdownParser<'a> {
    fn parse_atx_heading(&self) -> ParseResult<Block> {
        debug!(">> trying atx header");
        parse_or_ret!(self.try_read_char(b'#'));
        self.cur.prev();

        // read and count hashes
        let mut n = 0;
        while n < 6 {
            match self.cur.next_byte() {
                Some(b'#') => n += 1,
                Some(_) | None => { self.cur.prev(); break }
            }
        }

        let pm = self.cur.phantom_mark();

        // skip spaces after hashes
        // short-circuit if the document ends here
        if self.skip_spaces().is_end() {
            return Success(Heading {
                level: n,
                content: Vec::new()
            });
        }

        // read the rest of the line
        self.read_line();
        let buf = pm.slice_to_now();
        let buf = buf.slice_to(buf.len()-1);  // remove newline

        // skip hashes and spaces backwards
        let mut n = buf.len();
        while n > 0 {
            match buf[n-1] {
                b'#' => n -= 1,
                _ => break
            }
        }
        while n > 0 {
            match buf[n-1] {
                b' ' => n -= 1,
                _ => break
            }
        }

        // parse header contents
        let subp = MarkdownParser::new(buf.slice_to(n));
        let result = subp.parse_inline();

        Success(Heading {
            level: n,
            content: result
        })
    }
}
