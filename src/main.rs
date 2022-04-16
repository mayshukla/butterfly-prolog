extern crate pest;
#[macro_use]
extern crate pest_derive;

use pest::Parser;

#[derive(Parser)]
#[grammar = "grammar.pest"]
struct MyParser;

fn main() {
    let result = MyParser::parse(Rule::program, "abc\n\tAbc \n");
    println!("{:?}", result);

    let result = MyParser::parse(Rule::program, "# comment \n abc Y X if abc X Y and def \n # comment");
    println!("{:?}", result);
}
