impl MarkdownParser {
    fn atx_heading(&mut self) -> ParseResult<Block> {
        debug!(">> trying atx header");
        try_parse!(check_reset!(self; try_err_f!(self.read_char(b'#'))));
        self.unread_byte();

        // read and count hashes
        let mut n = 0;
        let mut err = None;
        while n < 6 {
            match break_err!(self.read_byte_pr(); -> err).unwrap() {
                b'#' => n += 1,
                _ => { self.unread_byte(); break }
            }
        }

        let mut buf = Vec::new();

        // skip spaces after hashes
        err = err.or_else(|| match self.skip_spaces() {
            PartialSuccess(_, e) | Failure(e) => Some(e),
            _ => None
        // read the rest of the line
        }).or_else(|| match self.read_line_pr(&mut buf) {
            Failure(e) => Some(e),
            _ => { 
                buf.pop();  // remove newline
                None
            }
        });

        // if there were errors, return partial success with empty text
        match err {
            Some(e) => return PartialSuccess(Heading {
                level: n,
                content: Vec::new()
            }, e),
            None => {}
        }
         
        // skip hashes and spaces backwards
        while buf.len() > 0 {
            match *buf.last().unwrap() {
                b'#' => { buf.pop(); }
                _ => break
            }
        }
        while buf.len() > 0 {
            match *buf.last().unwrap() {
                b' ' => { buf.pop(); }
                _ => break
            }
        }

        // parse header contents
        self.push_existing(buf);
        let result = self.inline();
        self.pop_buf();
        self.consume();

        result.map(|r| Heading {
            level: n,
            content: r
        })
    }
}
