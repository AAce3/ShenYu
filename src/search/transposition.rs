use crate::{board_state::zobrist::ZobristKey, move_generation::action::ShortMove};
// 32 bits: zob key
// 16 bits: shortmove
// 16 bits: score (could use less but negatives are weird)
// 2 bits: node type
// 6 bits: depth (did you really expect to get to depth 64?)

pub struct TranspositionTable {
    table: Vec<Entry>,
}

const CONVERSION: usize = (1 << 20) / 16;
impl TranspositionTable {
    pub fn new(size: usize) -> Self {
        // size is in MB
        let entries = size * CONVERSION;
        Self {
            table: vec![Entry::new(); entries],
        }
    }
    pub fn probe(&mut self, key: ZobristKey) -> *mut Entry {
        let idx = key as usize & (self.table.len() - 1);
        &mut self.table[idx]
    }
    pub fn clear(&mut self) {
        self.table.clear();
    }
}


type NodeType = u8;
pub const NULL: NodeType = 0;
pub const BETA: NodeType = 0b10;
pub const EXACT: NodeType = 0b11;
pub const QNODE: NodeType = 0b01;


#[derive(Clone, Copy)]
#[repr(align(16))]
pub struct Entry {
    pub key: u64,
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
    pub fn store(&mut self, key: u64, bestmove: ShortMove, eval: i16, depth: u8, nodetype: NodeType) {
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

    pub fn key_equals(&self, key: ZobristKey) -> bool {
        self.key == key
    }
}
