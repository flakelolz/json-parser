use json_parser::parser::JsonParser;
use std::fs::File;

fn main() {
    let file = File::open("test.json").unwrap();
    let parser = JsonParser::parse_from_file(file).unwrap();

    dbg!(parser);
}
