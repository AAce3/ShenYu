use crate::{
    board_state::{
        board::Board,
        typedefs::{Color, Piece, Square, BISHOP, KING, KNIGHT, NOPIECE, PAWN, QUEEN, ROOK},
    },
    move_generation::{
        action::{Action, Move, ShortMove},
        makemove::PASSANT,
        movelist::{MoveList, ScoreList},
    },
};

pub struct OrderData {
    pub history: [[[i16; 64]; 6]; 2],
    pub killers: [[ShortMove; 2]; 256],
}

impl OrderData {
    pub fn update_killer(&mut self, ply: u16, cutoffmove: Move) {
        let cutoffmove = cutoffmove as u16; // casting truncates
        let killers = &mut self.killers[ply as usize];
        if killers[0] == cutoffmove {
        } else {
            let sl1_killer = killers[0];
            killers[1] = sl1_killer;
            killers[0] = cutoffmove;
        }
    }
}

enum Stage {
    TTMove,
    WCaptures,
    //countermove
    Killers,
    LCaptures,
    Quiets,
}

pub struct MovePicker<'a> {
    board: &'a Board,
    orderdata: &'a OrderData,
    ply: u8,
    curr_idx: usize,
    curr_scorelist: ScoreList,
    curr_mvlist: MoveList,
    curr_stage: Stage,
    // exclude these after they are generated
    ttmove: ShortMove,
    generated_killer_1: ShortMove,
    generated_killer_2: ShortMove,
}

impl Iterator for MovePicker<'_> {
    type Item = Move;

    fn next(&mut self) -> Option<Self::Item> {
        match self.curr_stage {
            Stage::TTMove => {
                self.curr_stage = Stage::WCaptures;
                // we should have already verified that the ttmove is good
                return Some(self.ttmove.to_longmove(self.board));
            }
            Stage::WCaptures => {
                if self.curr_mvlist.length == 0 {
                    self.curr_mvlist = self.board.generate_moves(false, true);
                    for i in 0..self.curr_mvlist.length {
                        let action = self.curr_mvlist[i as usize];
                        self.curr_scorelist[i as usize] = self.board.capture_order(action);
                    }
                }
                let mut maxscore = i16::MIN;
                let mut bestidx = self.curr_idx;
                let mut second_best_score = i16::MIN;
                // find next
                for i in self.curr_idx..(self.curr_scorelist.length as usize) {
                    if self.curr_scorelist[i] > maxscore {
                        second_best_score = maxscore;
                        maxscore = self.curr_scorelist[i];
                        bestidx = i;
                    } else if self.curr_scorelist[i] > second_best_score {
                        second_best_score = self.curr_scorelist[i]; // we want to store the second best score as well
                    }
                }
                const WORST_GOOD_SCORE: i16 = 0;
                self.curr_scorelist.swap(self.curr_idx, bestidx);
                self.curr_mvlist.swap(self.curr_idx, bestidx);
                let bestmove = self.curr_mvlist[self.curr_idx];
                self.curr_idx += 1;
                if bestmove as u16 == self.ttmove {
                    // avoid generating moves we have already checked
                    return self.next();
                }

                if self.curr_idx as u8 == self.curr_mvlist.length
                    || second_best_score < WORST_GOOD_SCORE
                {
                    // we've seen all good captures.
                    // the next best score is worse than the worst "good" score, so we go into the next stage.
                    self.curr_stage = Stage::Killers;
                }

                return Some(bestmove);
            }
            Stage::Killers => {
                let killers = self.orderdata.killers[self.ply as usize];
                // test k1 and k2
                if self.generated_killer_1 == 0 {
                    self.generated_killer_1 = killers[0];
                    // check for legality of killer
                } else if self.generated_killer_2 == 0 {
                    self.generated_killer_2 = killers[1];
                    // verify killers
                } else {
                    self.curr_stage = Stage::LCaptures;
                }
            }
            Stage::LCaptures => {
                let mut maxscore = i16::MIN;
                let mut bestidx = self.curr_idx;
                // resume prev movelist
                for i in self.curr_idx..(self.curr_scorelist.length as usize) {
                    if self.curr_scorelist[i] > maxscore {
                        maxscore = self.curr_scorelist[i];
                        bestidx = i;
                    }
                }

                self.curr_scorelist.swap(self.curr_idx, bestidx);
                self.curr_mvlist.swap(self.curr_idx, bestidx);
                let bestmove = self.curr_mvlist[self.curr_idx];
                self.curr_idx += 1;
                if bestmove as u16 == self.ttmove
                    || bestmove as u16 == self.generated_killer_1
                    || bestmove as u16 == self.generated_killer_2
                {
                    return self.next();
                }
                self.curr_idx += 1;
                if self.curr_idx as u8 == self.curr_mvlist.length {
                    self.curr_mvlist = MoveList::new();
                    self.curr_scorelist = ScoreList::new();
                    self.curr_idx = 0;
                    self.curr_stage = Stage::Quiets;
                }
                return Some(bestmove);
            }
            Stage::Quiets => {
                if self.curr_mvlist.length == 0 {
                    self.curr_mvlist = self.board.generate_moves(true, false);
                    for i in 0..self.curr_mvlist.length {
                        let action = self.curr_mvlist[i as usize];
                        // assign history score
                        let piecemoved = action.piece_moved(self.board);
                        let squareto = action.move_to();
                        let history = self.orderdata.history[self.board.tomove as usize]
                            [piecemoved as usize][squareto as usize];
                        self.curr_scorelist[i as usize] = history;
                    }
                }
                if self.curr_scorelist.length as usize == self.curr_idx {
                    return None;
                }
                let mut maxscore = i16::MIN;
                let mut bestidx = self.curr_idx;
                // resume prev movelist
                for i in self.curr_idx..(self.curr_scorelist.length as usize) {
                    if self.curr_scorelist[i] > maxscore {
                        maxscore = self.curr_scorelist[i];
                        bestidx = i;
                    }
                }

                self.curr_scorelist.swap(self.curr_idx, bestidx);
                self.curr_mvlist.swap(self.curr_idx, bestidx);
                let bestmove = self.curr_mvlist[self.curr_idx];
                self.curr_idx += 1;
                if bestmove as u16 == self.ttmove
                    || bestmove as u16 == self.generated_killer_1
                    || bestmove as u16 == self.generated_killer_2
                {
                    return self.next();
                }
                return Some(bestmove);
            }
        }
        None
    }
}

