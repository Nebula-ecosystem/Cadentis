use std::mem::MaybeUninit;

/// A simple slab allocator.
///
/// A `Slab` stores values of type `T` in a contiguous array and
/// returns stable indices that can be reused after removal.
///
/// Internally, it keeps track of:
/// - initialized slots,
/// - free indices,
/// - and uninitialized memory using [`MaybeUninit`].
///
/// This structure is useful for building arenas, object pools,
/// or systems where allocation and deallocation must be fast
/// and indices must remain small and reusable.
///
/// # Safety
///
/// This type uses `unsafe` internally but provides a safe API
/// as long as indices returned by [`insert`](Self::insert)
/// are not reused after [`remove`](Self::remove).
pub(crate) struct Slab<T> {
    /// Storage for items (may contain uninitialized slots).
    items: Vec<MaybeUninit<T>>,
    /// Stack of free indices that can be reused.
    free: Vec<usize>,
    /// Marks whether a slot is currently initialized.
    used: Vec<bool>,
}

impl<T> Slab<T> {
    /// Creates a new `Slab` with a fixed initial capacity.
    ///
    /// All slots are initially free and uninitialized.
    ///
    /// # Arguments
    ///
    /// * `size` - Initial number of slots to allocate.
    ///
    /// # Examples
    ///
    /// ```rust,ignore
    /// let slab = Slab::<i32>::new(16);
    /// ```
    pub(crate) fn new(size: usize) -> Self {
        let items = (0..size).map(|_| MaybeUninit::<T>::uninit()).collect();
        let free = (0..size).collect();
        let used = (0..size).map(|_| false).collect();

        Self { items, free, used }
    }

    /// Inserts a value into the slab and returns its index.
    ///
    /// If a free slot is available, it is reused.
    /// Otherwise, the slab grows exponentially.
    ///
    /// # Returns
    ///
    /// The index at which the value was inserted.
    ///
    /// # Examples
    ///
    /// ```rust,ignore
    /// let mut slab = Slab::new(1);
    /// let idx = slab.insert(42);
    /// ```
    pub(crate) fn insert(&mut self, item: T) -> usize {
        let index = if let Some(i) = self.free.pop() {
            i
        } else {
            let len = self.items.len();
            let new_len = if len == 0 { 1 } else { 2 * len };

            self.items
                .extend((len..new_len).map(|_| MaybeUninit::<T>::uninit()));
            self.free.extend((len + 1)..new_len);
            self.used.extend((len..new_len).map(|_| false));

            len
        };

        self.items[index] = MaybeUninit::new(item);
        self.used[index] = true;

        index
    }

    /// Removes and returns the value stored at `index`.
    ///
    /// The slot becomes free and may be reused by future insertions.
    ///
    /// # Panics
    ///
    /// Panics if:
    /// - `index` is out of bounds
    /// - the slot is not currently in use
    ///
    /// # Examples
    ///
    /// ```rust,ignore
    /// let mut slab = Slab::new(1);
    /// let idx = slab.insert(10);
    /// let value = slab.remove(idx);
    /// assert_eq!(value, 10);
    /// ```
    pub(crate) fn remove(&mut self, index: usize) -> T {
        assert!(index < self.items.len(), "Index out of range");
        assert!(self.used[index], "Item is not set");

        self.free.push(index);
        self.used[index] = false;

        let item = unsafe { self.items[index].assume_init_read() };
        self.items[index] = MaybeUninit::uninit();

        item
    }

    /// Returns a mutable reference to the value at `index`.
    ///
    /// # Safety
    ///
    /// The caller must ensure that:
    /// - `index` is valid
    /// - the slot is currently initialized
    ///
    /// Violating these conditions results in undefined behavior.
    ///
    /// # Panics
    ///
    /// Panics if `index` is out of bounds.
    pub(crate) fn get_mut(&mut self, index: usize) -> &mut T {
        unsafe { self.items.get_mut(index).unwrap().assume_init_mut() }
    }
}

impl<T> Drop for Slab<T> {
    /// Drops all initialized elements stored in the slab.
    ///
    /// Uninitialized slots are ignored.
    fn drop(&mut self) {
        for (slot, &used) in self.items.iter_mut().zip(self.used.iter()) {
            if used {
                unsafe {
                    slot.assume_init_drop();
                }
            }
        }
    }
}
