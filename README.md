md.rs: Markdown parser in Rust
==============================

`md.rs` is intended to be a simple Markdown parser in Rust. It will provide token-based stream parser capabilities.

How to build
------------

`md.rs` uses Cargo, so just make a dependency on it in your `Cargo.toml` manifest:

    [dependencies.md]
    git = "https://github.com/netvl/md.rs"

How to use
----------

The main object in the library is `md::MarkdownParser` struct. It implements
`Iterator<md::tokens::Block>` trait, so you can use it in `for` loop:

    extern crate md;

    use std::io::File;

    use md::MarkdownParser;
    use md::tokens::Heading;

    fn main() {
        let mut f = File::open("/some/markdown/document.md").unwrap();
        let buf = f.read_to_end().unwrap();

        let mut p = MarkdownParser::new(buf.as_slice());
        for token in p {
            match token {
                Heading { level, content } =>
                    println!("Heading level {}, content: {}", level, content),
                _ =>
            }
        }
    }

See example programs in `examples` subpackage.

License
-------

This library is licensed under MIT license.

---
Copyright (C) Vladimir Matveev, 2014
