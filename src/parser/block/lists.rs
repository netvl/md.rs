use parser::{MarkdownParser, ParseResult, Success, End, NoParse};
use tokens::*;

enum ListItemInfo {
    Ordered {
        start: uint
    }, 
    Unordered {
        marker: u8
    }
}

pub trait ListsParser {
    fn parse_list(&self) -> ParseResult<Block>;
}

impl<'a> ListsParser for MarkdownParser<'a> {
    fn parse_list(&self) -> ParseResult<Block> {
        let mut result = Vec::new();
        let mut current_item = None;
        loop {
            let m = self.cur.mark();
            match self.parse_list_item(current_item) {
                Success((d, i)) => {
                    result.push(d);
                    current_item = Some(i);
                }
                NoParse | End => break
            }
        }

        match current_item {
            None => NoParse,
            Some(Ordered { start }) => Success(OrderedList {
                start_index: start,
                items: result
            }),
            Some(Unordered { .. }) => Success(UnorderedList {
                items: result
            })
        }
    }
}

trait Ops {
    fn parse_list_item(&self, current_item: Option<ListItemInfo>) -> ParseResult<(Document, ListItemInfo)>;
}

impl<'a> Ops for MarkdownParser<'a> {
    fn parse_list_item(&self, current_item: Option<ListItemInfo>) -> ParseResult<(Document, ListItemInfo)> {
        unimplemented!()
    }
}
