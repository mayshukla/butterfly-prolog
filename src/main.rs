extern crate pest;
#[macro_use]
extern crate pest_derive;

use pest::Parser;

mod ast;

#[derive(Parser)]
#[grammar = "grammar.pest"]
struct ButterflyPLParser;

fn main() {
    let result = ButterflyPLParser::parse(Rule::program, "abc\n\tAbc \n");
    println!("{:?}", result);

    let result = ButterflyPLParser::parse(Rule::program, "# comment \n abc Y X if abc X Y and def \n # comment");
    println!("{:?}", result);
}
