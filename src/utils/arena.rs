//! A simple arena alllocator
//!
//! Such a structure is extremely helpful with graph data structure
//! implementations in Rust; mananging lifetimes with parent-child node
//! relationships, possible cycles, etc..
//!
//! [generational-arena](https://github.com/fitzgen/generational-arena) is
//! a strong candidate for "there's something already out there" but
//! - we don't need to remove items once inserted, a major selling point
//! - I wanted to write one myself

/// Index type for the [Arena]
///
/// Only needed in case the indexing scheme ever changes
pub type ArenaIndex = usize;

/// A simple arena alllocator
///
/// Essentially a newtype wrapper around [Vec], most of the functionality
/// is calling the [Vec] methods of the same name.
///
/// The Arena-specific work lies in the [add] method.
#[derive(Debug)]
pub struct Arena<T> {
    /// The backing store for this allocator
    store: Vec<T>,
}

#[allow(unused)]
impl<T> Arena<T> {
    /// Creates a new, empty Arena.
    #[inline]
    pub fn new() -> Self {
        Self { store: Vec::new() }
    }

    /// Creates a new Arena with space for `capacity` amount of elements
    #[inline]
    pub fn with_capacity(capacity: usize) -> Self {
        Self {
            store: Vec::with_capacity(capacity),
        }
    }

    /// Adds the item to the arena
    ///
    /// Returns the index into the arena of the inserted item.
    #[inline]
    pub fn add(&mut self, item: T) -> ArenaIndex {
        let index = self.store.len();
        self.store.push(item);
        index
    }

    /// Returns a reference to the item at the provided index, or `None` if out of bounds.
    #[inline]
    pub fn get(&self, index: ArenaIndex) -> Option<&T> {
        self.store.get(index)
    }

    /// Shrinks the arena's backing store as close as possible to the contained elements
    #[inline]
    pub fn shrink_to_fit(&mut self) {
        self.store.shrink_to_fit()
    }

    #[inline]
    /// Returns the number of elements in the arena
    pub fn len(&self) -> usize {
        self.store.len()
    }
}

impl<T> std::ops::Index<ArenaIndex> for Arena<T> {
    type Output = T;

    fn index(&self, index: ArenaIndex) -> &Self::Output {
        self.store.index(index)
    }
}

impl<T> std::ops::IndexMut<ArenaIndex> for Arena<T> {
    fn index_mut(&mut self, index: ArenaIndex) -> &mut Self::Output {
        self.store.index_mut(index)
    }
}