impl Board {
    fn check_legality(&self, action: ShortMove) -> bool {
        let piecemoved = self.get_at_square(action.move_from());
        let is_correct_color = self.is_color(action.move_from(), self.tomove);
        if is_correct_color && piecemoved != NOPIECE {
            match piecemoved {
                PAWN => {
                    if self.is_empty(action.move_to()) {
                        if action.move_type() == PASSANT {
                            return self.passant_square.is_some_and(|&x| x == action.move_to());
                        } else {
                            let diff = action.move_from() as i8 - action.move_to() as i8;
                            const DIFFS: [i8; 2] = [-8, 8];
                            let doublepush = DIFFS[self.tomove as usize] * 2;
                            if doublepush && action.move_to() >> 3 == 3 {
                                // doublepush and on 4th rank
                                return true;
                            } else if DIFFS[self.tomove as usize] == diff {
                                return true;
                            }
                        }
                    }
                    todo!()
                }
                KNIGHT => {
                    todo!()
                }
                BISHOP => {
                    todo!()
                }
                ROOK => {
                    todo!()
                }
                QUEEN => {
                    todo!()
                }
                KING => {
                    todo!()
                }
                _ => return false,
            }
        }
        false
    }
}

pub const MATERIAL_VALUES: [i16; 7] = [0, 100, 315, 320, 500, 900, 0];
impl Board {
    #[inline]
    pub fn capture_order(&self, action: Move) -> i16 {
        let attacker = action.piece_moved(self);
        let victim = self.get_at_square(action.move_to());
        let attackerval = MATERIAL_VALUES[attacker as usize];
        let victimval = MATERIAL_VALUES[victim as usize];
        if attackerval > victimval {
            self.see(action)
        } else {
            attackerval - victimval / 16
        }
    }
}
