use std::{cmp, time::Instant};

use crate::{
    board_state::board::Board,
    move_generation::{
        action::{Action, Move},
        list::List,
    },
};

use super::{
    moveorder::{MovePicker, OrderData},
    transposition::{TranspositionTable, BETA, EXACT},
};
const CHECKMATE: i16 = 10_000;

pub struct SearchControl {
    pub searchdata: SearchData,
    pub curr_ply: u16,
    pub curr_board: Board,
}

impl SearchControl {
    pub fn go_search(&mut self) {
        let timer = Instant::now();
        self.searchdata.age_history();
        let mut pv = List::new();
        let mut depth = 0;
        let mut alpha = -CHECKMATE;
        let mut beta = CHECKMATE;
        loop {
            depth += 1;
            pv.clear();
            let score = self.curr_board.negamax::<true>(
                depth,
                alpha,
                beta,
                &mut self.searchdata,
                self.curr_ply,
                &mut pv,
            );

            if score <= alpha || score >= beta {
                beta = CHECKMATE;
                alpha = -CHECKMATE;
                depth -= 1;
                continue;
            }

            alpha = score - 35;
            beta = score + 35;

            let elapsed = timer.elapsed().as_millis() as u64;

            let nps = self.searchdata.nodecount * 1000 / elapsed;
            print!(
                "info depth {} score {} nodes {} nps {} time {} pv",
                depth, score, self.searchdata.nodecount, nps, elapsed
            );
            for i in 0..pv.length {
                print!(" {}", pv[i as usize].to_algebraic());
            }
            println!();
            if elapsed >= 6000 {
                break;
            }
        }
        println!("bestmove {}", pv[0].to_algebraic());
    }
}
impl Board {
    pub fn negamax<const ISROOT: bool>(
        &self,
        depth: u8,
        mut alpha: i16,
        beta: i16,
        data: &mut SearchData,
        ply: u16,
        pvline: &mut List<Move>,
    ) -> i16 {
        if self.is_draw() {
            return 0;
        }

        if depth == 0 {
            // If we are at zero depth, Q-search will allow us to evaluate a quiet position for accurate results
            return self.quiesce(alpha, beta, 0, data);
        }

        data.nodecount += 1;
        let mut bestmove = 0;
        let ispv = beta - alpha != 1;
        let tt_entry = data.tt.probe(self.zobrist_key);
        let tt_data = unsafe { *tt_entry };
        if tt_data.key_equals(self.zobrist_key) {
            // set our 'best guess' to be the tt move
            bestmove = tt_data.bestmove;
            let score = tt_data.score;
            // if we can use the score, then return that.
            if !ISROOT
                && tt_data.get_depth() >= depth
                && (tt_data.get_nodetype() == EXACT
                    || (tt_data.get_nodetype() == BETA && score >= beta))
            {
                return score;
            }
        } else if depth >= 4 {
            // If there is no TT move, it's faster to do a shorter search and use that as the best move instead
            let mut newpvline = List::new();
            self.negamax::<false>(depth - 2, alpha, beta, data, ply, &mut newpvline);
            bestmove = newpvline[0] as u16;
        }

        let moves_generated = MovePicker::new(self, bestmove, &data.ord, ply);
        let mut best_pvline = List::new();
        let mut best_score = -CHECKMATE;

        let mut stored_move = false;
        let mut num_moves = 0;
        for action in moves_generated {
            // Initialize a child PV line
            let mut newpvline = List::new();
            newpvline.push(action);
            let mut score: i16;

            let newb = self.do_move(action);
            // Search with a full window if we are in a pv node and this is the first move, or the depth is low
            if (ispv && num_moves == 0) || (depth <= 3) {
                score =
                    -newb.negamax::<false>(depth - 1, -beta, -alpha, data, ply + 1, &mut newpvline);
            } else {
                // Otherwise, search with a null window to maximize cutoffs
                score = -newb.negamax::<false>(
                    depth - 1,
                    -alpha - 1,
                    -alpha,
                    data,
                    ply + 1,
                    &mut newpvline,
                );
                if score > alpha && score < beta {
                    // If the score is within the bounds then we have to do a full window re-search to get the true score
                    score = -newb.negamax::<false>(
                        depth - 1,
                        -beta,
                        -alpha,
                        data,
                        ply + 1,
                        &mut newpvline,
                    );
                }
            }

            if score > best_score {
                best_score = score;
                best_pvline = newpvline;
                bestmove = action as u16;
                alpha = cmp::max(alpha, score);
                if score >= beta {
                    // The score is too good, and will be avoided. Stop searching
                    unsafe {
                        tt_entry.as_mut().unwrap().store(
                            self.zobrist_key,
                            action as u16,
                            score,
                            depth,
                            BETA,
                        );
                    }
                    if !action.is_capture() && num_moves == 0 {
                        data.ord.update_history(action, depth, self);
                        data.ord.update_killer(action, ply);
                    }
                    stored_move = true;
                    num_moves += 1;
                    break;
                }
            }

            num_moves += 1;
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
        if num_moves == 0 {
            if self.incheck(self.tomove) {
                return -CHECKMATE;
            } else {
                return 0;
            }
        }
        for i in 0..best_pvline.length {
            pvline.push(best_pvline[i as usize]);
        }

        best_score
    }

    pub fn quiesce(&self, mut alpha: i16, beta: i16, ply: u16, data: &mut SearchData) -> i16 {
        data.qnodecount += 1;
        data.nodecount += 1;

        let bestscore = self.evaluate();

        if bestscore >= beta {
            return bestscore;
        }
        if bestscore > alpha {
            alpha = bestscore;
        }

        let captures = MovePicker::new_capturepicker(self, &data.ord);
        for action in captures {
            let seevalue = self.see(action);
            if seevalue + 200 + bestscore < alpha {
                continue;
            }
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

pub struct SearchData {
    pub bestmove: Move,
    pub nodecount: u64,
    pub qnodecount: u64,
    pub tt: TranspositionTable,
    pub ord: OrderData,
}

impl Default for SearchData {
    fn default() -> Self {
        Self::new()
    }
}
impl SearchData {
    pub fn new() -> Self {
        let default_ord = OrderData {
            killers: [[0; 2]; 256],
            history: [[[0; 64]; 6]; 2],
        };
        let tt = TranspositionTable::new(32);
        SearchData {
            bestmove: 0,
            nodecount: 0,
            qnodecount: 0,
            tt,
            ord: default_ord,
        }
    }

    fn age_history(&mut self) {
        self.ord.age_history()
    }
}
