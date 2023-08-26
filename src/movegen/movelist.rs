use super::action::ScoredMove;

use std::{
    mem,
    ops::{Index, IndexMut},
};

pub const MAX_MOVES: usize = 256;
pub type MoveList = List<ScoredMove, MAX_MOVES>;

pub struct List<T, const N: usize> {
    items: [T; N],
    length: usize,
}

impl<T, const N: usize> Default for List<T, N> {
    fn default() -> Self {
        Self::new()
    }
}

impl<T, const N: usize> List<T, N> {
    #[allow(clippy::uninit_assumed_init)]
    pub fn new() -> Self {
        Self {
            items: unsafe { mem::MaybeUninit::uninit().assume_init() },
            length: 0,
        }
    }

    pub fn push(&mut self, action: T) {
        if self.length < self.items.len() {
            self.items[self.length] = action;
            self.length += 1;
        }
    }

    pub fn len(&self) -> usize {
        self.length
    }

    pub fn swap(&mut self, idx1: usize, idx2: usize) {
        self.items.swap(idx1, idx2);
    }

    pub fn iter(&self) -> impl Iterator<Item = &T> {
        self.items.iter().take(self.len())
    }

    pub fn iter_mut(&mut self) -> impl Iterator<Item = &mut T> {
        let len = self.len();
        self.items.iter_mut().take(len)
    }

    pub fn shrink(&mut self, new_size: usize) {
        self.length = new_size;
    }

    pub fn clear(&mut self) {
        self.length = 0;
    }
}

impl<T: Copy, const N: usize> List<T, N> {
    pub fn partial_insertion_sort<F>(&mut self, idx: usize, value_func: F) -> Option<T>
    where
        F: Fn(&T) -> i16,
    {
        if idx >= self.len() {
            return None;
        }

        let mut max = i16::MIN;
        let mut best_idx = idx;

        for (i, item) in self.iter().skip(idx).enumerate() {
            let value = value_func(item);
            if value > max {
                max = value;
                best_idx = i + idx;
            }
        }

        if max == i16::MIN {
            return None;
        }

        self.swap(idx, best_idx);

        Some(self[idx])
    }
}

impl<T, const N: usize> Index<usize> for List<T, N> {
    type Output = T;

    fn index(&self, index: usize) -> &Self::Output {
        if index >= self.len() {
            panic!("Index {index} out of bounds for length {}", self.len())
        }
        &self.items[index]
    }
}

impl<T, const N: usize> IndexMut<usize> for List<T, N> {
    fn index_mut(&mut self, index: usize) -> &mut Self::Output {
        &mut self.items[index]
    }
}
