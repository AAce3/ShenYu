use std::{
    mem,
    ops::{Index, IndexMut},
};
// Custom list implementation that can be stored on the stack
pub struct List<T> {
    pub items: [T; 255],
    pub length: u8,
}

impl<T> Default for List<T> {
    fn default() -> Self {
        Self::new()
    }
}

impl<T> List<T> {
    #[allow(clippy::uninit_assumed_init)]
    pub fn new() -> List<T> {
        Self {
            items: unsafe { mem::MaybeUninit::uninit().assume_init() },
            length: 0,
        }
    }

    #[inline]
    pub fn push(&mut self, item: T) {
        self.items[self.length as usize] = item;
        self.length += 1;
    }

    #[inline]
    pub fn swap(&mut self, idx1: usize, idx2: usize) {
        self.items.swap(idx1, idx2);
    }

    #[inline]
    #[allow(clippy::uninit_assumed_init)]
    pub fn clear(&mut self) {
        self.length = 0;
        self.items = unsafe { mem::MaybeUninit::uninit().assume_init() };
    }
}

impl<T> Index<usize> for List<T> {
    type Output = T;

    fn index(&self, index: usize) -> &Self::Output {
        &self.items[index]
    }
}

impl<T> IndexMut<usize> for List<T> {
    fn index_mut(&mut self, index: usize) -> &mut Self::Output {
        &mut self.items[index]
    }
}
