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
    pub fn new(board: &mut Board, ttmove: u16, ord: &OrderData, ply: u16) -> MovePicker {
        let mut picker = MovePicker {
            movelist: board.generate_moves::<true, true>(),
            scorelist: List::new(),
            curr_idx: 0,
        };
        picker.score_moves(board, ttmove, ord, ply);
        picker
    }

    pub fn new_capturepicker(board: &mut Board, ord: &OrderData) -> MovePicker {
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
    pub fn clear(&mut self) {
        self.history = [[[0; 64]; 6]; 2];
        self.killers = [[0; 2]; 256];
    }
}

impl Board {
    pub fn order_capture(&self, action: Move) -> i16 {
        let victim = self.get_at_square(action.move_to());
        let attacker = action.piece_moved();
        SEEVALUES[victim as usize] - (SEEVALUES[attacker as usize] / 8)
    }
}

pub struct StagedGenerator<'a> {
    pub ttmove: Move,
    pub curr_stage: Stage,
    captures: List<Move>,
    capture_scores: List<i16>,
    killers: [Move; 2],
    losing_captures: List<Move>,
    lcapture_scores: List<i16>,
    quiets: List<Move>,
    quiet_scores: List<i16>,
    current_index: usize,
    pub board: &'a mut Board,
    ply: u16,
}

impl StagedGenerator<'_> {
    pub fn next(&mut self, ord: &OrderData) -> Option<Move> {
        match self.curr_stage {
            Stage::TTMove => Some(self.ttmove), 
            // if we have a ttmove, return it. TTmove is checked beforehand to make sure that hash code is valid
            Stage::GenCaptures => {
                self.captures = self.board.generate_moves::<false, true>();
                self.capture_scores.length = self.captures.length;
                for i in 0..self.captures.length {
                    // score captures by mvv lva ordering
                    // high value targets take priority, low value attackers slightly alter that
                    let action = self.captures[i as usize];
                    let score = self.board.order_capture(action);
                    self.capture_scores[i as usize] = score;
                }
                // go to capture stage
                self.curr_stage = Stage::Captures;
                self.current_index = 0;
                self.next(ord)
            }
            Stage::Captures => {
                let ended = self.lazy_insertion_sort(); // swap the next best move to choose to the current index.
                if ended {
                    self.curr_stage = Stage::GenQuiets;
                    self.current_index = 0;
                    return self.next(ord);
                }
                let currbest = self.captures[self.current_index];
                let seevalue = self.board.see(currbest);
                // if the capture is a losing capture by SEE, put it on the losing captures list and skip it.
                if seevalue < 0 {
                    self.losing_captures.push(currbest);
                    self.lcapture_scores.push(seevalue);
                    self.current_index += 1;
                    self.next(ord)
                } else {
                    self.current_index += 1;
                    Some(currbest)
                }
            }
            Stage::GenQuiets => {
                self.quiets = self.board.generate_moves::<true, false>();
                for i in 0..self.quiets.length {
                    let action = self.quiets[i as usize];
                    match action {
                        // if they match the killers, store them
                        action if action == ord.killers[self.ply as usize][0] => {
                            self.killers[0] = action;
                            // give them a really bad score so that they don't get sorted with the quiets.
                            self.quiet_scores[i as usize] = i16::MIN;
                        }
                        action if action == ord.killers[self.ply as usize][1] => {
                            self.killers[1] = action;
                            self.quiet_scores[i as usize] = i16::MIN;
                        }
                        _ => {
                            // otherwise, score them via history heuristic
                            let score = ord.get_history(
                                action.piece_moved(),
                                self.board.tomove,
                                action.move_to(),
                            );
                            self.quiet_scores[i as usize] = score;
                        }
                    }
                }
                self.curr_stage = Stage::Killers;
                self.current_index = 0;
                self.next(ord)
            }
            Stage::Killers => {
                if self.current_index >= 2 {
                    self.curr_stage = Stage::LCaptures;
                    self.current_index = 0;
                }
                for (index, action) in self.killers.iter().skip(self.current_index).enumerate() {
                    // loop through both killers. Check if they were stored.
                    if *action != 0 {
                        // if they were stored, immediately stop and return. Increment the index.
                        self.current_index = index + 1;
                        return Some(*action);
                    }
                }
                // If none match, go to the next stage
                self.curr_stage = Stage::LCaptures;
                self.current_index = 0;
                self.next(ord)
            }
            Stage::LCaptures => {
                let ended = self.lazy_insertion_sort();
                if ended {
                    self.curr_stage = Stage::Quiets;
                    self.current_index = 0;
                    return self.next(ord);
                }
                let bestmove = self.losing_captures[self.current_index];
                self.current_index += 1;
                Some(bestmove)
            }
            Stage::Quiets => {
                let ended = self.lazy_insertion_sort();
                if ended {
                    return None;
                }
                let bestmove = self.quiets[self.current_index];
                if bestmove == self.killers[0] || bestmove == self.killers[1] { 
                    // killers should be at the end, since we assigned them the worst score. 
                    // If we get there, return immediately, as we have seen all possible moves.
                    return None;
                }
                self.current_index += 1;
                Some(bestmove)
            }
        }
    }

    // finds the next highest and places it at the current spot, returning true if we are out of bounds
    pub fn lazy_insertion_sort(&mut self) -> bool {
        let (movelist, scorelist) = match self.curr_stage {
            Stage::Captures => (&mut self.captures, &mut self.capture_scores),
            Stage::LCaptures => (&mut self.losing_captures, &mut self.lcapture_scores),
            Stage::Quiets => (&mut self.quiets, &mut self.quiet_scores),
            _ => return true,
        };
        if self.current_index >= movelist.length as usize {
            return true;
        }
        let mut max_score = i16::MIN;
        let mut best_idx = self.current_index;
        for i in self.current_index..(movelist.length as usize) {
            let score = scorelist[i];
            if score > max_score {
                max_score = score;
                best_idx = i;
            }
        }

        movelist.swap(self.current_index, best_idx);
        scorelist.swap(self.current_index, best_idx);
        false
    }
}

pub enum Stage {
    TTMove,
    GenCaptures,
    Captures,
    GenQuiets,
    Killers,
    LCaptures,
    Quiets,
}
