/**
 * "Heap" memory area to use for storing the compiled representation of a
 * program.
 *
 * Based on heap representation used in https://github.com/ptarau/iProlog
 */
#[derive(Debug, PartialEq)]
pub struct Heap {
    buffer: Vec<HeapEntry>,
}

#[derive(Copy, Clone, Debug, PartialEq)]
pub struct HeapEntry {
    pub tag: HeapTag,
    pub data: HeapIndex,
}

#[derive(Copy, Clone, Debug, PartialEq)]
pub enum HeapTag {
    // First occurence of variable in a clause
    Variable,

    // Second or further occurence of variable in a clause
    Unify,

    // Reference to another array slice representing a subterm
    Reference,

    // Index of constant in symbol table
    Constant,

    // Number literal
    Number,

    // Declares size of array slice (1 + number of arguments)
    Arity,

    // Heap entry has not yet been initialized
    Uninitialized,
}

pub type HeapIndex = usize;

impl Heap {
    pub fn new() -> Self {
        Heap { buffer: Vec::new() }
    }

    /**
     * Allocates an array of the given size and returns the index to the start
     * of the array.
     */
    pub fn alloc(&mut self, size: HeapIndex) -> HeapIndex{
        let index = self.buffer.len();
        self.buffer.resize(index + size, HeapEntry::empty());
        index
    }

    pub fn write(&mut self, index: HeapIndex, entry: HeapEntry) {
        self.buffer[index] = entry;
    }

    pub fn read(&self, index: HeapIndex) -> HeapEntry {
        self.buffer[index]
    }

    pub fn len(&self) -> HeapIndex {
        self.buffer.len()
    }
}

impl HeapEntry {
    fn empty() -> Self {
        HeapEntry { tag: HeapTag::Uninitialized, data: 0 }
    }

    pub fn new(tag: HeapTag, data: HeapIndex) -> Self {
        HeapEntry { tag, data }
    }

    pub fn is_var_or_unify(&self) -> bool {
        self.tag == HeapTag::Variable || self.tag == HeapTag::Unify
    }
}

#[cfg(test)]
mod tests {
    use crate::heap::*;

    #[test]
    fn test_alloc() {
        let mut heap = Heap::new();
        let index = heap.alloc(32);

        assert_eq!(0, index);

        let mut expected_buffer = Vec::new();
        expected_buffer.resize(
            32,
            HeapEntry::empty()
        );

        assert_eq!(
            expected_buffer,
            heap.buffer
        );

        let index = heap.alloc(1);
        assert_eq!(32, index);
    }

    #[test]
    fn test_write() {
        let mut heap = Heap::new();
        let index = heap.alloc(32);
        let entry = HeapEntry::new(HeapTag::Variable, 5);
        heap.write(index, entry);

        let mut expected_buffer = Vec::new();
        expected_buffer.resize(
            32,
            HeapEntry::empty()
        );
        expected_buffer[0] = entry;

        assert_eq!(
            expected_buffer,
            heap.buffer
        );

        assert_eq!(
            entry,
            heap.read(index)
        );
    }
}