impl MarkdownParser {
    fn block_code(&mut self) -> ParseResult<Block> {
        fn block_code_prefix<R: Reader>(this: &mut MarkdownParser<R>) -> ParseResult<()> {
            let mut n = 0u;
            this.mark().set();
            loop {
                if n == 4 { break };
                match try_err_f!(this.read_char(b' ')) {
                    NoParse => { this.mark().reset(); return NoParse },
                    _ => n += 1
                }
            }
            this.mark().unset();
            Success(())
        }

        debug!(">> trying code block");
        try_parse!(try_err_f!(block_code_prefix(self)));
        self.backtrack();

        let mut buf = Vec::new();
        let mut err = None;
        loop {
            match break_err!(block_code_prefix(self); -> err) {
                NoParse => {  // no prefix, check for emptiness
                    self.mark().set();
                    match break_err!(self.try_parse_empty_line(); -> err) {
                        // non-empty line without prefix
                        NoParse => { self.mark().reset(); break }
                        // empty line without prefix, add newline
                        _ => { self.mark().unset(); buf.push(b'\n') }
                    }
                }
                // prefix is ok, read everything else
                _ => { break_err!(self.read_line_pr(&mut buf); -> err); }
            }
        }

        self.consume();

        // TODO: handle UTF-8 decoding error
        let result = BlockCode { tag: None, content: String::from_utf8(buf).unwrap() };
        match err {
            Some(e) => PartialSuccess(result, e),
            None => Success(result)
        }
    }
}
