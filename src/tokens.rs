pub type Document = Vec<Block>;

pub type Text = Vec<Inline>;

pub enum Block {
    Heading {
        pub level: int,
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

pub enum Inline{
    LineBreak,

    Chunk(String),

    Emphasis(Text),

    MoreEmphasis(Text),

    Code(String),

    InlineLink {
        pub title: Option<Text>,  // None for automatic links
        pub link: String
    },

    ReferenceLink {
        pub title: String,
        pub id: String
    }

}
