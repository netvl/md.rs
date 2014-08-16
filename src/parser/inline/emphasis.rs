use parser::{MarkdownParser, PhantomMark, ParseResult, Success, End, NoParse};
use tokens::*;
use util::CharOps;

use super::InlineParser;

pub trait EmphasisParser {
    fn parse_emphasis(&self, ec: u8, n: uint) -> Option<Inline>;
}

impl<'a> EmphasisParser for MarkdownParser<'a> {
    fn parse_emphasis(&self, ec: u8, n: uint) -> Option<Inline> {
        let pm = self.cur.phantom_mark();
        loop {
            // a marker over the first character of closing emphasis
            let pm_last = opt_ret!(self.until_emph_closing(ec, n));
            let slice = pm.slice_to(&pm_last);
            self.cur.advance(n);  // it is safe to advance by n here

            // escaped closing emphasis
            if slice[slice.len()-1] != b' ' {
                if ec.is_code() {  // this is code inline
                    return Some(Code(String::from_utf8(slice.to_vec()).unwrap()));
                } else {
                    let mut subp = MarkdownParser::new(slice);
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
    fn until_emph_closing<'b>(&'b self, ec: u8, n: uint) -> Option<PhantomMark<'b, 'a>>;
}

impl<'a> Ops<'a> for MarkdownParser<'a> {
    fn until_emph_closing<'b>(&'b self, ec: u8, n: uint) -> Option<PhantomMark<'b, 'a>> {
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
                c if escaping => { advance!(); escaping = false }

                // found our emphasis character
                c if c == ec => {
                    let mut m = self.cur.mark();
                    let mut rn = 1;

                    while on_end!(self.try_read_char(ec) <- m.cancel(); advance!(); break).is_success() {
                        rn += 1;
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

        if pm_last.same_pos_as(&pm) { None } else { Some(pm_last) }
    }
}
