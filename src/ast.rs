#[derive(Debug, PartialEq)]
pub struct Program {
    pub clauses: Vec<Clause>,
    pub queries: Vec<Query>,
}

impl Program {
    pub fn new() -> Self {
        Program { clauses: Vec::new(), queries: Vec::new() }
    }

    pub fn push_clause(&mut self, clause: Clause) {
        self.clauses.push(clause);
    }

    pub fn push_query(&mut self, query: Query) {
        self.queries.push(query);
    }
}

#[derive(Debug, PartialEq)]
pub struct Clause {
    pub head: Term,
    pub body: Vec<Term>,
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

#[derive(Debug, PartialEq)]
pub struct Query {
    pub sub_queries: Vec<Term>,
}