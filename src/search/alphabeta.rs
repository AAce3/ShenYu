use std::{
    fmt::Write,
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
    gamehistory::GameHistory,
    moveorder::{CapturePicker, OrderData, StagedGenerator},
    timer::Timer,
    transposition::{TranspositionTable, ALPHA, BETA, EXACT}, 
};
const CHECKMATE: i16 = 10_000;

pub struct SearchControl {
    pub searchdata: SearchData,
    pub curr_board: Board,
}

impl SearchControl {
    pub fn go_search(&mut self) {
        let global_time = Instant::now();
        self.refresh();
        self.searchdata.timer.start_time = Instant::now();
        let mut bestmove = 0;
        let mut pv = List::new();
        let mut depth = 0;
        let alpha = -CHECKMATE;
        let beta = CHECKMATE;
        loop {
            depth += 1;
            if depth > 64 {
                break;
            }
            pv.clear();
            let score = self.curr_board.negamax::<true>(
                depth,
                alpha,
                beta,
                &mut self.searchdata,
                0,
                &mut pv,
            );
            if self.searchdata.timer.stopped {
                break;
            }

            let mut scoretype = "cp";
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

            println!(
                "info depth {} score {} {} nodes {} nps {} time {} pv{}",
                depth,
                scoretype,
                reported_score,
                self.searchdata.nodecount,
                nps,
                elapsed,
                format_pv(&pv)
            );
            bestmove = pv[0];
            if depth >= self.searchdata.timer.maxdepth
                || elapsed * 4 > self.searchdata.timer.time_alloted
            {
                break;
            }
        }
        println!("bestmove {}", bestmove.to_algebraic());
        self.searchdata.timer.refresh();
    }

