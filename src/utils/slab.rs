use std::mem::MaybeUninit;

pub(crate) struct Slab<T> {
    items: Vec<MaybeUninit<T>>,
    free: Vec<usize>,
    used: Vec<bool>,
}

impl<T> Slab<T> {
    pub(crate) fn new(size: usize) -> Self {
        let items = (0..size).map(|_| MaybeUninit::<T>::uninit()).collect();
        let free = (0..size).collect();
        let used = (0..size).map(|_| false).collect();

        Self { items, free, used }
    }

    pub(crate) fn remove(&mut self, index: usize) -> T {
        assert!(index < self.items.len(), "Index out of range");
        assert!(self.used[index], "Item is not set");

        self.free.push(index);
        self.used[index] = false;

        let item = unsafe { self.items[index].assume_init_read() };
        self.items[index] = MaybeUninit::uninit();

        item
    }

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
}

impl<T> Drop for Slab<T> {
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
