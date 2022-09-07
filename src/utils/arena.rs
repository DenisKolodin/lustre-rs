pub type ArenaIndex = usize;
pub use Option as ArenaNode;

#[derive(Debug)]
pub struct Arena<T> {
    store: Vec<ArenaNode<T>>,
}

impl<T> Arena<T> {
    /// Creates a new Arena.
    pub fn new() -> Self {
        Self { store: Vec::new() }
    }

    /// Creates a new Arena with space for `capacity` amount of elements
    pub fn with_capacity(capacity: usize) -> Self {
        Self {
            store: Vec::with_capacity(capacity),
        }
    }

    /// Adds the item to the arena.
    pub fn add(&mut self, item: T) -> ArenaIndex {
        let index = self.store.len();
        self.store.push(Some(item));
        index
    }

    /// Returns a reference to the [ArenaNode] at the provided index.
    ///
    /// # Panics
    ///
    /// Panics if the position is out of bounds.
    pub fn get(&self, index: ArenaIndex) -> &ArenaNode<T> {
        &self.store[index]
    }
}

impl<T> std::ops::Index<ArenaIndex> for Arena<T> {
    type Output = ArenaNode<T>;

    fn index(&self, index: ArenaIndex) -> &Self::Output {
        &self.store[index]
    }
}
