use std::num::NonZeroU32;
use std::marker::PhantomData;

#[derive(Copy, Clone)]
pub struct Id<T> {
    idx: NonZeroU32,
    _ty: PhantomData<T>
}

impl<T> Id<T> {
    fn idx(&self) -> usize {
        self.idx.get() as usize - 1
    }
}

pub struct IdArena<T> {
    items: Vec<T>,
}

impl<T> IdArena<T> {
    pub fn new() -> Self {
        Self {
            items: Vec::new(),
        }
    }

    pub fn insert(&mut self, item: T) -> Id<T> {
        self.items.push(item);
        let idx = NonZeroU32::new(self.items.len() as u32).unwrap();
        Id {
            idx,
            _ty: PhantomData
        }
    }

    pub fn get(&self, id: Id<T>) -> &T {
        let idx = id.idx();
        debug_assert!(idx < self.items.len());
        unsafe { self.items.get_unchecked(idx) }
    }

    pub fn get_mut(&mut self, id: Id<T>) -> &mut T {
        let idx = id.idx();
        debug_assert!(idx < self.items.len());
        unsafe { self.items.get_unchecked_mut(idx) }
    }

    pub fn try_index(&self, index: usize) -> Option<&T> {
        if index == 0 {
            None
        } else {
            self.items.get(index)
        }
    }

}