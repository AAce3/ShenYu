use std::cmp;

use crate::{
    board_state::{
        board::Board,
        typedefs::{Color, Piece, Square},
    },
    move_generation::{
        action::{Action, Move, ShortMove},
        makemove::PASSANT,
        movelist::{MoveList, ScoreList},
    },
};

const W_CAPTURE_OFFSET: i16 = 24_000;
const KILLER_OFFSET: i16 = 20_000;
const L_CAPTURE_OFFSET: i16 = 16_000;
const MAX_HISTORY_SCORE: i16 = 15_000;

pub struct MoveSorter<'a> {
    moves: MoveList,
    scores: ScoreList,
    curr_idx: usize,
    ttmove: ShortMove,
    board: &'a Board,
}
impl MoveSorter<'_> {
    pub fn new(board: &Board, ttmove: ShortMove) -> MoveSorter {
        MoveSorter {
            moves: board.generate_moves::<true, true>(),
            scores: ScoreList::new(0),
            curr_idx: 0,
            ttmove,
            board,
        }
    }
    pub fn next(&mut self, history: &History, killers: &[Move; 2]) -> Option<Move> {
        if self.curr_idx == self.moves.length as usize {
            return None;
        }
        let mut best_score = i16::MIN;
        let mut best_idx = self.curr_idx;
        if self.scores.length == 0 {
            // score moves
            self.scores.length = self.moves.length;
            for i in 0..self.moves.length {
                let action = self.moves[i as usize];
                let value = if action as u16 == self.ttmove {
                    i16::MAX
                } else if action.is_capture(self.board) {
                    let score = self.board.capture_order(action);
                    if score % 2 == 0 {
                        // it's a winning capture
                        W_CAPTURE_OFFSET + score
                    } else {
                        // it's a losing capture
                        L_CAPTURE_OFFSET + score
                    }
                } else if action == killers[0] {
                    KILLER_OFFSET + 2
                } else if action == killers[1] {
                    KILLER_OFFSET + 1
                } else {
                    history.get_history(
                        self.board.tomove,
                        action.piece_moved(self.board),
                        action.move_to(),
                    )
                };
                if value > best_score {
                    best_idx = i as usize;
                    best_score = value;
                }
            }
        } else {
            for i in self.curr_idx..(self.scores.length as usize) {
                let score = self.scores[i];
                if score > best_score {
                    best_idx = i;
                    best_score = score;
                }
            }
        }
        let bestmove = self.moves[best_idx];
        self.moves.swap(best_idx, self.curr_idx);
        self.scores.swap(best_idx, self.curr_idx);

        self.curr_idx += 1;
        Some(bestmove)
    }
}
pub struct History {
    pub history: [[[i16; 64]; 6]; 2],
}

impl History {
    pub fn get_history(&self, color: Color, piece: Piece, square: Square) -> i16 {
        self.history[color as usize][piece as usize - 1][square as usize]
    }
    pub fn increment_history(&mut self, color: Color, piece: Piece, moveto: Square, depth: u8) {
        let depth = depth as i16;
        let historyval = self.history[color as usize][piece as usize - 1][moveto as usize];
        self.history[color as usize][piece as usize - 1][moveto as usize] =
            cmp::min(historyval + depth * depth, MAX_HISTORY_SCORE);
    }
    
    pub fn age(&mut self) {
        for color in self.history.iter_mut() {
            for piecetype in color.iter_mut() {
                for square in piecetype.iter_mut() {
                    *square /= 2
                }
            }
        }
    }
}
pub struct Killer {
    killers: [[Move; 2]; 256],
}

impl Killer {
    pub fn update_killer(&mut self, ply: u16, cutoffmove: Move) {
        let cutoffmove = cutoffmove;
        let killers = &mut self.killers[ply as usize];
        if killers[0] == cutoffmove {
        } else {
            let sl1_killer = killers[0];
            killers[1] = sl1_killer;
            killers[0] = cutoffmove;
        }
    }
}

pub const MATERIAL_VALUES: [i16; 7] = [0, 100, 316, 320, 500, 900, 0];
impl Board {
    #[inline]
    pub fn capture_order(&self, action: Move) -> i16 {
        if action.move_type() == PASSANT {
            return 100;
        }
        let attacker = action.piece_moved(self);
        let victim = self.get_at_square(action.move_to());
        let attackerval = MATERIAL_VALUES[attacker as usize];
        let victimval = MATERIAL_VALUES[victim as usize];
        if attackerval > victimval {
            (victimval - attackerval / 8) - 1
        } else {
            victimval - attackerval / 8
        }
    }
}
