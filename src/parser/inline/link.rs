use std::str;
use std::borrow::ToOwned;

use parser::{MarkdownParser, Success, End, NoParse};
use tokens::*;
use util::{ByteSliceOps, CharOps};

pub trait LinkParser {
    fn parse_link(&self, is_image: bool) -> Option<Inline>;
}

impl<'a> LinkParser for MarkdownParser<'a> {
    fn parse_link(&self, is_image: bool) -> Option<Inline> {
        let pm = self.cur.phantom_mark();
        let label;

        // find matching closing brace
        let mut escaping = false;
        let mut level = 1usize;
        loop {
            let c = opt_ret!(self.cur.next_byte());
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
        let m = self.cur.mark();

        // TODO: footnote links?

        // skip spaces
        self.skip_spaces_and_newlines();

        let mut link = None;
        let mut title = None;
        let mut id = None;

        match self.cur.current_byte() {
            Some(b'(') => {  // inline link
                self.cur.next();

                // skip initial whitespace
                parse_or_ret_none!(self.skip_spaces_and_newlines());
                let pm = self.cur.phantom_mark();

                // read until link end, balancing parentheses
                let mut level = 0usize;
                loop {
                    let c = opt_ret!(self.cur.next_byte());
                    match c {
                        b'\\' => { self.cur.next(); },  // skip escaped char
                        b'(' => level += 1,
                        b')' => if level == 0 { break; } else { level -= 1; },
                        // encountered link title
                        cc if (cc == b'\'' || cc == b'"') &&
                            self.cur.peek_before_prev().is_space() => break,
                        _ => {}  // just pass through
                    }
                }

                let link_slice = self.cur.slice_until_now_from(pm);
                debug!("read link slice: {}", str::from_utf8(link_slice).unwrap());

                // read title, if it is there
                let pc = self.cur.peek_prev();
                if pc == b'\'' || pc == b'\"' {  // title
                    let pm = self.cur.phantom_mark();

                    let mut read_title = false;
                    loop {
                        let c = opt_ret!(self.cur.next_byte());
                        match c {
                            b'\\' => { self.cur.next(); },  // skip escaped byte
                            cc if cc == pc && !read_title => {
                                title = Some(self.cur.slice_until_now_from(pm));
                                read_title = true;
                            }
                            b')' if read_title => break,
                            _ => {}
                        }
                    }
                }
                
                link = Some(
                    link_slice.trim_right(|b: u8| b.is_space())
                        .trim_left_one(b'<').trim_right_one(b'>')
                );

                m.cancel();
            }

            Some(b'[') => {  // reference link
                self.cur.next();
                let pm = self.cur.phantom_mark();

                loop {
                    let c = opt_ret!(self.cur.next_byte());
                    match c {
                        b']' => break,
                        _ => {}
                    }
                }

                id = Some(self.cur.slice_until_now_from(pm));

                m.cancel();
            }

            _ => {  // shortcut reference link
                m.reset();  // revert to the first character after ']'

                id = Some(label);
            }
        }

        // TODO: parse link contents
        let text = vec![Chunk(str::from_utf8(label).unwrap().to_owned())];

        let link = link.map(|link| str::from_utf8(link).unwrap().to_owned());
        let id = id.map(|id| str::from_utf8(id).unwrap().to_owned());
        let title = title.map(|title| str::from_utf8(title).unwrap().to_owned());

        let link = if is_image {
            Image {
                id: id,
                link: link,
                title: title,
                alt: text
            }
        } else {
            Link {
                id: id,
                link: link,
                title: title,
                text: Some(text)
            }
        };

        Some(link)
    }
}
