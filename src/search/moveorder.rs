use std::cmp;

use crate::movegen::{
    action::Action,
    board::Board,
    genmoves::GenType,
    movelist::MoveList,
    types::{Color, Piece, Square},
};
use crate::search::alphabeta::MAX_DEPTH;

const MAX_HISTORY: i16 = 32_000;
const NUM_KILLERS: usize = 2;

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Stage {
    HashMove,
    GenCaptures,
    Captures,
    Killers,
    LCaptures,
    GenQuiets,
    Quiets,
}

pub struct StagedGenerator {
    stage: Stage,
    ttmove: Action,
    movelist: MoveList,
    ply: u16,
    idx: usize,
    lcapture_index: usize,
}

impl StagedGenerator {
    pub fn new(ttmove: Action, ply: u16) -> Self {
        Self {
            stage: Stage::HashMove,
            ttmove,
            movelist: MoveList::new(),
            ply,
            idx: 0,
            lcapture_index: 0,
        }
    }
    pub fn next_move(&mut self, ord: &OrderData, board: &mut Board) -> Option<(Action, Stage)> {
        let killers = &ord.killers[self.ply as usize];
        loop {
            match self.stage {
                Stage::HashMove => {
                    self.stage = Stage::GenCaptures;
                    if self.ttmove == Action::default() || !board.is_pseudolegal(self.ttmove) {
                        continue;
                    }

                    return Some((self.ttmove, Stage::HashMove));
                }
                Stage::GenCaptures => {
                    board.genmoves::<{ GenType::CAPTURES }>(&mut self.movelist);
                    // score captures using mvv/lva
                    for action in self.movelist.iter_mut() {
                        if **action != self.ttmove {
                            let score = board.mvv_lva(**action);
                            action.set_score(score)
                        }
                    }

                    self.stage = Stage::Captures;
                    self.idx = 0;
                    continue;
                }
                Stage::Captures => {
                    if let Some(action) = self
                        .movelist
                        .partial_insertion_sort(self.idx, |a| a.score())
                    {
                        let victim = board.get_piece(action.to());
                        let attacker = board.get_piece(action.from());
                        // if the capture is a high x low capture, check to make sure it doesn't lose material
                        if attacker as u8 > victim as u8 && attacker != Piece::K {
                            let see_value = board.see(*action);
                            // if it does lose material, place it in the "losing captures" list (beginning of the list).
                            // Losing captures are tried after killers, so we hope that we can get a cutoff
                            // from one of those instead.
                            if see_value < 0 {
                                self.movelist[self.idx].set_score(see_value);
                                // move the losing capture to the beginning of the list.
                                self.movelist.swap(self.idx, self.lcapture_index);
                                self.idx += 1;
                                self.lcapture_index += 1;
                                continue;
                            }
                        }

                        self.idx += 1;
                        return Some((*action, Stage::Captures));
                    } else {
                        // No more winning captures. next stage
                        self.stage = Stage::Killers;
                        // we know the number of l-captures. so shrink the list so we only consider them
                        self.movelist.shrink(self.lcapture_index);
                        self.idx = 0;
                        continue;
                    }
                }
                Stage::Killers => {
                    if self.idx < NUM_KILLERS {
                        let curr_killer = killers[self.idx];
                        // since killers can come from other branches at the same depth, they need to be checked
                        // for pseudolegality. (and make sure that they aren't captures)
                        if curr_killer != self.ttmove
                            && !board.is_color(curr_killer.to(), !board.active_color())
                            && board.is_pseudolegal(curr_killer)
                        {
                            self.idx += 1;
                            return Some((curr_killer, Stage::Killers));
                        } else {
                            // try the next killer
                            self.idx += 1;
                            continue;
                        }
                    } else {
                        // we've tried both killers. Go next.
                        self.stage = Stage::LCaptures;
                        self.idx = 0;
                        continue;
                    }
                }
                Stage::LCaptures => {
                    if let Some(action) = self
                        .movelist
                        .partial_insertion_sort(self.idx, |a| a.score())
                    {
                        assert!(*action != self.ttmove);

                        self.idx += 1;
                        return Some((*action, Stage::LCaptures));
                    } else {
                        self.stage = Stage::GenQuiets;
                        self.idx = 0;
                        continue;
                    }
                }
                Stage::GenQuiets => {
                    board.genmoves::<{ GenType::QUIETS }>(&mut self.movelist);
                    for action in self.movelist.iter_mut() {
                        if **action != self.ttmove
                            && **action != killers[0]
                            && **action != killers[1]
                        {
                            let piece = board.get_piece(action.from());
                            let score = ord.get_history(piece, board.active_color(), action.to());
                            action.set_score(score);
                        } else {
                            action.set_score(i16::MIN);
                        }
                    }

                    self.stage = Stage::Quiets;
                    continue;
                }
                Stage::Quiets => {
                    if let Some(action) = self
                        .movelist
                        .partial_insertion_sort(self.idx, |a| a.score())
                    {
                        self.idx += 1;
                        assert!(
                            *action != self.ttmove
                                && *action != killers[0]
                                && *action != killers[1]
                        );
                        return Some((*action, Stage::Quiets));
                    } else {
                        return None;
                    }
                }
            }
        }
    }
}