    pub fn reset(&mut self) {
        self.curr_board =
            Board::parse_fen("rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1").unwrap();
        self.searchdata.clear();
        self.searchdata.gamehistory.positions.clear();
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

fn format_pv(pv: &List<Move, 64>) -> String {
    let mut starting_str = String::new();
    for i in 0..pv.length {
        let thing = pv[i as usize].to_algebraic();
        write!(&mut starting_str, " {}", thing).unwrap();
    }
    starting_str
}
impl Board {
    #[allow(clippy::too_many_arguments)]
    pub fn negamax<const ISROOT: bool>(
        &mut self,
        mut depth: u8,
        mut alpha: i16,
        beta: i16,
        data: &mut SearchData,
        ply: u16,
        pvline: &mut List<Move, 64>,
    ) -> i16 {
        if depth > 1 && (self.is_draw() || data.gamehistory.find(self.halfmove_clock)) {
            return 0;
        }

        data.nodecount += 1;
        if data.nodecount >= data.timer.max_nodes {
            data.timer.stopped = true;
        }

        let checknodes = data.nodecount % 4096 == 0;
        if checknodes {
            let has_msg = match data.message_recv.as_ref().unwrap().try_recv() {
                Ok(key) => key == Control::Stop,
                Err(TryRecvError::Empty) => false,
                Err(TryRecvError::Disconnected) => panic!("Channel disconnected"),
            };
            data.timer.stopped = has_msg || data.timer.stopped;
        }
        if data.timer.is_timed && checknodes {
            data.timer.stopped = data.timer.check_time() || data.timer.stopped;
        }

        if data.timer.stopped {
            return 0;
        }
        let incheck = self.incheck(self.tomove);
        if incheck {
            depth += 1;
        }
        if depth == 0 {
            // If we are at zero depth, Q-search will allow us to evaluate a quiet position for accurate results
            data.nodecount -= 1;
            let qvalue = self.quiesce(alpha, beta, 0, data);
            if data.timer.stopped {
                return 0;
            } else {
                return qvalue;
            }
        }

        let mut bestmove = 0;
        let ispv = beta - alpha != 1;
        let tt_entry = data.tt.probe(self.zobrist_key);
        let tt_data = unsafe { *tt_entry };
        if tt_data.key_equals(self.zobrist_key) && self.check_move(tt_data.bestmove) {
            // set our 'best guess' to be the tt move
            bestmove = tt_data.bestmove;
            let score = tt_data.score;
            let shoulduse = match tt_data.get_nodetype() {
                EXACT => true,
                ALPHA => score <= alpha,
                BETA => score > beta,
                _ => false,
            };

            // if we can use the score, then return that.
            if !ISROOT && tt_data.get_depth() >= depth && shoulduse {
                if mated_in(score) < 64 && mated_in(score) > -64 {
                    if score.is_positive() {
                        return score - (ply as i16);
                    } else {
                        return score + (ply as i16);
                    }
                }
                return score;
            }
        } else if depth >= 4 {
            // If there is no TT move, it's faster to do a shorter search and use that as the best move instead
            let mut newpvline = List::new();
            self.negamax::<true>(depth - 2, alpha, beta, data, ply, &mut newpvline);
            if newpvline.length != 0 {
                bestmove = newpvline[0];
            }
        }

        let mut moves_generated = StagedGenerator::new(bestmove, *self, ply);
        let mut best_pvline = List::new();
        let mut best_score = -CHECKMATE;

        let mut stored_move = false;
        let mut num_moves = 0;
        let mut raised_alpha = false;

        while let Some(action) = moves_generated.next(&data.ord) {
            // Initialize a child PV line
            let mut newpvline = List::new();
            newpvline.push(action);
            let mut score: i16;

            let mut newb = self.do_move(action);
            data.gamehistory.store(newb.zobrist_key);
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

                if score >= alpha && score < beta {
                    newpvline.clear();
                    newpvline.push(action);
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
            data.gamehistory.retract();
            if data.timer.stopped {
                return 0;
            }
            if score > best_score {
                best_score = score;
                best_pvline = newpvline;
                bestmove = action;
                if score > alpha {
                    raised_alpha = true;
                    alpha = score;
                }
                if score >= beta {
                    // The score is too good, and will be avoided. Stop searching
                    unsafe {
                        tt_entry.as_mut().unwrap().store(
                            self.zobrist_key,
                            action,
                            score,
                            depth,
                            BETA,
                        );
                    }
                    if !action.is_capture() {
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
        let nodetype = if raised_alpha { EXACT } else { ALPHA };
        if !stored_move {
            unsafe {
                tt_entry.as_mut().unwrap().store(
                    self.zobrist_key,
                    bestmove,
                    best_score,
                    depth,
                    nodetype,
                );
            }
        }
        if num_moves == 0 {
            if incheck {
                return mate_score(ply, 0);
            } else {
                return 0;
            }
        }
        for i in 0..best_pvline.length {
            pvline.push(best_pvline[i as usize]);
        }

        best_score
    }

    pub fn quiesce(&mut self, mut alpha: i16, beta: i16, ply: u16, data: &mut SearchData) -> i16 {
        data.qnodecount += 1;
        data.nodecount += 1;
        if data.nodecount >= data.timer.max_nodes {
            data.timer.stopped = true;
        }
        let checknodes = data.nodecount % 4096 == 0;
        if checknodes {
            let has_msg = match data.message_recv.as_ref().unwrap().try_recv() {
                Ok(key) => key == Control::Stop,
                Err(TryRecvError::Empty) => false,
                Err(TryRecvError::Disconnected) => panic!("Channel disconnected"),
            };
            data.timer.stopped = has_msg || data.timer.stopped;
        }
        if data.timer.is_timed && checknodes {
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

        let captures = CapturePicker::new_capturepicker(self);

        for action in captures {
            let seevalue = self.see(action);
            if seevalue + 200 < alpha || seevalue < 0 {
                continue;
            }
            let mut newb = self.do_move(action);

            let score = -newb.quiesce(-beta, -alpha, ply + 1, data);

            if score > alpha {
                alpha = score;

                if score >= beta {
                    break;
                }
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
    pub gamehistory: GameHistory,
}

impl SearchData {
    pub fn new(t: Timer) -> Self {
        let default_ord = OrderData {
            killers: [[0; 2]; 64],
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
            gamehistory: GameHistory::new(),
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
