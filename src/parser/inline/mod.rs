use parser::{MarkdownParser, Cursor, PhantomMark, ParseResult, Success, End, NoParse};
use tokens::*;
use util::CharOps;

use self::emphasis::EmphasisParser;
use self::escape::EscapeParser;

mod emphasis;
mod escape;

pub trait InlineParser {
    fn parse_inline(&self) -> Text;
}

struct InlineParsingState<'b, 'a> {
    tokens: Vec<Inline>,
    cur: Cursor<'a>,
    pm: PhantomMark<'b, 'a>,
    pm_last: PhantomMark<'b, 'a>
}

impl<'b, 'a> InlineParsingState<'b, 'a> {
    #[inline]
    fn update(&'b mut self) {
        self.pm = self.cur.phantom_mark();
        self.pm_last = self.pm;
    }

    fn push_token(&'b mut self, token: Inline) {
        match (self.tokens.mut_last(), token) {
            // merge chunks
            (Some(&Chunk(ref mut buf)), Chunk(ref buf0)) => buf.push_all(buf0.as_slice()),
            (_, token) => self.tokens.push(token)
        }
    }

    fn push_chunk(&'b mut self) {
        let slice = self.pm.slice_to(&self.pm_last);
        if slice.is_empty() { return; }

        let chunk = slice.to_vec();
        // TODO: handle UTF-8 decoding error
        self.tokens.push(Chunk(String::from_utf8(chunk).unwrap()));

        self.update();
    }

    #[inline]
    fn advance(&'b mut self) {
        self.pm_last = self.cur.phantom_mark();
    }
}


impl<'a> InlineParser for MarkdownParser<'a> {
    fn parse_inline(&self) -> Text {
        let mut s = InlineParsingState {
            tokens: Vec::new(),
            cur: self.cur,
            pm: self.cur.phantom_mark(),
            pm_last: self.cur.phantom_mark()
        };

        loop {
            let c = opt_break!(self.cur.next_byte());
            match c {
                b'\\' => match break_on_end!(self.parse_escape()).unwrap() {
                    Some(token) => {
                        s.push_chunk();
                        s.push_token(token);
                        s.update();
                    }
                    None => s.advance()
                },

                c if c.is_emphasis() || c.is_code() => {
                    s.push_chunk();

                    // one or two emphasis characters
                    let mut n = 1;
                    if break_on_end!(self.try_read_char(c)).is_success() {
                        n += 1;
                    }

                    let token = opt_break!(self.parse_emphasis(c, n));
                    s.push_token(token);
                }

                // just advance
                _ => s.advance()
            }
        }

        s.push_chunk();

        s.tokens
    }
}
