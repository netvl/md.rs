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
        debug!(">> counting hashes");
        let mut level = 0;
        while level < 6 {
            match self.cur.next_byte() {
                Some(b'#') => level += 1,
                Some(_) => { self.cur.prev(); break }
                None => break
            }
        }
        debug!(">> hashes: {}", level);

        // skip spaces after hashes
        // short-circuit if the document ends here
        debug!(">> skipping spaces");
        if self.skip_spaces().is_end() {
            return Success(Heading {
                level: level,
                content: Vec::new()
            });
        }

        let pm = self.cur.phantom_mark();

        // read the rest of the line
        debug!(">> reading rest of the line");
        self.read_line();
        let buf = self.cur.slice_until_now_from(pm);  // without newline
        debug!(">> header line: {}", buf);

        debug!(">> skipping ending hashes and spaces");
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

        debug!(">> parsing header inline content");
        // parse header contents
        let subp = self.fork(buf.slice_to(n));
        let result = self.fix_links(subp.parse_inline());
        debug!(">> parsed: {}", result);

        Success(Heading {
            level: level,
            content: result
        })
    }
}
