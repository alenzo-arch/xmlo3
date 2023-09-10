use std::fs;
mod parser;
use parser::parse;
fn main() {
    let doc = fs::read_to_string("./files/note.xml").unwrap();
    parse(doc);
}
