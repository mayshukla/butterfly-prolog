pub type Program = Vec<Clause>; 

pub struct Clause {
    head: Term,
    body: Option<Vec<Term>>,
}

pub enum Term {
    CompoundTerm,
    SimpleTerm,
}

pub struct CompoundTerm {
    head: SimpleTerm,
    parameters: Vec<Term>,
}

pub enum SimpleTerm {
    Atom(String),
    Variable(String),
}