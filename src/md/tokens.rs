
pub enum TopLevel {
    Heading {
        pub level: int,
        pub content: Vec<InlineToken>
    },
    
    BlockQuote(Vec<String>),  // TODO: maybe Vec<String>?

    BlockCode {
        pub tag: Option<String>,
        pub content: Vec<String>
    },

    OrderedList(Vec<Vec<TopLevel>>),

    UnorderedList(Vec<Vec<TopLevel>>),

    Paragraph(Vec<InlineToken>),

    HorizontalRule,

    LinkDefinition {
        pub id: String,
        pub link: String,
        pub title: Option<String>
    }
}

pub enum InlineToken {
    LineBreak,

    Text(String),

    Emphasis(String),

    MoreEmphasis(String),

    Code(String),

    InlineLink {
        pub title: Option<String>,  // None for automatic links
        pub link: String
    },

    ReferenceLink {
        pub title: String,
        pub id: String
    }

}
