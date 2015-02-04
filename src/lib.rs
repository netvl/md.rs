#![feature(unsafe_destructor, core, collections)]

#[macro_use] extern crate log;

pub use tokens::*;
pub use parser::MarkdownParser;

mod util;

pub mod tokens;
pub mod parser;


