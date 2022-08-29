

use crate::{
    board_state::{
        bitboard::{Bitboard, BB},
        board::Board,
        typedefs::{Square, BISHOP, KING, KNIGHT, NOPIECE, PAWN, QUEEN, ROOK, Piece, Color},
    },
    move_generation::{
        action::{Action, Move, ShortMove},
        makemove::{C1, C8, CASTLE, G1, G8, PASSANT},
        masks::{KING_ATTACKS, PAWN_CAPTURES},
        movegen::INBETWEENS,
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
    pub fn get_history(&self, color: Color, piece: Piece, square: Square) -> i16{
        self.history[color as usize][piece as usize - 1][square as usize]
    }
}
#[derive(Debug)]
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

impl MovePicker<'_> {
    pub fn new<'a>(
        board: &'a Board,
        orderdata: &'a OrderData,
        ply: u8,
        ttmove: ShortMove,
    ) -> MovePicker<'a> {
        MovePicker {
            board,
            orderdata,
            ply,
            curr_idx: 0,
            curr_scorelist: ScoreList::new(0),
            curr_mvlist: MoveList::new(),
            curr_stage: Stage::TTMove,
            ttmove,
            generated_killer_1: 0,
            generated_killer_2: 0,
        }
    }
}

impl Iterator for MovePicker<'_> {
    type Item = Move;

    fn next(&mut self) -> Option<Self::Item> {
        match self.curr_stage {
            Stage::TTMove => {
                self.curr_stage = Stage::WCaptures;
                // we should have already verified that the ttmove is good
                Some(self.ttmove.to_longmove(self.board))
            }
            Stage::WCaptures => {
                const WORST_GOOD_SCORE: i16 = 0;
                let mut maxscore = i16::MIN;
                let mut bestidx = self.curr_idx;
                let mut second_best_score = i16::MIN;
                if self.curr_mvlist.length == 0 {
                    
                    self.curr_mvlist = self.board.generate_moves(false, true);
                    self.curr_scorelist.length = self.curr_mvlist.length;
                    for i in 0..(self.curr_mvlist.length as usize) {
                        let action = self.curr_mvlist[i as usize];
                        let value = self.board.capture_order(action);
                        self.curr_scorelist[i as usize] = value;
                        if value > maxscore {
                            second_best_score = maxscore;
                            maxscore = value;
                            bestidx = i;
                        } else if value > second_best_score {
                            second_best_score = value;
                        }
                    }
                } else {
                    
                    for i in self.curr_idx..(self.curr_scorelist.length as usize) {
                        if self.curr_scorelist[i] > maxscore {

                            second_best_score = maxscore;
                            maxscore = self.curr_scorelist[i];
                            bestidx = i;
                        } else if self.curr_scorelist[i] > second_best_score {
                            second_best_score = self.curr_scorelist[i]; // we want to store the second best score as well
                        }
                    }
                }
                
                self.curr_scorelist.swap(self.curr_idx, bestidx);
                self.curr_mvlist.swap(self.curr_idx, bestidx);
                let bestmove = self.curr_mvlist[self.curr_idx];
                self.curr_idx += 1;
                if bestmove as u16 == self.ttmove {
                    // avoid generating moves we have already checked
                    return self.next();
                }

                if second_best_score < WORST_GOOD_SCORE
                {
                    // we've seen all good captures.
                    // the next best score is worse than the worst "good" score, so we go into the next stage.
                    self.curr_stage = Stage::Killers;
                    
                }

                Some(bestmove)
            }
            Stage::Killers => {
                let killers = self.orderdata.killers[self.ply as usize];
                // test k1 and k2
                if self.generated_killer_1 == 0 {
                    self.generated_killer_1 = killers[0];
                    if self.board.check_move_legal(self.generated_killer_1) {
                        Some(self.generated_killer_1.to_longmove(self.board))
                    } else {
                        self.next()
                    }
                } else if self.generated_killer_2 == 0 {
                    self.generated_killer_2 = killers[1];
                    if self.board.check_move_legal(self.generated_killer_2) {
                        Some(self.generated_killer_1.to_longmove(self.board))
                    } else {
                        self.next()
                    }
                } else {
                    self.curr_stage = Stage::LCaptures;
                    self.next()
                }
            },
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
                if bestmove as u16 == self.ttmove {
                    self.next()
                } else {
                    let seescore = self.board.see(bestmove);
                    self.curr_scorelist[self.curr_idx as usize] = seescore;
                    if seescore < maxscore {
                        return self.next();
                    }

                    self.curr_idx += 1;
                    if self.curr_idx as u8 == self.curr_mvlist.length {
                        self.curr_mvlist = MoveList::new();
                        self.curr_scorelist = ScoreList::new(0);
                        self.curr_idx = 0;
                        self.curr_stage = Stage::Quiets;
                        
                    }
                    Some(bestmove)
                }
            }
            Stage::Quiets => {
               
                let mut maxscore = i16::MIN;
                let mut bestidx = self.curr_idx;

                if self.curr_mvlist.length == 0 {
                    self.curr_mvlist = self.board.generate_moves(true, false);
                    self.curr_scorelist.length = self.curr_mvlist.length;
                    for i in 0..(self.curr_mvlist.length as usize) {
                        let action = self.curr_mvlist[i as usize];
                        // assign history score
                        let piecemoved = action.piece_moved(self.board);
                        let squareto = action.move_to();
                        let history = self.orderdata.get_history(self.board.tomove, piecemoved, squareto);
                        self.curr_scorelist[i as usize] = history;
                        if history > maxscore {
                            maxscore = history;
                            bestidx = i;
                        }
                    }
                } else {
                    // resume prev movelist
                    for i in self.curr_idx..(self.curr_scorelist.length as usize) {
                        if self.curr_scorelist[i] > maxscore {
                            maxscore = self.curr_scorelist[i];
                            bestidx = i;
                        }
                    }
                }
                if self.curr_scorelist.length as usize == self.curr_idx {
                    return None;
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
                Some(bestmove)
            }
        }

    }
}

