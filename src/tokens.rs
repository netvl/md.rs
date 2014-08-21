pub type Document = Vec<Block>;

pub type Text = Vec<Inline>;

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

    InlineLink {
        pub text: Option<Text>,  // None for automatic links
        pub link: String,
        pub title: Option<String>
    },

    ReferenceLink {
        pub text: String,
        pub id: String
    }

}
