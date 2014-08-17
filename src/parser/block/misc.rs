use parser::{MarkdownParser, ParseResult, Success, End, NoParse};
use tokens::*;
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

        let mut m = self.cur.mark();
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
            match self.try_parse_empty_line() {
                Success(_) | End => break,
                NoParse => {}
            }

            // header line means that the paragraph ended, and its last line
            // should be parsed as a heading
            match self.try_parse_header_line() {
                Success(r) => { level = Some(r); break }
                End => break,  // End is in fact impossible here
                NoParse => {}
            }

            // TODO: check for atx header, hrule or quote

            // TODO: lax spacing rules: check for list/html, block/code fence or quote
        }

        let mut buf = self.cur.slice(pm, pm_last);

        match level {
            // extract last line from the buffer
            Some(level) => {
                // unwrap is safe because buf will contain at least one line in this case
                let nl_idx = buf.as_slice().rposition_elem(&b'\n').unwrap();
                let head_content = buf.slice_from(nl_idx+1);

                let subp = MarkdownParser::new(head_content);
                let result = subp.parse_inline();

                let heading_result = Heading {
                    level: level.to_numeric(),
                    content: result
                };

                buf = buf.slice_to(nl_idx+1);
                self.event_queue.borrow_mut().push(heading_result);
            }
            None => {}
        }

        let subp = MarkdownParser::new(buf);
        let result = subp.parse_inline();

        Success(Paragraph(result))
    }
}

trait Ops {
    fn try_parse_header_line(&self) -> ParseResult<SetextHeaderLevel>;
}

impl<'a> Ops for MarkdownParser<'a> {
    fn try_parse_header_line(&self) -> ParseResult<SetextHeaderLevel> {
        let mut m = self.cur.mark();

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
