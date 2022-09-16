use std::cmp;

use crate::{
    board_state::{
        board::Board,
        typedefs::{Color, Piece, Square},
    },
    move_generation::{
        action::{Action, Move},
        list::List,
    },
};

use super::see::SEEVALUES;

pub struct MovePicker {
    pub movelist: List<Move>,
    pub scorelist: List<i16>,
    pub curr_idx: usize,
}

const CAPTURE_OFFSET: i16 = 20_000;
const MAX_HISTORY: i16 = 18_000;
impl MovePicker {
    pub fn new(board: &Board, ttmove: u16, ord: &OrderData, ply: u16) -> MovePicker {
        let mut picker = MovePicker {
            movelist: board.generate_moves::<true, true>(),
            scorelist: List::new(),
            curr_idx: 0,
        };
        picker.score_moves(board, ttmove, ord, ply);
        picker
    }

    pub fn new_capturepicker(board: &Board, ord: &OrderData) -> MovePicker {
        let mut picker = MovePicker {
            movelist: board.generate_moves::<false, true>(),
            scorelist: List::new(),
            curr_idx: 0,
        };
        picker.score_moves(board, 0, ord, 0);
        picker
    }

    pub fn score_moves(&mut self, board: &Board, ttmove: u16, ord: &OrderData, ply: u16) {
        self.scorelist.length = self.movelist.length;
        for i in 0..self.movelist.length {
            let action = self.movelist[i as usize];
            let value = if action as u16 == ttmove {
                25_000
            } else if action.is_capture() {
                CAPTURE_OFFSET + board.order_capture(action)
            } else if action == ord.killers[ply as usize][0]
                || action == ord.killers[ply as usize][1]
            {
                CAPTURE_OFFSET
            } else {
                ord.get_history(action.piece_moved(), board.tomove, action.move_to())
            };
            self.scorelist[i as usize] = value;
        }
    }
}
// Looks through the moves to find the one with a highest score, returning none if there are no more moves
impl Iterator for MovePicker {
    type Item = Move;

    fn next(&mut self) -> Option<Self::Item> {
        if self.curr_idx as u8 >= self.scorelist.length {
            return None;
        }
        let mut bestidx = self.curr_idx;
        let mut best_score = i16::MIN;
        for i in self.curr_idx..(self.scorelist.length as usize) {
            if self.scorelist[i as usize] > best_score {
                bestidx = i;
                best_score = self.scorelist[i as usize];
            }
        }
        self.scorelist.swap(self.curr_idx, bestidx as usize);
        self.movelist.swap(self.curr_idx, bestidx as usize);
        let bestmove = self.movelist[self.curr_idx];
        self.curr_idx += 1;
        Some(bestmove)
    }
}

pub struct OrderData {
    pub killers: [[Move; 2]; 256],
    pub history: [[[i16; 64]; 6]; 2],
}

impl OrderData {
    pub fn update_killer(&mut self, action: Move, ply: u16) {
        let firstkiller = self.killers[ply as usize][0];
        if firstkiller != action {
            self.killers[ply as usize][1] = firstkiller;
            self.killers[ply as usize][0] = action;
        }
    }

    pub fn get_history(&self, piece: Piece, color: Color, square: Square) -> i16 {
        self.history[color as usize][piece as usize - 1][square as usize]
    }

    pub fn update_history(&mut self, action: Move, depth: u8, board: &Board) {
        let depth = depth as i16;
        self.history[board.tomove as usize][action.piece_moved() as usize - 1]
            [action.move_to() as usize] = cmp::min(
            self.history[board.tomove as usize][action.piece_moved() as usize - 1]
                [action.move_to() as usize]
                + (depth * depth),
            MAX_HISTORY,
        );
    }

    pub fn age_history(&mut self) {
        for color in self.history.iter_mut() {
            for piece in color.iter_mut() {
                for square in piece.iter_mut() {
                    *square /= 2
                }
            }
        }
    }
    pub fn clear(&mut self){
        self.history = [[[0; 64]; 6]; 2];
        self.killers =  [[0; 2]; 256];
    }
}

impl Board {
    pub fn order_capture(&self, action: Move) -> i16 {
        let victim = self.get_at_square(action.move_to());
        let attacker = action.piece_moved();
        SEEVALUES[victim as usize] - (SEEVALUES[attacker as usize] / 8)
    }
}
