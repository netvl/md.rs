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

impl MarkdownParser {
    fn horizontal_rule(&mut self) -> ParseResult<Block> {
        debug!(">> trying hrule");
        try_reset!(self; self.skip_initial_spaces());

        let mut c = iotry_err!(self.read_byte()).unwrap();
        if c == b'-' || c == b'*' || c == b'_' {
            loop {
                match iotry_err!(self.read_byte()).unwrap() {
                    b'\n' => break,
                    b' ' => c = b' ',  // from now on everything should be spaces
                    cc if cc == c => {}
                    _ => { self.backtrack(); return NoParse }
                }
            }
            self.consume();
            Success(HorizontalRule)
        } else {
            self.backtrack();
            NoParse
        }
    }

    fn paragraph(&mut self) -> ParseResult<Block> {
        debug!(">> reading paragraph");

        let mut buf = Vec::new();
        let mut err0 = None;
        let mut level = None;
        loop {
            break_err!(self.read_line_pr(&mut buf); -> err0);

            // empty line means paragraph end
            match break_err!(self.try_parse_empty_line(); -> err0) {
                Success(_) => break,
                _ => {}
            }

            // header line means that the paragraph ended, and its last line
            // should be parsed as a heading
            match break_err!(self.try_parse_header_line(); -> err0) {
                Success(r) => { level = Some(r); break }
                _ => {}
            }

            // TODO: check for atx header, hrule or quote

            // TODO: lax spacing rules: check for list/html block/code fence
        }

        match level {
            // extract last line from the buffer
            Some(level) => {
                // unwrap is safe because buf will contain at least one
                // line in this case
                let nl_idx = buf.as_slice().rposition_elem(&b'\n').unwrap();
                let head_content = buf.slice_from(nl_idx+1).to_vec();
                buf.truncate(nl_idx+1);

                self.push_existing(head_content);
                let result = self.inline();
                self.pop_buf();

                let heading_result = result.map(|r| Heading {
                    level: level.to_numeric(),
                    content: r
                });
                self.event_queue.push(heading_result.to_md_result());
            }
            None => {}
        }

        self.push_existing(buf);
        let result = self.inline();
        self.pop_buf();
        self.consume();

        match (err0, result) {
            (Some(e), Success(r)) => PartialSuccess(Paragraph(r), e),
            (Some(e), PartialSuccess(r, _)) => PartialSuccess(Paragraph(r), e),
            (Some(e), _) => Failure(e),
            (None, r) => r.map(Paragraph)
        }
    }

    fn try_parse_header_line(&mut self) -> ParseResult<SetextHeaderLevel> {
        self.mark().set();

        // TODO: is it important that the mark can be left set?
        let mut cc = match iotry_err!(self.read_byte()).unwrap() {
            c if c == b'=' || c == b'-' => c,
            _ => { self.mark().reset(); return NoParse }
        };
        let level = FromPrimitive::from_u8(cc).unwrap();

        loop {
            match iotry_err!(self.read_byte()).unwrap() {
                c if c == cc => {},
                b' ' => cc = b' ',  // consume only spaces from now on
                b'\n' => { self.mark().unset(); return Success(level) },
                _ => { self.mark().reset(); return NoParse }
            }
        }
    }
}