impl Board {
    pub fn check_move_legal(&self, action: ShortMove) -> bool {
        if self.check_pseudolegal(action) {
            let movingpiece = action.piece_moved(self);
            if movingpiece == KING {
                return true;
            } else if action.move_type() == PASSANT {
                let newb = self.do_move(action);
                return !newb.incheck(self.tomove);
            } else {
                let legalsquares = self.check_for_legalmoves();
                let bpinsquares = self.generate_bishop_pins();
                let rpinsquares = self.generate_rook_pins();

                let tobb = Bitboard::new(action.move_to());
                let frombb = Bitboard::new(action.move_from());
                if tobb & legalsquares != 0 {
                    if bpinsquares & frombb != 0 {
                        return tobb & bpinsquares != 0
                            && (movingpiece == BISHOP
                                || movingpiece == QUEEN
                                || movingpiece == PAWN);
                    } else if rpinsquares & frombb != 0 {
                        return tobb & rpinsquares != 0
                            && (movingpiece == ROOK
                                || movingpiece == QUEEN
                                || movingpiece == PAWN);
                    } else {
                        return true;
                    }
                }
            }
        }
        false
    }

    #[inline]
    fn check_pseudolegal(&self, action: ShortMove) -> bool {
        if action == 0 {
            return false;
        }
        let piecemoved = self.get_at_square(action.move_from());
        let is_correct_color = self.is_color(action.move_from(), self.tomove);
        let legal_destination = !self.is_color(action.move_to(), self.tomove);
        if is_correct_color && piecemoved != NOPIECE && legal_destination {
            match piecemoved {
                PAWN => {
                    if self.is_empty(action.move_to()) {
                        if action.move_type() == PASSANT {
                            return self.passant_square.is_some_and(|&x| x == action.move_to());
                        } else {
                            let diff = action.move_from() as i8 - action.move_to() as i8;
                            const DIFFS: [i8; 2] = [-8, 8];
                            const FOURTHS: [u8; 2] = [3, 4];
                            let doublepush = DIFFS[self.tomove as usize] * 2 == diff;
                            return (doublepush && action.move_to() >> 3 == FOURTHS[self.tomove as usize])
                                || DIFFS[self.tomove as usize] == diff;
                        }
                    } else if self.is_color(action.move_to(), !self.tomove) {
                        let captures =
                            PAWN_CAPTURES[self.tomove as usize][action.move_from() as usize];
                        let newbb = Bitboard::new(action.move_to());
                        return captures & newbb != 0;
                    }
                }
                KNIGHT | BISHOP | ROOK | QUEEN => {
                    return Self::attackable(
                        action.move_from(),
                        action.move_to(),
                        self.get_occupancy(),
                    );
                }

                KING => {
                    let occ = self.get_occupancy() ^ self.get_pieces(KING, self.tomove);
                    let atkmask = self.generate_atk_mask(!self.tomove, occ);

                    if action.move_type() == CASTLE {
                        let (relevant_castleright, blockmask, checkmask) = match action.move_to() {
                            G1 => (self.castling_rights >> 3, 0x60, 0x70),
                            C1 => ((self.castling_rights >> 2) & 1, 0xe, 0x1c),
                            G8 => (
                                (self.castling_rights >> 1) & 1,
                                0x6000000000000000,
                                0x7000000000000000,
                            ),
                            C8 => (
                                (self.castling_rights) & 1,
                                0xe00000000000000,
                                0x1c00000000000000,
                            ),
                            _ => panic!("Bad castle"),
                        };

                        return relevant_castleright != 0
                            && blockmask & occ == 0
                            && atkmask & checkmask == 0;
                    } else {
                        let bbmove = Bitboard::new(action.move_to());
                        let atks = KING_ATTACKS[action.move_from() as usize] & !atkmask;
                        return atks & bbmove != 0;
                    }
                }
                _ => return false,
            }
        }
        false
    }
    fn attackable(from: Square, to: Square, occupancy: Bitboard) -> bool {
        INBETWEENS[from as usize][to as usize] & occupancy == 0
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
           victimval - attackerval
        } else {
            victimval - attackerval / 8
        }
    }
}
