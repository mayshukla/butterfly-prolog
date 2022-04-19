use std::collections::HashMap;

use crate::ast::*;
use crate::heap::*;

struct Compiler {
    heap: Heap,
    clauses: Vec<ClauseDescriptor>,
    symbol_table: SymbolTable,

    // Keep track of indeces of variables during compilation
    // This is cleared before compiling each clause.
    current_clause_variables: HashMap<String, HeapIndex>,
}

/**
 * A descriptor of a clause on the heap.
 * Based on the "Clause" class in https://github.com/ptarau/iProlog
 */
struct ClauseDescriptor {
    // Index to start of clause
    base: HeapIndex,
    // Length of clause array slice in heap
    length: HeapIndex,
    // Length of head of clause
    neck: HeapIndex,

    // Toplevel skeleton of clause (indeces of top-level elements)
    gs: Vec<HeapIndex>,
    // Used to store dereferenced data
    xs: Vec<HeapIndex>,
}

#[derive(Debug)]
struct SymbolTable {
    // TODO allow storing other types of data like floats
    symbols: Vec<String>,
    symbols_to_indeces: HashMap<String, usize>,
}

impl Compiler {
    pub fn new() -> Self {
        Compiler {
            heap: Heap::new(),
            clauses: Vec::new(),
            symbol_table: SymbolTable::new(),
            current_clause_variables: HashMap::new(),
        }
    }

    pub fn compile(&mut self, program: Program) {
        for clause in program {
            self.compile_clause(clause);
        }
    }

    fn compile_clause(&mut self, clause: Clause) {
        self.current_clause_variables.clear();
        self.compile_term(clause.head);
        for term in clause.body {
            self.compile_term(term);
        }
        // TODO create ClauseDescriptor and push
    }

    fn compile_term(&mut self, term: Term) {
        match term {
            Term::Compound(term) => self.compile_compound_term(term),
            Term::Simple(term) => self.compile_simple_term(term),
        }
    }

    fn compile_simple_term(&mut self, term: SimpleTerm) {
        match term {
            SimpleTerm::Atom(atom) => {
                // Get index from symbol table, creating new symbol if one
                // doesn't already exist.
                let index = match self.symbol_table.get_index(&atom) {
                    Some(index) => index,
                    None => self.symbol_table.push(&atom)
                };
                let heap_entry = HeapEntry::new(HeapTag::Constant, index);
                let index = self.heap.alloc(1);
                self.heap.write(index, heap_entry);
            },
            SimpleTerm::Variable(variable) => {
                match self.current_clause_variables.get(&variable) {
                    Some(variable_index) => {
                        // Variable has been seen before
                        let heap_index = self.heap.alloc(1);
                        self.heap.write(heap_index, HeapEntry::new(HeapTag::Unify, *variable_index));
                    },
                    None => {
                        // First time seeing variable
                        let index = self.heap.alloc(1);
                        self.heap.write(index, HeapEntry::new(HeapTag::Variable, index));
                        self.current_clause_variables.insert(variable, index);
                    }
                }
            },
        }
    }

    fn compile_compound_term(&mut self, term: CompoundTerm) {
        todo!()
    }
}

impl SymbolTable {
    fn new() -> Self {
        SymbolTable { symbols: Vec::new(), symbols_to_indeces: HashMap::new() }
    }

    fn push(&mut self, symbol: &str) -> usize {
        self.symbols.push(symbol.to_string());
        let index = self.symbols.len() - 1;
        self.symbols_to_indeces.insert(symbol.to_string(), index);
        index
    }

    fn get(&self, index: usize) -> &str {
        &self.symbols[index]
    }

    fn get_index(&self, symbol: &str) -> Option<usize> {
        let found = self.symbols_to_indeces.get(symbol);
        match found {
            Some(index) => Some(*index),
            None => None
        }
    }
}


#[cfg(test)]
mod tests {
    use crate::compiler::*;

    #[test]
    fn test_compile_atom() {
        let mut program = Program::new();

        let head = Term::Simple(SimpleTerm::Atom(String::from("a")));
        let mut body = Vec::new();
        body.push(Term::Simple(SimpleTerm::Atom(String::from("b"))));
        body.push(Term::Simple(SimpleTerm::Atom(String::from("c"))));
        program.push(
            Clause {
                head,
                body
            }
        );

        let head = Term::Simple(SimpleTerm::Atom(String::from("b")));
        let body = Vec::new();
        program.push(
            Clause {
                head,
                body
            }
        );

        let head = Term::Simple(SimpleTerm::Atom(String::from("c")));
        let body = Vec::new();
        program.push(
            Clause {
                head,
                body
            }
        );

        let mut compiler = Compiler::new();
        compiler.compile(program);

        let expected_heap = vec![
            HeapEntry::new(HeapTag::Constant, 0),
            HeapEntry::new(HeapTag::Constant, 1),
            HeapEntry::new(HeapTag::Constant, 2),
            HeapEntry::new(HeapTag::Constant, 1),
            HeapEntry::new(HeapTag::Constant, 2),
        ];

        for i in 0..expected_heap.len() {
            assert_eq!(expected_heap[i], compiler.heap.read(i));
        }

        assert_eq!(compiler.symbol_table.get(0), "a");
        assert_eq!(compiler.symbol_table.get(1), "b");
        assert_eq!(compiler.symbol_table.get(2), "c");
    }

    #[test]
    fn test_compile_variable() {
        let mut program = Program::new();

        let head = Term::Simple(SimpleTerm::Atom(String::from("a")));
        let mut body = Vec::new();
        body.push(Term::Simple(SimpleTerm::Variable(String::from("B"))));
        body.push(Term::Simple(SimpleTerm::Variable(String::from("C"))));
        body.push(Term::Simple(SimpleTerm::Variable(String::from("B"))));
        program.push(
            Clause {
                head,
                body
            }
        );

        // The variable "B" in this clause should be considered different from B
        // in previous clause.
        let head = Term::Simple(SimpleTerm::Atom(String::from("a")));
        let mut body = Vec::new();
        body.push(Term::Simple(SimpleTerm::Variable(String::from("B"))));
        program.push(
            Clause {
                head,
                body
            }
        );

        let mut compiler = Compiler::new();
        compiler.compile(program);

        println!("heap: {:?}", compiler.heap);

        let expected_heap = vec![
            HeapEntry::new(HeapTag::Constant, 0),
            HeapEntry::new(HeapTag::Variable, 1),
            HeapEntry::new(HeapTag::Variable, 2),
            HeapEntry::new(HeapTag::Unify, 1),
            HeapEntry::new(HeapTag::Constant, 0),
            HeapEntry::new(HeapTag::Variable, 5),
        ];

        for i in 0..expected_heap.len() {
            assert_eq!(expected_heap[i], compiler.heap.read(i));
        }
    }
}