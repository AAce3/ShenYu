use std::{mem, ops::{Index, IndexMut}};

use super::action::Move;

pub struct MoveList {
    pub moves: [Move; 255],
    pub length: u8,
}

impl Default for MoveList {
    fn default() -> Self {
        Self::new()
    }
}

impl MoveList {
    #[allow(clippy::uninit_assumed_init)]
    pub fn new() -> Self {
        Self {
            moves: unsafe { mem::MaybeUninit::uninit().assume_init() },
            length: 0,
        }
    }
    #[inline]
    pub fn push(&mut self, item: Move) {
        self.moves[self.length as usize] = item;
        self.length += 1;
    }

    #[inline]
    pub fn swap(&mut self, idx1: usize, idx2: usize){
        self.moves.swap(idx1, idx2);
    }
}
impl Index<usize> for MoveList {
    type Output = Move;

    fn index(&self, index: usize) -> &Self::Output {
        assert!(index < self.length as usize);
        &self.moves[index]
    }
}

pub struct ScoreList{
    pub scores: [i16; 256],
    pub length: u8,
}

impl Default for ScoreList {
    fn default() -> Self {
        Self::new()
    }
}

impl ScoreList {
    #[allow(clippy::uninit_assumed_init)]
    pub fn new() -> Self {
        Self {
            scores: unsafe { mem::MaybeUninit::uninit().assume_init() },
            length: 0,
        }
    }
    #[inline]
    pub fn push(&mut self, item: i16) {
        self.scores[self.length as usize] = item;
        self.length += 1;
    }

    #[inline]
    pub fn swap(&mut self, idx1: usize, idx2: usize){
        self.scores.swap(idx1, idx2);
    }
}

impl IndexMut<usize> for ScoreList{
    fn index_mut(&mut self, index: usize) -> &mut Self::Output {
        &mut self.scores[index]
    }
}
impl Index<usize> for ScoreList {
    type Output = i16;

    fn index(&self, index: usize) -> &Self::Output {
        assert!(index < self.length as usize);
        &self.scores[index]
    }
}