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

    queries: Vec<QueryDescriptor>,
    spines: Vec<Spine>,
    trail: Vec<HeapIndex>,
}

/**
 * A descriptor of a clause on the heap.
 * Based on the "Clause" class in https://github.com/ptarau/iProlog
 */
#[derive(Clone, Debug, PartialEq)]
struct ClauseDescriptor {
    // Index to start of clause
    base: HeapIndex,
    // Length of clause array slice in heap
    length: HeapIndex,
    // Length of head of clause
    neck: HeapIndex,

    // Toplevel skeleton of clause (Reference entries that point to each
    // top-level term)
    terms: Vec<HeapEntry>,
    // Dereferenced subterms of head
    head_subterms: Vec<HeapEntry>,
}

/**
 * A descriptor of a query on the heap.
 */
#[derive(Clone, Debug, PartialEq)]
struct QueryDescriptor {
    // Index to start of clause
    base: HeapIndex,
    // Length of query array slice in heap
    length: HeapIndex,

    // Reference entries that point to each top-level term (sub-goal)
    terms: Vec<HeapEntry>,
}

/**
 * Stores the current state of the search.
 * Based on the "Spine" class in https://github.com/ptarau/iProlog
 */
#[derive(Debug, PartialEq)]
struct Spine {
    // Top of heap when this spine created
    base: HeapIndex,
    // Top of trail when this spine created
    trail_top: HeapIndex,

    goals: Vec<HeapEntry>,
    dereferenced_elements: Vec<HeapEntry>,
    // TODO clarify meaning of this field
    // Clauses (potentially?) unifiable with first goal in goals
    // Elements are indeces in Compiler.clauses
    unifiable_clauses: Vec<usize>,

