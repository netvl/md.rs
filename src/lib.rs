#![feature(globs, macro_rules, phase, unsafe_destructor)]

#[phase(plugin, link)] extern crate log;
extern crate collections;

pub use tokens::*;
pub use parser::MarkdownParser;

mod util;

pub mod tokens;
pub mod parser;


