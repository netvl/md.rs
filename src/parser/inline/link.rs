use parser::{MarkdownParser, PhantomMark, Success, End};
use tokens::*;

pub trait LinkParser {
    fn parse_link(&self) -> Option<Inline>;
}

impl<'a> LinkParser for MarkdownParser<'a> {
    fn parse_link(&self) -> Option<Inline> {
        let m = self.cur.mark();
        let pm = self.cur.phantom_mark();

        // find matching closing brace
        let mut escaping = false;
        let mut level = 1;
        loop {
            match self.next_byte() {
                Some(b'\\') => escaping = true,
                Some(_) if escaping => escaping = false,
                Some(b'[') => level += 1,
                Some(b']') => {
                    level -= 1;
                    if level <= 0 { break; }
                }
                None => return None
            }
        }

        let label = self.cur.slice_until_now_from(pm);

        // TODO: footnote links?

        // skip spaces
        parse_or_ret_none!(self.skip_spaces_or_newlines());

        match self.cur.current_byte() {
            Some(b'(') => {  // inline link
                self.next();

                // skip whitespace
                parse_or_ret_none!(self.skip_spaces_and_newlines());

            }
            Some(b'[') => {  // reference link
            }
            _ => {  // shortcut reference link
            }
        }
    }
}
