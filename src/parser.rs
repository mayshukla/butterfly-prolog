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

    let mut clauses = Vec::new();
    for pair in parsed_program.into_inner() {
        let clause = construct_clause(pair);
        clauses.push(clause);
    }

    Ok(clauses)
}

fn construct_clause(pair: Pair<Rule>) -> Clause {
    let mut it = pair.into_inner();
    let head = construct_term(it.next().unwrap());

    let mut body = Vec::new();
    let mut it = it.next().unwrap().into_inner();
    for pair in it {
        body.push(construct_term(pair));
    }

    Clause { head, body }
}

fn construct_term(pair: Pair<Rule>) -> Term {
    match pair.as_rule() {
        Rule::atom => Term::Simple(SimpleTerm::Atom(pair.as_str().to_string())),
        Rule::variable => Term::Simple(SimpleTerm::Variable(pair.as_str().to_string())),
        Rule::compound_term => construct_compound_term(pair),
        Rule::term => construct_term(pair),
        _ => unreachable!()
    }
}

fn construct_compound_term(pair: Pair<Rule>) -> Term {
    let mut it = pair.into_inner();
    let name = match construct_term(it.next().unwrap()) {
        Term::Compound(_) => unreachable!(),
        Term::Simple(simple_term) => simple_term,
    };

    let mut parameters = Vec::new();
    for pair in it {
        let param = construct_term(pair);
        parameters.push(param);
    }

    Term::Compound(CompoundTerm { name, parameters })
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
    fn test_construct_term_atom() {
        let pair = parse_and_unwrap(Rule::term, "abc");
        let term = construct_term(pair);
        assert_eq!(term, Term::Simple(SimpleTerm::Atom(String::from("abc"))));
    }

    #[test]
    fn test_construct_term_variable() {
        let pair = parse_and_unwrap(Rule::term, "Abc");
        let term = construct_term(pair);
        assert_eq!(term, Term::Simple(SimpleTerm::Variable(String::from("Abc"))));
    }

    #[test]
    fn test_construct_term_underscore() {
        let pair = parse_and_unwrap(Rule::term, "_abc");
        let term = construct_term(pair);
        assert_eq!(term, Term::Simple(SimpleTerm::Variable(String::from("_abc"))));
    }

    #[test]
    #[should_panic]
    fn test_construct_term_panic() {
        let pair = parse_and_unwrap(Rule::term, "#abc");
        let term = construct_term(pair);
    }

    #[test]
    fn test_construct_term_compound() {
        let pair = parse_and_unwrap(Rule::term, "a (a (b e f)) c");
        let term = construct_term(pair);

        let mut parameters = Vec::new();
        parameters.push(Term::Simple(SimpleTerm::Atom(String::from("e"))));
        parameters.push(Term::Simple(SimpleTerm::Atom(String::from("f"))));
        let arg1 =
            Term::Compound(CompoundTerm {
                name: SimpleTerm::Atom(String::from("b")),
                parameters
            });

        let mut parameters = Vec::new();
        parameters.push(arg1);
        let arg2 =
            Term::Compound(CompoundTerm {
                name: SimpleTerm::Atom(String::from("a")),
                parameters
            });

        let mut parameters = Vec::new();
        parameters.push(arg2);
        parameters.push(Term::Simple(SimpleTerm::Atom(String::from("c"))));
        let expected_term =
            Term::Compound(CompoundTerm {
                name: SimpleTerm::Atom(String::from("a")),
                parameters
            });

        assert_eq!(expected_term, term);
    }

    #[test]
    fn test_construct_clause_without_body() {
        let pair = parse_and_unwrap(Rule::clause, "a (a (b e f)) c");
        let clause = construct_clause(pair);

        let mut parameters = Vec::new();
        parameters.push(Term::Simple(SimpleTerm::Atom(String::from("e"))));
        parameters.push(Term::Simple(SimpleTerm::Atom(String::from("f"))));
        let arg1 =
            Term::Compound(CompoundTerm {
                name: SimpleTerm::Atom(String::from("b")),
                parameters
            });

        let mut parameters = Vec::new();
        parameters.push(arg1);
        let arg2 =
            Term::Compound(CompoundTerm {
                name: SimpleTerm::Atom(String::from("a")),
                parameters
            });

        let mut parameters = Vec::new();
        parameters.push(arg2);
        parameters.push(Term::Simple(SimpleTerm::Atom(String::from("c"))));
        let expected_head =
            Term::Compound(CompoundTerm {
                name: SimpleTerm::Atom(String::from("a")),
                parameters
            });

        let mut expected_body = Vec::new();

        let expected_clause = Clause {
            head: expected_head,
            body: expected_body
        };

        assert_eq!(expected_clause, clause);
    }

    #[test]
    fn test_construct_clause_with_body() {
        let pair = parse_and_unwrap(Rule::clause, "a (a (b e f)) c if a and b");
        let clause = construct_clause(pair);

        let mut parameters = Vec::new();
        parameters.push(Term::Simple(SimpleTerm::Atom(String::from("e"))));
        parameters.push(Term::Simple(SimpleTerm::Atom(String::from("f"))));
        let arg1 =
            Term::Compound(CompoundTerm {
                name: SimpleTerm::Atom(String::from("b")),
                parameters
            });

        let mut parameters = Vec::new();
        parameters.push(arg1);
        let arg2 =
            Term::Compound(CompoundTerm {
                name: SimpleTerm::Atom(String::from("a")),
                parameters
            });

        let mut parameters = Vec::new();
        parameters.push(arg2);
        parameters.push(Term::Simple(SimpleTerm::Atom(String::from("c"))));
        let expected_head =
            Term::Compound(CompoundTerm {
                name: SimpleTerm::Atom(String::from("a")),
                parameters
            });

        let mut expected_body = Vec::new();
        expected_body.push(Term::Simple(SimpleTerm::Atom(String::from("a"))));
        expected_body.push(Term::Simple(SimpleTerm::Atom(String::from("b"))));

        let expected_clause = Clause {
            head: expected_head,
            body: expected_body
        };

        assert_eq!(expected_clause, clause);
    }
}