impl MarkdownParser {
    fn ordered_list(&mut self) -> ParseResult<Block> {
        NoParse
    }

    fn unordered_list(&mut self) -> ParseResult<Block> {
        NoParse
    }
}
