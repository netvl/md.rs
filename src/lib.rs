#![feature(struct_variant, globs, macro_rules, phase)]

#[phase(plugin, link)] extern crate log;
extern crate collections;

use std::io;
use std::mem;

pub use tokens::*;
pub use parser::MarkdownParser;

mod macros;
mod util;

pub mod tokens;
pub mod result;
pub mod parser;


