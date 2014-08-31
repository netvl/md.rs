use parser::{MarkdownParser, ParseResult, Success, End, NoParse};
use tokens::*;
use parser::block::atx_heading::AtxHeadingParser;
use parser::block::block_quote::BlockQuoteParser;
use parser::inline::InlineParser;

pub trait MiscParser {
    fn parse_horizontal_rule(&self) -> ParseResult<Block>;
    fn parse_paragraph(&self) -> ParseResult<Block>;
}

#[repr(u8)]
#[deriving(FromPrimitive)]
enum SetextHeaderLevel {
    StxFirst = b'=',
    StxSecond = b'-'
}

impl SetextHeaderLevel {
    #[inline]
    fn to_numeric(self) -> uint {
        match self {
            StxFirst => 1,
            StxSecond => 2
        }
    }
}

impl<'a> MiscParser for MarkdownParser<'a> {
    fn parse_horizontal_rule(&self) -> ParseResult<Block> {
        debug!(">> trying hrule");
        parse_or_ret!(self.try_skip_initial_spaces());

        let m = self.cur.mark();
        match self.cur.next_byte() {
            Some(mut c) if one_of!(c, b'-', b'*', b'_')  => {
                loop {
                    match self.cur.next_byte() {
                        Some(b'\n') | None => break,
                        Some(b' ') => c = b' ',  // from now on everything should be spaces
                        Some(cc) if cc == c => {}
                        Some(_) => return NoParse
                    }
                }
                m.cancel();
                Success(HorizontalRule)
            }
            Some(_) => NoParse,
            None => End
        }
    }

    fn parse_paragraph(&self) -> ParseResult<Block> {
        debug!(">> reading paragraph");

        let pm = self.cur.phantom_mark();
        let mut pm_last = pm;
        let mut level = None;

        loop {
            parse_or_break!(self.read_line());
            pm_last = self.cur.phantom_mark();

            // empty line means paragraph end
            debug!(">> trying to parse empty line");
            match self.try_parse_empty_line() {
                Success(_) | End => break,
                NoParse => {}
            }

            // header line means that the paragraph ended, and its last line
            // should be parsed as a heading
            debug!(">> trying to parse header line");
            match self.try_parse_header_line() {
                Success(r) => { level = Some(r); break }
                End => break,  // End is in fact impossible here
                NoParse => {}
            }

            // Check for ATX heading just after the paragraph
            debug!(">> trying to parse ATX heading");
            match self.parse_atx_heading() {
                Success(heading) => {
                    self.enqueue_event(heading);
                    break
                }
                End => break,   // End is impossible here
                NoParse => {}
            }

            // Check for horizontal rule just after the paragraph
            debug!(">> trying to parse horizontal rule");
            match self.parse_horizontal_rule() {
                Success(hrule) => {
                    self.enqueue_event(hrule);
                    break
                }
                End => break,   // End is impossible here
                NoParse => {}
            }

            // Check for block quote just after the paragraph
            debug!(">> trying to parse block quote");
            match self.parse_block_quote() {
                Success(quote) => {
                    self.enqueue_event(quote);
                    break
                }
                End => break,   // End is impossible here
                NoParse => {}
            }

            // TODO: lax spacing rules: check for list/html, block/code fence or quote
        }

        let mut buf = self.cur.slice(pm, pm_last);
        debug!("read paragraph, contents: [{}]", ::std::str::from_utf8(buf).unwrap());

        match level {
            // extract last line from the buffer
            Some(level) => {
                debug!("found setext header of level {}", level.to_numeric());

                // ignore last newline which is always there
                let sbuf = if buf.ends_with(b"\n") { buf.slice_to(buf.len()-1) } else { buf };

                // last newline or start of the block
                let after_nl_idx = sbuf.rposition_elem(&b'\n').map(|i| i + 1).unwrap_or(0);
                let head_content = sbuf.slice_from(after_nl_idx);

                let subp = self.fork(head_content);
                let result = self.fix_links(subp.parse_inline());

                let heading_result = Heading {
                    level: level.to_numeric(),
                    content: result
                };

                buf = buf.slice_to(after_nl_idx);

                if buf.is_empty() {
                    return Success(heading_result);
                } else {
                    self.enqueue_event(heading_result);
                }
            }
            None => {}
        }

        let subp = self.fork(buf);
        let result = self.fix_links(subp.parse_inline());

        Success(Paragraph(result))
    }
}

trait Ops {
    fn try_parse_header_line(&self) -> ParseResult<SetextHeaderLevel>;
}

impl<'a> Ops for MarkdownParser<'a> {
    fn try_parse_header_line(&self) -> ParseResult<SetextHeaderLevel> {
        let m = self.cur.mark();

        let mut cc = match self.cur.next_byte() {
            Some(c) if one_of!(c, b'=', b'-') => c,
            Some(_) => return NoParse,
            None => return End
        };
        let level = FromPrimitive::from_u8(cc).unwrap();  // unwrap is safe due to check above

        loop {
            match self.cur.next_byte() {
                None | Some(b'\n') => break,
                Some(c) if c == cc => {},
                Some(b' ') => cc = b' ',  // consume only spaces from now on
                Some(_) => return NoParse
            }
        }
        m.cancel();
        Success(level)
    }
}
