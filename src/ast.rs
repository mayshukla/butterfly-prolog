pub type Program = Vec<Clause>; 

#[derive(Debug, PartialEq)]
pub struct Clause {
    head: Term,
    body: Option<Vec<Term>>,
}

#[derive(Debug, PartialEq)]
pub enum Term {
    CompoundTerm,
    SimpleTerm,
}

#[derive(Debug, PartialEq)]
pub struct CompoundTerm {
    head: SimpleTerm,
    parameters: Vec<Term>,
}

#[derive(Debug, PartialEq)]
pub enum SimpleTerm {
    Atom(String),
    Variable(String),
}