use std::collections::HashMap;

pub type Document = Vec<Block>;

pub type Text = Vec<Inline>;

pub type LinkMap = HashMap<String, LinkDescription>;

pub struct LinkDescription {
    pub id: String,
    pub link: String,
    pub title: Option<String>
}

#[deriving(PartialEq, Eq, Show)]
pub enum Block {
    Heading {
        pub level: uint,
        pub content: Text
    },
    
    BlockQuote(Document),

    BlockCode {
        pub tag: Option<String>,
        pub content: String
    },

    OrderedList(Vec<Document>),

    UnorderedList(Vec<Document>),

    Paragraph(Text),

    HorizontalRule,

    LinkDefinition {
        pub id: String,
        pub link: String,
        pub title: Option<String>
    }
}

#[deriving(PartialEq, Eq, Show)]
pub enum Inline{
    LineBreak,

    Chunk(String),

    Emphasis(Text),

    MoreEmphasis(Text),

    Code(String),

    Link {
        pub text: Option<Text>,  // None for automatic links
        pub link: Option<String>,
        pub title: Option<String>,
        pub id: Option<String>
    }
}

pub trait FixLinks {
    #[inline]
    fn fix_links_opt(&mut self, link_map: Option<&LinkMap>) {
        match link_map {
            Some(hm) => self.fix_links(hm),
            None => {}
        }
    }

    fn fix_links(&mut self, link_map: &LinkMap);
}

impl FixLinks for Block {
    fn fix_links(&mut self, link_map: &LinkMap) {
        match *self {
            BlockQuote(ref mut content) => content.fix_links(link_map),

            OrderedList(ref mut items) | UnorderedList(ref mut items) =>
                for item in items.mut_iter() {
                    item.fix_links(link_map);
                },

            Paragraph(ref mut content) | Heading { ref mut content, .. } => 
                content.fix_links(link_map),

            _ => {}
        }
    }
}

impl FixLinks for Document {
    fn fix_links(&mut self, link_map: &LinkMap) {
        for b in self.mut_iter() {
            b.fix_links(link_map);
        }
    }
}

impl FixLinks for Text {
    fn fix_links(&mut self, link_map: &LinkMap) {
        for i in self.mut_iter() {
            i.fix_links(link_map);
        }
    }
}

impl FixLinks for Inline {
    fn fix_links(&mut self, link_map: &LinkMap) {
        match *self {
            Emphasis(ref mut content) | MoreEmphasis(ref mut content) =>
                content.fix_links(link_map),

            Link { ref mut link, ref mut title, id: Some(ref id), .. } => {
                match link_map.find(id) {
                    Some(ld) => {
                        if link.is_none() {
                            *link = Some(ld.link.clone());
                        }
                        if title.is_none() && ld.title.is_none() {
                            *title = ld.title.clone();
                        }
                    }
                    None => {}
                }
            }
            
            _ => {}
        }
    }
}
