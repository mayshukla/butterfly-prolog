use pest::Parser;
use pest::iterators::Pair;

use crate::ast::*;

#[derive(Parser)]
#[grammar = "grammar.pest"]
struct ButterflyPLParser;

/**
 * Top-level function for parsing a program.
 */
pub fn parse(code: &str) -> Result<Program, &str> {
    let parsed_program = ButterflyPLParser::parse(Rule::program, code)
        .expect("Parsing error.")
        .next()
        .unwrap();

    let clauses = Vec::new();
    for parsed_clause in parsed_program.into_inner() {
        /*
        let parsed_head = parsed_clause.into_inner().next().unwrap();
        let clause = Clause {

        }
        */
    }

    Ok(clauses)
}

fn construct_term(pair: Pair<Rule>) -> Term {
    /*
    match pair.as_rule() {
        Rule::compound_term =>
        Rule::simple_term =>
    }
    */
    todo!()
}

fn construct_simple_term(pair: Pair<Rule>) -> SimpleTerm {
    match pair.as_rule() {
        Rule::atom => SimpleTerm::Atom(pair.as_str().to_string()),
        Rule::variable => SimpleTerm::Variable(pair.as_str().to_string()),
        _ => unreachable!()
    }
}

fn construct_compound_term(pair: Pair<Rule>) -> CompoundTerm {
    todo!()
}

#[cfg(test)]
mod tests {
    use crate::parser::*;

    fn parse_and_unwrap(rule: Rule, code: &str) -> Pair<Rule> {
        ButterflyPLParser::parse(rule, code)
            .unwrap()
            .next()
            .unwrap()
    }

    #[test]
    fn test_construct_simple_term_atom() {
        let pair = parse_and_unwrap(Rule::simple_term, "abc");
        let term = construct_simple_term(pair);
        assert_eq!(term, SimpleTerm::Atom(String::from("abc")));
    }

    #[test]
    fn test_construct_simple_term_variable() {
        let pair = parse_and_unwrap(Rule::simple_term, "Abc");
        let term = construct_simple_term(pair);
        assert_eq!(term, SimpleTerm::Variable(String::from("Abc")));
    }

    #[test]
    fn test_construct_simple_term_underscore() {
        let pair = parse_and_unwrap(Rule::simple_term, "_abc");
        let term = construct_simple_term(pair);
        assert_eq!(term, SimpleTerm::Variable(String::from("_abc")));
    }

    #[test]
    #[should_panic]
    fn test_construct_simple_term_panic() {
        let pair = parse_and_unwrap(Rule::simple_term, "#abc");
        let term = construct_simple_term(pair);
    }
}