use parser::{MarkdownParser, PhantomMark, ParseResult, Success, End, NoParse};
use tokens::*;
use util::CharOps;

use super::InlineParser;

pub trait EmphasisParser {
    fn parse_emphasis(&self, ec: u8, n: uint) -> Option<Inline>;
}

impl<'a> EmphasisParser for MarkdownParser<'a> {
    fn parse_emphasis(&self, ec: u8, n: uint) -> Option<Inline> {
        debug!("reading emphasis, char [{}], n = {}", ec.to_ascii(), n);
        let pm = self.cur.phantom_mark();
        loop {
            // a marker over the first character of closing emphasis
            let pm_last = opt_ret!(self.until_emph_closing(ec, n));
            let slice = self.cur.slice(pm, pm_last);
            debug!("checking slice: [{}], n: {}", ::std::str::from_utf8(slice).unwrap(), n);

            // escaped closing emphasis
            if slice[slice.len()-1] != b' ' {
                if ec.is_code() {  // this is code inline
                    return Some(Code(String::from_utf8(slice.to_vec()).unwrap()));
                } else {
                    let subp = MarkdownParser::new(slice);
                    let result = subp.parse_inline();
                    
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
    fn until_emph_closing(&self, ec: u8, n: uint) -> Option<PhantomMark>;
}

impl<'a> Ops<'a> for MarkdownParser<'a> {
    fn until_emph_closing(&self, ec: u8, n: uint) -> Option<PhantomMark> {
        assert!(n > 0);  // need at least one emphasis character

        let pm = self.cur.phantom_mark();
        let mut pm_last = pm;
        let mut escaping = false;

        macro_rules! advance(
            () => (pm_last = self.cur.phantom_mark())
        )

        'outer: loop {
            let c = opt_break!(self.cur.next_byte());
            
            match c {
                // pass escaped characters as is
                b'\\' => { advance!(); escaping = true }
                _ if escaping => { advance!(); escaping = false }

                // found our emphasis character
                c if c == ec => {
                    let mut m = self.cur.mark();
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
                        m.reset();
                        advance!();
                    }
                }

                // inline code block started, and we're not an inline code block ourselves
                // we need to pass through this block as is
                c if c == b'`' => {
                    advance!();

                    // count `s
                    let mut sn = 1u;
                    while break_on_end!(self.try_read_char(b'`')).is_success() {
                        sn += 1;
                    }
                    
                    // read until closing delimiter
                    let mut tn = 0;
                    while tn < sn {
                        match self.cur.next_byte() {
                            Some(b) => {
                                advance!();
                                match b {
                                    b'`' => tn += 1,
                                    _    => tn = 0
                                }
                            }
                            None => break 'outer
                        }
                    }
                }

                // TODO: skip hyperlink

                // just pass through any other character
                _ => advance!()
            }
        }

        if pm_last == pm { None } else { Some(pm_last) }
    }
}
