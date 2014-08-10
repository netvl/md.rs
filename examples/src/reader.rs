extern crate md;

use std::os;
use std::io::File;

use md::MarkdownParser;

fn main() {
    let args = os::args();
    if args.len() < 2 {
        return;
    }

    let f = File::open(&Path::new(args[1].as_slice()));
    let p = MarkdownParser::new(f);

    for t in p.tokens() {
        println!("Read token: {}", t);
    }
}
