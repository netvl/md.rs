extern crate md;

use std::os;
use std::io::File;

use md::MarkdownParser;

fn main() {
    let args = os::args();
    if args.len() < 2 {
        return;
    }

    let mut f = File::open(&Path::new(args[1].as_slice()));
    let buf = f.read_to_end().unwrap();
    let mut p = MarkdownParser::new(buf.as_slice());

    for t in p {
        println!("Read token: {}", t);
    }
}