pub struct QSearchGenerator {
    movelist: MoveList,
    curr_idx: usize,
}

impl QSearchGenerator {
    pub fn new(board: &mut Board) -> Self {
        let mut list = MoveList::new();
        board.genmoves::<{ GenType::CAPTURES }>(&mut list);
        let mut generator = QSearchGenerator {
            movelist: list,
            curr_idx: 0,
        };
        generator.score_moves(board);
        generator
    }

    fn score_moves(&mut self, board: &Board) {
        for action in self.movelist.iter_mut() {
            let value = board.mvv_lva(**action);
            action.set_score(value)
        }
    }
}

impl Iterator for QSearchGenerator {
    type Item = Action;

    fn next(&mut self) -> Option<Self::Item> {
        let next_action = self
            .movelist
            .partial_insertion_sort(self.curr_idx, |a| a.score())?;
        self.curr_idx += 1;
        Some(*next_action)
    }
}
pub struct OrderData {
    pub killers: [[Action; 2]; MAX_DEPTH],
    pub history: [[[i16; 64]; 6]; 2], // color, piece, destination sqr
}

impl OrderData {
    pub fn new() -> Self {
        Self {
            history: [[[0; MAX_DEPTH]; 6]; 2],
            killers: [[Action::default(); NUM_KILLERS]; MAX_DEPTH],
        }
    }
    pub fn update_killer(&mut self, action: Action, ply: u16) {
        let firstkiller = self.killers[ply as usize][0];
        if firstkiller != action {
            self.killers[ply as usize][1] = firstkiller;
            self.killers[ply as usize][0] = action;
        }
    }

    pub fn get_history(&self, piece: Piece, color: Color, square: Square) -> i16 {
        self.history[color as usize][piece as usize][square as usize]
    }

    pub fn update_history(&mut self, action: Action, depth: u8, board: &Board) {
        let piece_moved = board.get_piece(action.from());
        let depth = depth as i16;
        self.history[board.active_color() as usize][piece_moved as usize][action.to() as usize] =
            cmp::min(
                self.history[board.active_color() as usize][piece_moved as usize]
                    [action.to() as usize]
                    + (depth * depth),
                MAX_HISTORY,
            );
    }

    pub fn age_history(&mut self) {
        for i in self.history.iter_mut() {
            for j in i.iter_mut() {
                for k in j.iter_mut() {
                    *k /= 2
                }
            }
        }
    }

    pub fn clear(&mut self) {
        self.history = [[[0; MAX_DEPTH]; 6]; 2];
        self.killers = [[Action::default(); NUM_KILLERS]; MAX_DEPTH];
    }
}
