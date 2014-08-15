
impl MarkdownParser {
    fn inline(&mut self) -> ParseResult<Text> {
        let mut tokens = Vec::new();
        let mut buf = Vec::new();
        let mut err;
        let mut escaping = false;
        let mut space = false;

        macro_rules! continue_with(
            ($($c:expr),+) => ({
                $(buf.push($c);)+
                continue
            })
        )

        fn find_emph_closing<R: Reader>(this: &mut MarkdownParser<R>, ec: u8, n: uint) 
            -> ParseResult<Vec<u8>> {
            assert!(n > 0);  // need at least one emphasis character

            let mut result = Vec::new();
            let mut err = None;
            let mut escaping = false;

            'outer: loop {
                match break_err!(this.read_byte_pr(); -> err).unwrap() {
                    // pass escaped characters as is
                    b'\\' => { result.push(b'\\'); escaping = true }
                    c if escaping => { result.push(c); escaping = false }

                    // found our emphasis character
                    c if c == ec => {
                        let mut rn = 1;
                        this.mark().set();
                        while break_err!(this.read_char(ec); -> err).is_success() {
                            rn += 1;
                        }

                        if rn == n { // this is our emphasis boundary, finish reading
                            this.unread_byte();
                            this.mark().unset();
                            break;
                            
                        } else { // otherwise reset to the character just after the one we just have read
                            this.mark().reset();
                            result.push(c);
                        }
                    }

                    // inline code block started, and we're not inline code block ourselves
                    // we need to pass through this block as is
                    c if c == b'`' => {
                        result.push(b'`');

                        // count `s
                        let mut sn = 1u;
                        while break_err!(this.read_char(b'`'); -> err).is_success() {
                            result.push(b'`');
                            sn += 1;
                        }
                        this.unread_byte();

                        let mut tn = 0;
                        while tn < sn {
                            match this.read_byte() {
                                Ok(b) => {
                                    result.push(b);
                                    match b {
                                        b'`' => tn += 1,
                                        _    => tn = 0
                                    }
                                },

                                Err(e) => {
                                    err = Some(IoError(e));
                                    break 'outer;
                                }
                            }
                        }

                    }

                    // TODO: skip hyperlinks

                    // just pass through any other character
                    c => result.push(c)
                }
            }

            match err {
                Some(e) => PartialSuccess(result, e),
                None => Success(result)
            }
        }

        loop {
            match break_err!(self.read_byte_pr(); -> err).unwrap() {
                b' ' => { space = true; continue_with!(b' ') }

                b'\\' => { escaping = true; continue }
                c if escaping => { escaping = false; continue_with!(c) }

                c if c.is_emphasis() || c.is_code() => { 
                    // TODO: handle UTF-8 decoding error
                    tokens.push(Chunk(
                        String::from_utf8(mem::replace(&mut buf, Vec::new())).unwrap()
                    ));
                    self.consume();

                    if space {
                        space = false;
                        match break_err!(self.read_byte_pr(); -> err).unwrap() {
                            b' ' => { 
                                self.unread_byte();
                                continue_with!(c)  // this character is escaped
                            }
                            _ => self.unread_byte()
                        }
                    } 
                    
                    // one or two emphasis characters
                    let mut n = 1;
                    if break_err_check!(self; self.read_char(c); -> err).is_success() {
                        n += 1;
                    }

                    // read everything until closing emphasis bracket
                    let buf = break_err!(find_emph_closing(self, c, n); -> err).unwrap();
                    let result = if c.is_code() {  // code block
                        self.consume();
                        Code(String::from_utf8(buf).unwrap())  // TODO: handle UTF8 errors
                    } else {
                        self.push_existing(buf);
                        let result = break_err!(self.inline(); -> err).unwrap();
                        self.pop_buf();
                        self.consume();  // up to and including closing emphasis
                        match n {
                            1 => Emphasis(result),
                            2 => MoreEmphasis(result),
                            _ => unreachable!()
                        }
                    };

                    tokens.push(result)
                }

                // push plain characters to the buffer
                c => buf.push(c)
            }
        }

        if buf.len() > 0 {
            tokens.push(Chunk(String::from_utf8(buf).unwrap()));
        }

        match err {
            // TODO: handle UTF-8 decoding errors
            None => Success(tokens),
            Some(IoError(ref e)) if e.kind == io::EndOfFile => Success(tokens),
            Some(e) => PartialSuccess(tokens, e)
        }
    }
}
