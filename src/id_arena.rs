use std::num::NonZeroU32;
use std::marker::PhantomData;
use std::cmp::Ordering;
use std::hash::{Hash, Hasher};
use std::ops::{Index, IndexMut};
use std::fmt::{Debug, Formatter};

pub struct Id<T> {
    idx: NonZeroU32,
    _ty: PhantomData<T>
}

// #[derive] bug means we have to impl these manually because of PhantomData
impl<T> Clone for Id<T> {
    fn clone(&self) -> Self {
        Id {
            idx: self.idx.clone(),
            _ty: PhantomData
        }
    }
}

impl<T> Copy for Id<T> {}

impl<T> PartialEq for Id<T> {
    fn eq(&self, other: &Self) -> bool {
        self.idx == other.idx
    }
}

impl<T> Eq for Id<T> {}

impl<T> PartialOrd for Id<T> {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        self.idx.partial_cmp(&other.idx)
    }
}

impl<T> Ord for Id<T> {
    fn cmp(&self, other: &Self) -> Ordering {
        self.idx.cmp(&other.idx)
    }
}

impl<T> Hash for Id<T> {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.idx.hash(state)
    }
}

impl<T> Debug for Id<T> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        self.idx.fmt(f)
    }
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

impl<T> Index<Id<T>> for IdArena<T> {
    type Output = T;

    fn index(&self, index: Id<T>) -> &Self::Output {
        self.get(index)
    }
}

impl<T> IndexMut<Id<T>> for IdArena<T> {
    fn index_mut(&mut self, index: Id<T>) -> &mut Self::Output {
        self.get_mut(index)
    }
}