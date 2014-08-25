use parser::{MarkdownParser, PhantomMark, Success, End};
use tokens::*;
use util::ByteSliceOps;

pub trait LinkParser {
    fn parse_link(&self) -> Option<Inline>;
}

impl<'a> LinkParser for MarkdownParser<'a> {
    fn parse_link(&self) -> Option<Inline> {
        let m = self.cur.mark();
        let pm = self.cur.phantom_mark();
        let label;

        // find matching closing brace
        let mut escaping = false;
        let mut level = 1;
        loop {
            let c = opt_ret!(self.next_byte());
            match c {
                b'\\' => escaping = true,
                _ if escaping => escaping = false,
                b'[' => level += 1,
                b']' => {
                    level -= 1;
                    if level <= 0 { break; }
                }
                _ => {}
            }
        }

        label = self.cur.slice_until_now_from(pm);
        
        // if this is shortcut link, we'll return here
        let m = { m.cancel(); self.cur.mark() };  

        // TODO: footnote links?

        // skip spaces
        self.skip_spaces_or_newlines();

        let mut link = None;
        let mut title = None;
        let mut id = None;

        match self.cur.current_byte() {
            Some(b'(') => {  // inline link
                self.next();
                let pm = self.cur.phantom_mark();

                // skip initial whitespace
                parse_or_ret_none!(self.skip_spaces_and_newlines());

                // read until link end, balancing parentheses
                let mut level = 0;
                loop {
                    let c = opt_ret!(self.next_byte());
                    match c {
                        b'\\' => self.cur.next(),  // skip escaped byte
                        b'(' => level += 1,
                        b')' => if level == 0 { break; } else { level -= 1; },
                        // encountered link title
                        cc if self.cur.peek_prev().is_space() && 
                              (cc == b'\'' || cc == b'"') => break,
                        _ => {}  // just pass through
                    }
                }

                let link_slice = self.cur.slice_until_now_from(pm);

                // read title, if it is there
                let pc = self.cur.peek_prev();
                if pc == '\'' || pc == '\"' {  // title
                    let qc = pc;
                    let pm = self.cur.phantom_mark();

                    let mut read_title = false;
                    loop {
                        let c = opt_ret!(self.next_byte());
                        match c {
                            b'\\' => self.cur.next(),  // skip escaped byte
                            cc if c == qc && !read_title => {
                                title = Some(self.cur.slice_until_now_from(pm));
                                read_title = true;
                            }
                            b')' if read_title => break,
                            _ => {}
                        }
                    }
                }
                
                link = Some(
                    link_slice.trim_right(|b| b.is_space())
                        .trim_left_one(b'<').trim_right_one(b'>')
                );

                m.cancel();
            }

            Some(b'[') => {  // reference link
                self.next();
                let pm = self.cur.phantom_mark();

                loop {
                    let c = opt_ret!(self.next_byte());

                }
            }

            _ => {  // shortcut reference link
            }
        }
    }
}
