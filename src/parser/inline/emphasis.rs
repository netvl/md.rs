use std::str;
use std::borrow::ToOwned;

use parser::{MarkdownParser, PhantomMark, Success, End, NoParse};
use tokens::*;
use util::CharOps;

use super::InlineParser;

pub trait EmphasisParser {
    fn parse_emphasis(&self, ec: u8, n: usize) -> Option<Inline>;
}

impl<'a> EmphasisParser for MarkdownParser<'a> {
    fn parse_emphasis(&self, ec: u8, n: usize) -> Option<Inline> {
        debug!("reading emphasis, char [{}], n = {}", ec as char, n);
        let pm = self.cur.phantom_mark();
        loop {
            // a marker over the first character of closing emphasis
            let pm_last = opt_ret!(self.until_emph_closing(ec, n));
            let slice = self.cur.slice(pm, pm_last);
            debug!("checking slice: [{}], n: {}", ::std::str::from_utf8(slice).unwrap(), n);

            // escaped closing emphasis
            if slice[slice.len()-1] != b' ' {
                if ec.is_code() {  // this is code inline
                    return Some(Code(str::from_utf8(slice).unwrap().to_owned()));
                } else {
                    let subp = self.fork(slice);
                    let result = self.fix_links(subp.parse_inline());
                    
                    return Some(match n {
                        1 => Emphasis(result),
                        2 => MoreEmphasis(result),
                        _ => unreachable!()  // for now
                    });
                }
            }
        }
    }
}

trait Ops<'a> {
    fn until_emph_closing(&self, ec: u8, n: usize) -> Option<PhantomMark>;
}

impl<'a> Ops<'a> for MarkdownParser<'a> {
    fn until_emph_closing(&self, ec: u8, n: usize) -> Option<PhantomMark> {
        assert!(n > 0);  // need at least one emphasis character

        let pm = self.cur.phantom_mark();
        let mut pm_last = pm;
        let mut escaping = false;

        macro_rules! advance {
            () => (pm_last = self.cur.phantom_mark())
        }
        macro_rules! retract {
            () => (pm_last = pm)
        }

        'outer: loop {
            let c = match self.cur.next_byte() {
                Some(c) => c,
                None => return None  // if we're here then we haven't found the closing "brace"
            };
            
            match c {
                // pass escaped characters as is
                b'\\' => escaping = true,
                _ if escaping => escaping = false,

                // found our emphasis character
                c if c == ec => {
                    // TODO: make something more pretty
                    self.cur.prev();
                    advance!();
                    self.cur.next();

                    let m = self.cur.mark();
                    let mut rn = 1;

                    loop {
                        match self.try_read_char(ec) {
                            Success(_) => rn += 1,
                            _ => break
                        }
                    }

                    if rn == n {  // this is our emphasis boundary, finish reading
                        m.cancel();
                        break;
                    } else {  
                        // otherwise reset to the character just after the one 
                        // we have just read
                        retract!();
                        m.reset();
                    }
                }

                // inline code block started, and we're not an inline code block ourselves
                // we need to pass through this block as is
                c if c == b'`' => {
                    // count `s
                    let mut sn = 1us;
                    while break_on_end!(self.try_read_char(b'`')).is_success() {
                        sn += 1;
                    }

                    let mut ec_mark = None;
                    
                    // read until closing delimiter
                    let mut tn = 0;
                    while tn < sn {
                        match self.cur.next_byte() {
                            Some(b'`') => tn += 1,
                            Some(cc) => {
                                tn = 0;
                                if cc == ec {
                                    advance!();
                                    ec_mark = Some(self.cur.mark());
                                }
                            }
                            None => break 'outer
                        }
                    }

                    retract!();
                    ec_mark.map(|m| m.cancel());
                }

                // skip hyperlinks
                c if c == b'[' => {
                    debug!("encountered link start");
                    let mut ec_mark = None;

                    // read until closing brace
                    loop {
                        match self.cur.next_byte() {
                            Some(b']') => break,
                            Some(c) if ec_mark.is_none() && c == ec && self.lookahead_chars(n-1, ec) => {
                                debug!("encountered emphasis inside link, setting a mark");
                                advance!();
                                ec_mark = Some(self.cur.mark());
                            }
                            Some(_) => {}
                            None => break 'outer
                        }
                    }
                    debug!("read first link part, skipping whitespace");

                    // skip whitespace between delimiting braces
                    parse_or_break!(self.skip_spaces_and_newlines());
                    debug!("skipped whitespace, current char: {}", *self.cur as char);

                    // determine closing brace for the second part of the link
                    let cc = match *self.cur {
                        b'[' => b']',
                        b'(' => b')',
                        _ => if ec_mark.is_some() { break 'outer } else { continue 'outer }
                    };
                    self.cur.next();

                    debug!("expected closing character is {}, skipping rest of the link", cc as char);
                    // skip second part of the link
                    loop {
                        match self.cur.next_byte() {
                            Some(c) if c == cc => break,
                            Some(c) if ec_mark.is_none() && c == ec && self.lookahead_chars(n-1, ec) => {
                                debug!("encountered emphasis inside link second part, setting a mark");
                                advance!();
                                ec_mark = Some(self.cur.mark());
                            }
                            Some(_) => {}
                            None => break 'outer
                        }
                    }
                    debug!("everything skipped, resetting marks");

                    retract!();
                    ec_mark.map(|m| m.cancel());
                }

                // just pass through any other character
                _ => {}
            }
        }

        if pm_last == pm { None } else { Some(pm_last) }
    }
}
