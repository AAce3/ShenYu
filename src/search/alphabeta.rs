use crate::{
    board_state::board::Board,
    move_generation::{
        action::{Move, Action},
        list::List,
    },
};

use super::{
    moveorder::{CaptureSorter, History, Killer, MoveSorter},
    transposition::{TranspositionTable, BETA, EXACT},
};
const CHECKMATE: i16 = 10_000;
impl Board {
    pub fn negamax(
        &self,
        depth: u8,
        mut alpha: i16,
        beta: i16,
        data: &mut SearchData,
        ply: u16,
        pvline: &mut List<Move>,
    ) -> i16 {
        if depth == 0 {
            return self.quiesce(alpha, beta, 0, data);
        }

        let mut bestmove = 0;

        let tt_entry = data.tt.probe(self.zobrist_key);
        let tt_data = unsafe { *tt_entry };
        if tt_data.key_equals(self.zobrist_key) {
            bestmove = tt_data.bestmove;
            let score = tt_data.score;
            if tt_data.get_depth() >= depth
                && (tt_data.get_nodetype() == EXACT
                    || (tt_data.get_nodetype() == BETA && score >= beta))
            {
                return score;
            }
        } else if depth >= 4 {
            // IID
            let mut newpvline = List::new();
            self.negamax(depth - 2, alpha, beta, data, ply, &mut newpvline);
            bestmove = newpvline[0] as u16;
        }
        data.nodecount += 1;
        let mut moves_generated = MoveSorter::new(self, bestmove);

        let mut best_pvline = List::new();
        let mut best_score = -CHECKMATE;

        let mut stored_move = false;

        while let Some(action) = // fetches the best action from move sorting
            moves_generated.next(&data.history, &data.killers.table[ply as usize])
        {
            let mut newpvline = List::new();
            newpvline.push(action);

            let newb = self.do_move(action);
            let score = -newb.negamax(depth - 1, -beta, -alpha, data, ply + 1, &mut newpvline);

            if score > best_score {
                best_score = score;
                best_pvline = newpvline;
                bestmove = action as u16;
                if score > alpha {
                    alpha = score;
                }
                if score >= beta {
                    if !action.is_capture(self) {
                        data.history.increment_history(
                            self.tomove,
                            action.piece_moved(self),
                           action.move_to(),
                            depth,
                        );
                        data.killers.update_killer(ply, action);
                    }
                    unsafe {
                        tt_entry.as_mut().unwrap().store(
                            self.zobrist_key,
                            action as u16,
                            score,
                            depth,
                            BETA,
                        );
                    }
                    stored_move = true;
                    break;
                }
            }
        }
        if !stored_move {
            unsafe {
                tt_entry.as_mut().unwrap().store(
                    self.zobrist_key,
                    bestmove,
                    best_score,
                    depth,
                    EXACT,
                );
            }
        }
        for i in 0..best_pvline.length {
            pvline.push(best_pvline[i as usize]);
        }

        best_score
    }

    pub fn quiesce(&self, mut alpha: i16, beta: i16, ply: u16, data: &mut SearchData) -> i16 {
        data.nodecount += 1;
        let bestscore = self.evaluate();
        if ply > 6 {
            return bestscore;
        }
        if bestscore >= beta {
            return bestscore;
        }
        if bestscore > alpha {
            alpha = bestscore;
        }

        let mut captures = CaptureSorter::new(self);
        while let Some(action) = captures.next(100, alpha, true) {
            let newb = self.do_move(action);
            let score = -newb.quiesce(-beta, -alpha, ply + 1, data);
            if score >= beta {
                return score;
            }

            if score > alpha {
                alpha = score
            }
        }
        alpha
    }
}

pub struct SearchData<'a> {
    pub killers: Killer,
    pub history: History,
    pub bestmove: Move,
    pub nodecount: u64,
    pub qnodecount: u64,
    pub tt: &'a mut TranspositionTable,
}
