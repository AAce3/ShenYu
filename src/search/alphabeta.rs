use std::{
    cmp,
    sync::mpsc::{Receiver, TryRecvError},
    time::Instant,
};

use crate::{
    board_state::board::Board,
    move_generation::{
        action::{Action, Move},
        list::List,
    },
    uci::Control,
};

use super::{
    moveorder::{MovePicker, OrderData},
    timer::Timer,
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
        let global_time = Instant::now();
        self.refresh();
        let mut bestmove = 0;
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
                self.curr_ply as u8,
                &mut pv,
            );
            if self.searchdata.timer.stopped {
                break;
            }
            if score <= alpha || score >= beta {
                beta = CHECKMATE;
                alpha = -CHECKMATE;
                depth -= 1;
                continue;
            }
            bestmove = pv[0];
            alpha = cmp::max(score - 35, -CHECKMATE);
            beta = cmp::min(score + 35, CHECKMATE);
            let mut scoretype = "score cp";
            let mut reported_score = score;
            if mated_in(score) < 64 && mated_in(score) > -64 {
                scoretype = "mate";
                reported_score = mated_in(score).div_ceil(2)
            }
            let elapsed = global_time.elapsed().as_millis() as u64;

            let nps = if elapsed == 0 {
                0
            } else {
                self.searchdata.nodecount * 1000 / elapsed
            };

            print!(
                "info depth {} {} {} nodes {} nps {} time {} pv",
                depth, scoretype, reported_score, self.searchdata.nodecount, nps, elapsed
            );
            for i in 0..pv.length {
                print!(" {}", pv[i as usize].to_algebraic());
            }
            println!();
            if depth >= self.searchdata.timer.maxdepth {
                break;
            }
        }
        println!("bestmove {}", bestmove.to_algebraic());
    }

    pub fn reset(&mut self) {
        self.curr_board =
            Board::parse_fen("rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1").unwrap();
        self.searchdata.clear();
        self.curr_ply = 0;
    }

    pub fn refresh(&mut self) {
        self.searchdata.age_history();
        self.searchdata.nodecount = 0;
        self.searchdata.qnodecount = 0;
       
    }

    pub fn get_recv(&self) -> Option<Control> {
        match self.searchdata.message_recv.as_ref().unwrap().try_recv() {
            Ok(key) => Some(key),
            Err(TryRecvError::Empty) => None,
            Err(TryRecvError::Disconnected) => panic!("Channel disconnected"),
        }
    }
}

impl Board {
    #[allow(clippy::too_many_arguments)]
    pub fn negamax<const ISROOT: bool>(
        &self,
        mut depth: u8,
        mut alpha: i16,
        beta: i16,
        data: &mut SearchData,
        ply: u16,
        starting_ply: u8,
        pvline: &mut List<Move>,
    ) -> i16 {
        if self.is_draw() {
            return 0;
        }
        let incheck = self.incheck(self.tomove);
        if incheck {
            depth += 1;
        }
        data.nodecount += 1;
        if data.nodecount >= data.timer.max_nodes {
            data.timer.stopped = true;
        }
        if data.timer.is_timed && data.nodecount % 4096 == 0 {
            let has_msg = match data.message_recv.as_ref().unwrap().try_recv() {
                Ok(key) => key == Control::Stop,
                Err(TryRecvError::Empty) => false,
                Err(TryRecvError::Disconnected) => panic!("Channel disconnected"),
            };
            data.timer.stopped = data.timer.check_time() || has_msg || data.timer.stopped;
        }

        if data.timer.stopped {
            return 0;
        }

        if depth == 0 {
            // If we are at zero depth, Q-search will allow us to evaluate a quiet position for accurate results
            data.nodecount -= 1;
            return self.quiesce(alpha, beta, 0, data);
        }

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
                if mated_in(score) < 64 && mated_in(score) > -64 {
                    if score.is_positive() {
                        return score - (ply as i16 - starting_ply as i16);
                    } else {
                        return score + (ply as i16 - starting_ply as i16);
                    }
                }
                return score;
            }
        } else if depth >= 4 {
            // If there is no TT move, it's faster to do a shorter search and use that as the best move instead
            let mut newpvline = List::new();
            self.negamax::<false>(
                depth - 2,
                alpha,
                beta,
                data,
                ply,
                starting_ply,
                &mut newpvline,
            );
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
                score = -newb.negamax::<false>(
                    depth - 1,
                    -beta,
                    -alpha,
                    data,
                    ply + 1,
                    starting_ply,
                    &mut newpvline,
                );
            } else {
                // Otherwise, search with a null window to maximize cutoffs
                score = -newb.negamax::<false>(
                    depth - 1,
                    -alpha - 1,
                    -alpha,
                    data,
                    ply + 1,
                    starting_ply,
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
                        starting_ply,
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
            if incheck {
                return mate_score(ply, starting_ply);
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
        if data.nodecount >= data.timer.max_nodes {
            data.timer.stopped = true;
        }
        if data.timer.is_timed && data.nodecount % 4096 == 0 {
            data.timer.stopped = data.timer.check_time() || data.timer.stopped;
        }
        if data.timer.stopped {
            return 0;
        }
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
    pub nodecount: u64,
    pub qnodecount: u64,
    pub tt: TranspositionTable,
    pub ord: OrderData,
    pub timer: Timer,
    pub message_recv: Option<Receiver<Control>>,
}

impl SearchData {
    pub fn new(t: Timer) -> Self {
        let default_ord = OrderData {
            killers: [[0; 2]; 256],
            history: [[[0; 64]; 6]; 2],
        };
        let tt = TranspositionTable::new(32);
        SearchData {
            nodecount: 0,
            qnodecount: 0,
            tt,
            ord: default_ord,
            timer: t,
            message_recv: None,
        }
    }
    fn clear(&mut self) {
        self.tt.clear();
        self.ord.clear();
        self.nodecount = 0;
        self.qnodecount = 0;
    }
    fn age_history(&mut self) {
        self.ord.age_history()
    }
}

const fn mate_score(ply: u16, starting_ply: u8) -> i16 {
    -CHECKMATE + (ply as i16 - starting_ply as i16)
}
const fn mated_in(score: i16) -> i16 {
    if score.is_positive() {
        CHECKMATE - score
    } else {
        -CHECKMATE - score
    }
}
