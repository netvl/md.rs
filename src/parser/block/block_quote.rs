impl MarkdownParser {
    fn block_quote_prefix<R: Reader>(&mut self) -> ParseResult<()> {

    }

    fn block_quote(&mut self) -> ParseResult<Block> {
        fn block_quote_prefix<R: Reader>(this: &mut MarkdownParser<R>) -> ParseResult<()> {
            try_reset!(this; this.skip_initial_spaces());
            try_reset!(this; this.read_char(b'>'));
            attempt!(this.try_read_char(b' ')) // optional space after >
        }

        debug!(">> trying blockquote");
        try_parse!(try_err_f!(block_quote_prefix(self)));
        self.backtrack();

        let mut buf = Vec::new();
        let mut err0 = None;
        loop {
            break_err!(block_quote_prefix(self); -> err0);
            break_err!(self.read_line_pr(&mut buf); -> err0);

            // break if there is an empty line followed by non-quote line after this line
            match break_err!(self.try_parse_empty_line(); -> err0) {
                Success(_) => {
                    self.mark().set();
                    match break_err!(block_quote_prefix(self); -> err0) {
                        NoParse => { self.mark().reset(); break }
                        _ => self.mark().reset()
                    }
                }
                _ => {}
            }
        }

        self.push_existing(buf);
        let (result, err) = self.read_while_possible();
        self.pop_buf();
        self.consume();

        // TODO: validate this table and errors priority
        match (err0, err, result.is_empty()) {
            (_,       Some(e), true) => Failure(e),
            (_,       Some(e), false) => PartialSuccess(BlockQuote(result), e),
            (Some(e), None,    _) => PartialSuccess(BlockQuote(result), e),
            (None,    None,    _) => Success(BlockQuote(result))
        }
    }

}
