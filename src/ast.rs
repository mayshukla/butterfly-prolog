pub type Program = Vec<Clause>; 

#[derive(Debug, PartialEq)]
pub struct Clause {
    pub head: Term,
    pub body: Option<Vec<Term>>,
}

#[derive(Debug, PartialEq)]
pub enum Term {
    Compound(CompoundTerm),
    Simple(SimpleTerm),
}

#[derive(Debug, PartialEq)]
pub struct CompoundTerm {
    pub name: SimpleTerm,
    pub parameters: Vec<Term>,
}

#[derive(Debug, PartialEq)]
pub enum SimpleTerm {
    Atom(String),
    Variable(String),
}