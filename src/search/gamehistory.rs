use crate::{
    board_state::{board::Board, zobrist::ZobristKey},
    move_generation::list::List,
};
#[derive(PartialEq, Eq)]
pub struct GameHistory {
    pub positions: List<ZobristKey>,
}
impl Default for GameHistory {
    fn default() -> Self {
        Self::new()
    }
}
impl GameHistory {
    pub fn store(&mut self, key: ZobristKey) {
        self.positions.push(key);
    }

    pub fn retract(&mut self) {
        if self.positions.length <= 1 {
            return;
        }
        self.positions.pop();
    }
    pub fn new() -> Self {
        let mut list = List::new();
        let startpos =
            Board::parse_fen("rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1").unwrap();
        list.push(startpos.zobrist_key);
        Self { positions: list }
    }

    pub fn find(&self, hmc: u8) -> bool {
        assert_ne!(self.positions.length, 0);
        let start = self.positions.length - 1;
        if hmc < 4 {
            return false;
        }
        let item = self.positions[start as usize];
        for i in (0..(self.positions.length))
            .rev()
            .take(hmc as usize + 1)
            .step_by(2)
            .skip(1)
        {
            if self.positions[i as usize] == item {
                return true;
            }
        }
        false
    }
}
