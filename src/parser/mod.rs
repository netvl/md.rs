pub use tokens::*;

pub struct MarkdownParser<'a> {
    buf: &'a [u8],
    stack: BufStack,
    event_queue: Vec<MarkdownResult<Block>>
}

impl<'a> MarkdownParser<'a> {
    pub fn new(buffer: &[u8]) -> MarkdownParser {
        MarkdownParser {
            buf: buffer,
            stack: BufStack::new(),
            event_queue: Vec::new()
        }
    }
    
    pub fn tokens(self) -> MarkdownTokens<R> {
        MarkdownTokens { parser: self }
    }
}

pub struct MarkdownTokens<'a> {
    parser: MarkdownParser<'a>
}

impl<'a> Iterator<Block> for MarkdownTokens<'a> {
    #[inline]
    fn next(&mut self) -> Option<Block> {
        self.parser.next().to_result().ok()
    }
}

