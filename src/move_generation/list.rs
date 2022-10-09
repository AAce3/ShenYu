use std::{
    mem,
    ops::{Index, IndexMut},
};
// Custom list implementation that can be stored on the stack
#[derive(PartialEq, Eq)]
pub struct List<T, const SIZE: usize> {
    pub items: [T; SIZE],
    pub length: usize,
}

impl<T, const SIZE: usize> Default for List<T, SIZE> {
    fn default() -> Self {
        Self::new()
    }
}

impl<T, const SIZE: usize> List<T, SIZE> {
    #[allow(clippy::uninit_assumed_init)]
    pub fn new() -> List<T, SIZE> {
        Self {
            items: unsafe { mem::MaybeUninit::uninit().assume_init() },
            length: 0,
        }
    }
    #[inline]
    pub fn pop(&mut self) {
        assert!(self.length != 0);
        self.length -= 1;
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

impl<T, const SIZE: usize> Index<usize> for List<T, SIZE> {
    type Output = T;

    fn index(&self, index: usize) -> &Self::Output {
        &self.items[index]
    }
}

impl<T, const SIZE: usize> IndexMut<usize> for List<T, SIZE> {
    fn index_mut(&mut self, index: usize) -> &mut Self::Output {
        &mut self.items[index]
    }
}
