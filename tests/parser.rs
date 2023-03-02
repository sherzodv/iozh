extern crate pest;
#[macro_use]
extern crate pest_derive;

use pest::Parser;
#[derive(Parser)]
#[grammar = "iozh.pest"]
pub struct IozhParser;

#[test]
fn test_identifier_parser() {
    let id1 = IozhParser::parse(Rule::identifier, "boo");
    let id2 = IozhParser::parse(Rule::identifier, "boo foo");
    let id3 = IozhParser::parse(Rule::identifier, "foo boo");

    println!("{:?}", id1);
    println!("{:?}", id2);
    println!("{:?}", id3);
}
