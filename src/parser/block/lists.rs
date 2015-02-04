use parser::{MarkdownParser, ParseResult, Success, End, NoParse};
use tokens::*;

use self::ListItemInfo::*;

use util::CharOps;

#[derive(Copy)]
enum ListItemInfo {
    Ordered {
        start: usize
    }, 
    Unordered {
        marker: u8
    },
    Unknown
}

pub trait ListsParser {
    fn parse_list(&self) -> ParseResult<Block>;
}

impl<'a> ListsParser for MarkdownParser<'a> {
    fn parse_list(&self) -> ParseResult<Block> {
        let mut result = Vec::new();
        let mut current_item = Unknown;
        loop {
            let m = self.cur.mark();
            match self.parse_list_item(current_item) {
                Success((d, i)) => {
                    result.push(d);
                    current_item = i;
                    m.cancel();
                }
                NoParse | End => break
            }
        }

        match current_item {
            Unknown => NoParse,
            Ordered { start } => Success(OrderedList {
                start_index: start,
                items: result
            }),
            Unordered { .. } => Success(UnorderedList {
                items: result
            })
        }
    }
}

trait Ops {
    fn parse_list_item(&self, current_item: ListItemInfo) 
        -> ParseResult<(Document, ListItemInfo)>;
    fn parse_list_item_content(&self) -> ParseResult<Document>;
}

impl<'a> Ops for MarkdownParser<'a> {
    fn parse_list_item(&self, current_item: ListItemInfo) 
            -> ParseResult<(Document, ListItemInfo)> {
        parse_or_ret!(self.try_skip_initial_spaces());

        //match current_item {
            //Ordered { .. } => {}
            //Unordered { marker } => {}
            //Unknown => {
                //match self.parse(|c: u8| c.is_numeric()) {
                    //Success(n) => {
                        //let start: usize = from_str(n).unwrap();
                        //self.parse_list_item_content()
                            //.map(|d| (d, Ordered { start: start }))
                    //}
                //}
            //}
        //}
        unimplemented!()
    }

    fn parse_list_item_content(&self) -> ParseResult<Document> {
        unimplemented!()
    }
}
