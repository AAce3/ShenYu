use crate::move_generation::action::ShortMove;
// 32 bits: zob key
// 16 bits: shortmove
// 16 bits: score (could use less but negatives are weird)
// 2 bits: node type
// 6 bits: depth (did you really expect to get to depth 64?)

pub struct Bucket {
    pub mcts_idx: u32,
    pub mcts_hash: u32,
    pub entries: [Entry; 6],
}

impl Bucket {
    pub fn probe(&mut self, key: u64) -> &mut Entry {
        &mut self.entries[key as usize % 6]
    }
}

type NodeType = u8;
pub const NULL: NodeType = 0;
pub const UPPER: NodeType = 0b10;
pub const LOWER: NodeType = 0b01;
pub const EXACT: NodeType = 0b11;

#[repr(packed)]
pub struct Entry {
    pub key: u32,
    pub bestmove: ShortMove,
    pub score: i16,
    pub otherdata: u8,
}

impl Entry {
    pub fn create(key: u64, bestmove: ShortMove, eval: i16, depth: u8, nodetype: NodeType) -> Self{
        let relevant_key = (key >> 32) as u32;
        let data = (depth << 2) | nodetype;
        Self { key: relevant_key, bestmove, score: eval, otherdata: data }
    }
    pub fn get_depth(&self) -> u8 {
        self.otherdata >> 2
    }

    pub fn get_nodetype(&self) -> NodeType {
        self.otherdata & 0b11
    }
}