    // TODO clarify meaning of this field
    // Number clauses that have already been unified
    num_unified_clauses: usize,
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
            queries: Vec::new(),
            spines: Vec::new(),
            trail: Vec::new(),
        }
    }

    pub fn compile(&mut self, program: Program) {
        for clause in program.clauses {
            self.compile_clause(clause);
        }

        for query in program.queries {
            self.compile_query(query);
        }
        self.create_initial_spine(self.queries.clone());
    }

    fn compile_clause(&mut self, clause: Clause) {
        self.current_clause_variables.clear();

        let base = self.heap.len();
        if let Term::Simple(_) = clause.head {
            self.create_arity_entry_for_simple_term();
        }

        self.compile_term(clause.head);
        let neck = self.heap.len();

        let mut terms = Vec::new();
        terms.push(base);

        for term in clause.body {
            let term_index = self.heap.len();
            if let Term::Simple(_) = term {
                self.create_arity_entry_for_simple_term();
            }
            self.compile_term(term);
            terms.push(term_index);
        }

        let length = self.heap.len() - base;

        // Convert indeces into heap entries
        let terms: Vec<HeapEntry> = terms
            .into_iter()
            .map(|index| HeapEntry { tag: HeapTag::Reference, data: index } ).collect();

        let head_subterms = self.get_subterms(terms[0]);

        self.clauses.push(ClauseDescriptor {
            base,
            length,
            neck,
            terms,
            head_subterms,
        });
    }

    /**
     * Top-level simple terms need an Arity entry. This is because every entry
     * in the terms array of a ClauseDescriptor is expected to point to an Arity
     * entry.
     */
    fn create_arity_entry_for_simple_term(&mut self) {
        let index = self.heap.alloc(1);
        self.heap.write(index, HeapEntry::new(
            HeapTag::Arity,
            // simple terms always have arity 1
            1
        ));
    }

    fn compile_term(&mut self, term: Term) -> HeapIndex {
        match term {
            Term::Compound(term) => self.compile_compound_term(term),
            Term::Simple(term) => self.compile_simple_term(term),
        }
    }

    fn compile_simple_term(&mut self, term: SimpleTerm) -> HeapIndex {
        let index = self.heap.alloc(1);
        self.compile_simple_term_no_alloc(term, index);
        index
    }

    /**
     * Compiles a simple term and places on the heap at index.
     * Heap must have one space allocated at index.
     */
    fn compile_simple_term_no_alloc(&mut self, term: SimpleTerm, index: HeapIndex) {
        match term {
            SimpleTerm::Atom(atom) => {
                // Get index from symbol table, creating new symbol if one
                // doesn't already exist.
                let symbol_index = match self.symbol_table.get_index(&atom) {
                    Some(prev_index) => prev_index,
                    None => self.symbol_table.push(&atom)
                };
                let heap_entry = HeapEntry::new(HeapTag::Constant, symbol_index);
                self.heap.write(index, heap_entry);
            },
            SimpleTerm::Variable(variable) => {
                match self.current_clause_variables.get(&variable) {
                    Some(variable_index) => {
                        // Variable has been seen before
                        self.heap.write(index, HeapEntry::new(HeapTag::Unify, *variable_index));
                    },
                    None => {
                        // First time seeing variable
                        self.heap.write(index, HeapEntry::new(HeapTag::Variable, index));
                        self.current_clause_variables.insert(variable, index);
                    }
                }
            },
        }
    }

    /**
     * Compiles compound term and returns index of start of term in heap.
     */
    fn compile_compound_term(&mut self, term: CompoundTerm) -> HeapIndex {
        // Allocate heap space for 2 + parameters.len()
        // + 2 to make room for arity and name
        let arity = term.parameters.len() + 1;
        let start_index = self.heap.alloc(1 + arity);
        let mut index = start_index;

        let arity_cell = HeapEntry::new(HeapTag::Arity, arity);
        self.heap.write(index, arity_cell);

        index += 1;
        self.compile_simple_term_no_alloc(term.name, index);

        for param in term.parameters {
            index += 1;
            match param {
                Term::Simple(simple_term) => {
                    self.compile_simple_term_no_alloc(simple_term, index);
                },
                Term::Compound(compound_term) => {
                    // Compile the subterm somewhere else in the heap.
                    let subterm_index = self.compile_compound_term(compound_term);
                    // Place a reference to the compiled subterm in the current
                    // term's array slice.
                    let reference = HeapEntry::new(HeapTag::Reference, subterm_index);
                    self.heap.write(index, reference);
                }
            }
        }

        start_index
    }

    fn compile_query(&mut self, query: Query) {
        let base = self.heap.len();
        let mut terms = Vec::new();
        for term in query.sub_queries {
            let term_index = self.heap.len();
            if let Term::Simple(_) = term {
                self.create_arity_entry_for_simple_term();
            }
            self.compile_term(term);
            terms.push(HeapEntry::new(HeapTag::Reference, term_index));
        }
        let length = self.heap.len() - base;
        self.queries.push( QueryDescriptor {
            base,
            length,
            terms,
        });
    }

    /**
     * Given the index of a term, returns dereferenced heap entries of subterms.
     */
    fn get_subterms(&self, reference: HeapEntry) -> Vec<HeapEntry> {
        let start_index = reference.data + 1;
        let arity = self.deref_once(reference).data;

        let mut subterms = Vec::new();
        for i in 0..arity {
            let mut entry = self.deref(self.heap.read(start_index + i));
            if let HeapTag::Reference = entry.tag {
                entry = self.deref_once(entry);
            }
            // TODO should we store Variable entries with data=0?
            // (that's what iProlog does)
            subterms.push(entry);
        }

        subterms
    }

    /**
     * Returns the HeapEntry that the pointer HeapEntry points to.
     */
    fn deref_once(&self, pointer: HeapEntry) -> HeapEntry {
        self.heap.read(pointer.data)
    }

    /**
     * Follows chain of references until reaching first occurance of variable or
     * a non-variable entry.
     */
    fn deref(&self, pointer: HeapEntry) -> HeapEntry {
        let mut result = pointer;
        while result.is_var_or_unify() {
            let dereferenced = self.deref_once(result);
            if dereferenced == result {
                // result is first occurence of variable
                break;
            }
            result = dereferenced;
        }
        result
    }

    fn create_initial_spine(&mut self, queries: Vec<QueryDescriptor>) {
        let base = self.heap.len();
        // Push to self.spines in reverse order of queries so that they are
        // popped in correct order
        let unifiable_clauses: Vec<usize> = self.clauses.clone().into_iter().enumerate().map(|(i, _)| i).collect();
        for query in queries.into_iter().rev() {
            let spine = Spine::new(
                base,
                self.trail.len(),
                query.terms,
                unifiable_clauses.clone(),
                0
            );
            self.spines.push(spine);
        }
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

impl Spine {
    fn new(
        base: usize,
        trail_top: usize,
        goals: Vec<HeapEntry>,
        unifiable_clauses: Vec<usize>,
        num_unified_clauses: usize
    ) -> Self {
        Spine {
            base,
            trail_top,
            goals,
            dereferenced_elements: Vec::new(),
            unifiable_clauses,
            num_unified_clauses,
        }
    }
}


#[cfg(test)]
mod tests {
    use crate::compiler::*;

    /**
     * Returns a Term representing "a (a (b e f)) c"
     */
    fn make_compound_term() -> Term {
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

        Term::Compound(CompoundTerm {
            name: SimpleTerm::Atom(String::from("a")),
            parameters
        })
    }

    #[test]
    fn test_compile_atom() {
        let mut program = Program::new();

        let head = Term::Simple(SimpleTerm::Atom(String::from("a")));
        let mut body = Vec::new();
        body.push(Term::Simple(SimpleTerm::Atom(String::from("b"))));
        body.push(Term::Simple(SimpleTerm::Atom(String::from("c"))));
        program.push_clause(
            Clause {
                head,
                body
            }
        );

        let head = Term::Simple(SimpleTerm::Atom(String::from("b")));
        let body = Vec::new();
        program.push_clause(
            Clause {
                head,
                body
            }
        );

        let head = Term::Simple(SimpleTerm::Atom(String::from("c")));
        let body = Vec::new();
        program.push_clause(
            Clause {
                head,
                body
            }
        );

        let mut compiler = Compiler::new();
        compiler.compile(program);

        let expected_heap = vec![
            HeapEntry::new(HeapTag::Arity, 1),
            HeapEntry::new(HeapTag::Constant, 0),
            HeapEntry::new(HeapTag::Arity, 1),
            HeapEntry::new(HeapTag::Constant, 1),
            HeapEntry::new(HeapTag::Arity, 1),
            HeapEntry::new(HeapTag::Constant, 2),
            HeapEntry::new(HeapTag::Arity, 1),
            HeapEntry::new(HeapTag::Constant, 1),
            HeapEntry::new(HeapTag::Arity, 1),
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
        program.push_clause(
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
        program.push_clause(
            Clause {
                head,
                body
            }
        );

        let mut compiler = Compiler::new();
        compiler.compile(program);

        let expected_heap = vec![
            HeapEntry::new(HeapTag::Arity, 1),
            HeapEntry::new(HeapTag::Constant, 0),
            HeapEntry::new(HeapTag::Arity, 1),
            HeapEntry::new(HeapTag::Variable, 3),
            HeapEntry::new(HeapTag::Arity, 1),
            HeapEntry::new(HeapTag::Variable, 5),
            HeapEntry::new(HeapTag::Arity, 1),
            HeapEntry::new(HeapTag::Unify, 3),
            HeapEntry::new(HeapTag::Arity, 1),
            HeapEntry::new(HeapTag::Constant, 0),
            HeapEntry::new(HeapTag::Arity, 1),
            HeapEntry::new(HeapTag::Variable, 11),
        ];

        for i in 0..expected_heap.len() {
            assert_eq!(expected_heap[i], compiler.heap.read(i));
        }
    }

    #[test]
    fn test_compile_compound_term() {
        let mut program = Program::new();
        let head = make_compound_term();
        program.push_clause(Clause {
            head,
            body: Vec::new()
        });

        //println!("ast: {:?}", program);
        let mut compiler = Compiler::new();
        compiler.compile(program);

        //println!("heap: {:?}", compiler.heap);
        //println!("symbol_table: {:?}", compiler.symbol_table);
        //println!("clauses: {:?}", compiler.clauses);

        let expected_heap = vec![
            // 0: a a _4 c
            HeapEntry::new(HeapTag::Arity, 3),
            HeapEntry::new(HeapTag::Constant, 0),
            HeapEntry::new(HeapTag::Reference, 4),
            HeapEntry::new(HeapTag::Constant, 4),

            // 4: a _7
            HeapEntry::new(HeapTag::Arity, 2),
            HeapEntry::new(HeapTag::Constant, 0),
            HeapEntry::new(HeapTag::Reference, 7),

            // 7: b e f
            HeapEntry::new(HeapTag::Arity, 3),
            HeapEntry::new(HeapTag::Constant, 1),
            HeapEntry::new(HeapTag::Constant, 2),
            HeapEntry::new(HeapTag::Constant, 3),
        ];

        for i in 0..expected_heap.len() {
            assert_eq!(expected_heap[i], compiler.heap.read(i));
        }
    }

    #[test]
    fn test_compile_clause_and_query() {
        let mut program = Program::new();
        let head = make_compound_term();
        let mut body = Vec::new();
        body.push(Term::Simple(SimpleTerm::Atom(String::from("x"))));
        body.push(Term::Simple(SimpleTerm::Variable(String::from("Y"))));
        program.push_clause(Clause { head, body });

        let mut sub_queries = Vec::new();
        sub_queries.push(Term::Simple(SimpleTerm::Atom(String::from("a"))));
        sub_queries.push(Term::Simple(SimpleTerm::Atom(String::from("b"))));
        program.push_query(Query {
            sub_queries
        });

        println!("ast: {:?}", program);
        let mut compiler = Compiler::new();
        compiler.compile(program);

        println!("heap: {:?}", compiler.heap);
        println!("symbol_table: {:?}", compiler.symbol_table);
        println!("clauses: {:?}", compiler.clauses);

        let expected_heap = vec![
            // 0: a a _4 c
            HeapEntry::new(HeapTag::Arity, 3),
            HeapEntry::new(HeapTag::Constant, 0),
            HeapEntry::new(HeapTag::Reference, 4),
            HeapEntry::new(HeapTag::Constant, 4),

            // 4: a _7
            HeapEntry::new(HeapTag::Arity, 2),
            HeapEntry::new(HeapTag::Constant, 0),
            HeapEntry::new(HeapTag::Reference, 7),

            // 7: b e f
            HeapEntry::new(HeapTag::Arity, 3),
            HeapEntry::new(HeapTag::Constant, 1),
            HeapEntry::new(HeapTag::Constant, 2),
            HeapEntry::new(HeapTag::Constant, 3),

            // 11: x
            HeapEntry::new(HeapTag::Arity, 1),
            HeapEntry::new(HeapTag::Constant, 5),

            // 13: y
            HeapEntry::new(HeapTag::Arity, 1),
            HeapEntry::new(HeapTag::Variable, 14),

            // Start of query
            // 15: a
            HeapEntry::new(HeapTag::Arity, 1),
            HeapEntry::new(HeapTag::Constant, 0),

            // 17: b
            HeapEntry::new(HeapTag::Arity, 1),
            HeapEntry::new(HeapTag::Constant, 1),
        ];

        for i in 0..expected_heap.len() {
            assert_eq!(expected_heap[i], compiler.heap.read(i));
        }

        let expected_clause = ClauseDescriptor {
            base: 0,
            length: 15,
            neck: 11,
            terms: vec![
                HeapEntry { tag: HeapTag::Reference, data: 0 },
                HeapEntry { tag: HeapTag::Reference, data: 11 },
                HeapEntry { tag: HeapTag::Reference, data: 13 },
            ],
            head_subterms: vec![expected_heap[1], expected_heap[4], expected_heap[3]],
        };

        assert_eq!(expected_clause, compiler.clauses[0]);

        let expected_query = QueryDescriptor {
            base: 15,
            length: 4,
            terms: vec![
                HeapEntry { tag: HeapTag::Reference, data: 15 },
                HeapEntry { tag: HeapTag::Reference, data: 17 },
            ],
        };

        assert_eq!(expected_query, compiler.queries[0]);

        let expected_spine = Spine::new(
            19,
            0,
            expected_query.terms.clone(),
            vec![0],
            0
        );

        assert_eq!(expected_spine, compiler.spines[0]);
    }
}