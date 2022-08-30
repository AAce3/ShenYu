use fastdivide::DividerU64;

use crate::{board_state::zobrist::ZobristKey, move_generation::action::ShortMove};
// 32 bits: zob key
// 16 bits: shortmove
// 16 bits: score (could use less but negatives are weird)
// 2 bits: node type
// 6 bits: depth (did you really expect to get to depth 64?)

pub struct TranspositionTable {
    table: Vec<Bucket>,
    divide_by: DividerU64,
}

const CONVERSION: usize = (1 << 20) / 64;
impl TranspositionTable {
    pub fn new(size: usize) -> Self {
        // size is in MB
        let entries = size * CONVERSION;
        Self {
            table: vec![Bucket::new(); entries],
            divide_by: DividerU64::divide_by(entries as u64),
        }
    }
    pub fn probe(&mut self, key: ZobristKey) -> *mut Entry {
        let idx = self.divide_by.divide(key);
        self.table[idx as usize].probe(key)
    }
}
#[derive(Clone, Copy)]
#[repr(align(64))]
pub struct Bucket {
    pub mcts_idx: u32,
    pub mcts_hash: u32,
    pub entries: [Entry; 6],
}

impl Default for Bucket {
    fn default() -> Self {
        Self::new()
    }
}

impl Bucket {
    pub fn new() -> Self {
        Self {
            mcts_idx: 0,
            mcts_hash: 0,
            entries: [Entry::new(); 6],
        }
    }
    pub fn probe(&mut self, key: u64) -> *mut Entry {
        &mut self.entries[key as usize % 6]
    }
}

type NodeType = u8;
pub const NULL: NodeType = 0;
pub const BETA: NodeType = 0b10;
pub const EXACT: NodeType = 0b11;

#[repr(packed)]
#[derive(Clone, Copy)]
pub struct Entry {
    pub key: u32,
    pub bestmove: ShortMove,
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
            bestmove: 0,
            score: 0,
            otherdata: 0,
        }
    }
    pub fn create(key: u64, bestmove: ShortMove, eval: i16, depth: u8, nodetype: NodeType) -> Self {
        let relevant_key = (key >> 32) as u32;
        let data = (depth << 2) | nodetype;
        Self {
            key: relevant_key,
            bestmove,
            score: eval,
            otherdata: data,
        }
    }
    pub fn get_depth(&self) -> u8 {
        self.otherdata >> 2
    }

    pub fn get_nodetype(&self) -> NodeType {
        self.otherdata & 0b11
    }

    pub fn key_equals(&self, key: ZobristKey) -> bool {
        ((self.key as u64) << 32) ^ key == 0
    }
}
