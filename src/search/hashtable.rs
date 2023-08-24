
// 64 bits: zob key  8 bytes
// 16 bits: best move 2 bytes
// 16 bits: score 2 bytes
// 2 bits: node type 1 byte
// 6 bits: depth
// total: 13 bytes

use std::mem;

use crate::movegen::{action::Action, zobrist::Zobrist};

pub struct TranspositionTable {
    table: Vec<Entry>,
}

const CONVERSION: usize = (1 << 20) / mem::size_of::<Entry>();

impl TranspositionTable {
    pub fn new(size: usize) -> Self {
        // size is in MB
        let entries = size * CONVERSION;
        if entries == 0 {
            return Self {
                table: vec![Entry::new(); 1],
            };
        }
        Self {
            table: vec![Entry::new(); entries],
        }
    }
    pub fn probe(&mut self, key: Zobrist) -> *mut Entry {
        let idx = key as usize % self.table.len();
        &mut self.table[idx]
    }
    pub fn clear(&mut self) {
        for i in self.table.iter_mut() {
            (*i).store(0, Action::default(), 0, 0, 0);
        }
    }
}

type NodeType = u8;
pub const BETA: NodeType = 0b10;
pub const EXACT: NodeType = 0b11;
pub const ALPHA: NodeType = 0b01;

#[derive(Clone, Copy)]
#[repr(align(16))]
pub struct Entry {
    pub key: u64,
    pub bestmove: Action,
    pub score: i16,
    pub otherdata: u8,
}

impl Default for Entry {
    fn default() -> Self {
        Self::new()
    }
}

impl Entry {
    pub fn new() -> Entry {
        Entry {
            key: 0,
            bestmove: Action::default(),
            score: 0,
            otherdata: EXACT,
        }
    }

    pub fn store(&mut self, key: u64, bestmove: Action, eval: i16, depth: u8, nodetype: NodeType) {
        let data = (depth << 2) | nodetype;
        self.key = key;
        self.bestmove = bestmove;
        self.score = eval;
        self.otherdata = data;
    }

    pub fn get_depth(&self) -> u8 {
        self.otherdata >> 2
    }

    pub fn get_nodetype(&self) -> NodeType {
        self.otherdata & 0b11
    }

    pub fn key_equals(&self, key: Zobrist) -> bool {
        self.key == key
    }
}