use std::cmp;

use crate::{
    board_state::{
        bitboard::{Bitboard, BB},
        board::Board,
        typedefs::{Color, Piece, Square, BISHOP, KING, KNIGHT, PAWN, QUEEN, ROOK, WHITE},
    },
    move_generation::{
        action::{Action, Move},
        list::List,
        makemove::{CASTLE, PASSANT},
        movegen::{
            BLOCK_CHECKED_KINGSIDE, BLOCK_CHECKED_QUEENSIDE, BLOCK_OCCUPIED_KINGSIDE,
            BLOCK_OCCUPIED_QUEENSIDE, INBETWEENS,
        },
    },
};

use super::see::SEEVALUES;

pub struct CapturePicker {
    pub movelist: List<Move>,
    pub scorelist: List<i16>,
    pub curr_idx: usize,
}

const MAX_HISTORY: i16 = 18_000;
impl CapturePicker {
    pub fn new_capturepicker(board: &mut Board) -> CapturePicker {
        let mut picker = CapturePicker {
            movelist: board.generate_moves::<false, true>(),
            scorelist: List::new(),
            curr_idx: 0,
        };
        picker.score_moves(board);
        picker
    }

    pub fn score_moves(&mut self, board: &Board) {
        self.scorelist.length = self.movelist.length;
        for i in 0..self.movelist.length {
            let action = self.movelist[i as usize];
            let value = board.order_capture(action);
            self.scorelist[i as usize] = value;
        }
    }
}
// Looks through the moves to find the one with a highest score, returning none if there are no more moves
impl Iterator for CapturePicker {
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
        SEEVALUES[victim as usize] - (SEEVALUES[attacker as usize] / 16)
    }

    pub fn check_move(&mut self, action: Move) -> bool {
        action != 0
            && self.get_at_square(action.move_from()) == action.piece_moved()
            && self.is_color(action.move_from(), self.tomove)
            && if action.is_capture() {
                self.is_color(action.move_to(), !self.tomove)
                    && self.get_pieces(KING, !self.tomove).lsb() != action.move_to()
            // don't capture enemy king
            } else {
                !self.is_occupied(action.move_to())
            }
            && match action.piece_moved() {
                KNIGHT => true,
                BISHOP | ROOK | QUEEN => {
                    INBETWEENS[action.move_from() as usize][action.move_to() as usize]
                        & self.get_occupancy()
                        == 0
                }
                PAWN => {
                    if action.is_pawn_doublepush() {
                        // if it's a pawn doublepush, it only could have passed previous predicates if it actually was a pawn on that square
                        // and it only could have been generated if there actually was an available doublepush.
                        // So, we only have to check that it
                        let forward_one = if self.tomove == WHITE {
                            action.move_from() + 8
                        } else {
                            action.move_from() - 8
                        };
                        self.is_empty(forward_one)
                    } else if action.move_type() == PASSANT {
                        self.passant_square == Some(action.move_to())
                    } else {
                        true
                    }
                }
                KING => {
                    let occupancy = self.get_occupancy();
                    let kingbb = self.get_pieces(KING, self.tomove);
                    let blind_board = occupancy ^ kingbb; // ray-attackers "see through" the king
                    let atk_mask = self.generate_atk_mask(!self.tomove, blind_board);
                    let rights = self.get_castlerights(self.tomove);
                    Bitboard::new(action.move_to()) & atk_mask == 0
                        && (action.move_type() != CASTLE || {
                            let is_kingside = action.move_to() > action.move_from();
                            if is_kingside {
                                rights & 0b10 != 0
                                    && BLOCK_OCCUPIED_KINGSIDE[self.tomove as usize] & occupancy
                                        == 0
                                    && BLOCK_CHECKED_KINGSIDE[self.tomove as usize] & atk_mask == 0
                            } else {
                                rights & 1 != 0
                                    && BLOCK_OCCUPIED_QUEENSIDE[self.tomove as usize] & occupancy
                                        == 0
                                    && BLOCK_CHECKED_QUEENSIDE[self.tomove as usize] & atk_mask == 0
                            }
                        })
                }

                _ => panic!("Invalid Piece"),
            }
            && {
                if action.piece_moved() != KING {
                    self.get_movemask() & Bitboard::new(action.move_to()) != 0
                        && (Bitboard::new(action.move_from()) & self.get_rpinmask() == 0
                            || Bitboard::new(action.move_to()) & self.get_rpinmask() != 0)
                        && (Bitboard::new(action.move_from()) & self.get_bpinmask() == 0
                            || Bitboard::new(action.move_to()) & self.get_bpinmask() != 0)
                } else if action.move_type() == PASSANT {
                    let newb = self.do_move(action);
                    newb.incheck(self.tomove)
                } else {
                    true
                }
            }
    }
}

pub struct StagedGenerator {
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
    pub board: Board,
    ply: u16,
}
impl StagedGenerator {
    pub fn new(ttmove: Move, board: Board, ply: u16) -> Self {
        Self {
            ttmove,
            curr_stage: Stage::TTMove,
            captures: List::new(),
            capture_scores: List::new(),
            killers: [0; 2],
            losing_captures: List::new(),
            lcapture_scores: List::new(),
            quiets: List::new(),
            quiet_scores: List::new(),
            current_index: 0,
            board,
            ply,
        }
    }
}
impl StagedGenerator {
    pub fn next(&mut self, ord: &OrderData) -> Option<Move> {
        match self.curr_stage {
            // if we have a ttmove, return it. TTmove is checked beforehand to make sure that hash code is valid
            Stage::TTMove => {
                self.curr_stage = Stage::GenCaptures;
                if self.ttmove == 0 {
                    return self.next(ord);
                }
                Some(self.ttmove)
            }

            Stage::GenCaptures => {
                self.captures = self.board.generate_moves::<false, true>();
                for i in 0..self.captures.length {
                    // score captures by mvv lva ordering
                    // high value targets take priority
                    let action = self.captures[i as usize];
                    let score = if action == self.ttmove {
                        i16::MIN // avoid searching the ttmove
                    } else {
                        self.board.order_capture(action)
                    };
                    self.capture_scores.push(score);
                }
                // go to next stage
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

                let victim = self.board.get_at_square(currbest.move_to());
                let attacker = currbest.piece_moved();

                // if the capture is a losing capture by SEE, put it on the losing captures list and skip it.
                if attacker > victim && attacker != KING {
                    let seevalue = self.board.see(currbest);
                    if seevalue < 0 {
                        self.losing_captures.push(currbest);
                        self.lcapture_scores.push(seevalue);
                        self.current_index += 1;
                        return self.next(ord);
                    }
                }
                self.current_index += 1;
                Some(currbest)
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
                            self.quiet_scores.push(i16::MIN);
                        }
                        action if action == ord.killers[self.ply as usize][1] => {
                            self.killers[1] = action;
                            self.quiet_scores.push(i16::MIN);
                        }
                        _ => {
                            // otherwise, score them via history heuristic
                            let score = if action == self.ttmove {
                                i16::MIN
                            } else {
                                ord.get_history(
                                    action.piece_moved(),
                                    self.board.tomove,
                                    action.move_to(),
                                )
                            };
                            self.quiet_scores.push(score);
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
                for i in self.current_index..2 {
                    let killer = self.killers[i];
                    if killer != 0 {
                        self.current_index = i + 1;
                        return Some(killer);
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
        if max_score == i16::MIN {
            // already generated moves are assigned i16::MIN
            // it is impossible for any other score to reach that low, so when we get here we know that there are no more moves
            return true;
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
