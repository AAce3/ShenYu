use crossbeam::channel::TryRecvError;

use crate::{
    movegen::{
        action::Action,
        genmoves::GenType,
        movelist::{List, MoveList},
    },
    search::{
        moveorder::{QSearchGenerator, Stage, StagedGenerator},
        searchcontrol::PVLine,
    },
};

use super::{
    hashtable::{ALPHA, BETA, EXACT},
    searchcontrol::{Searcher, CHECKMATE},
};

pub const MAX_DEPTH: usize = 64;
impl Searcher {
    pub fn alphabeta<const IS_ROOT: bool>(
        &mut self,
        mut depth: i16,
        ply: u16,
        mut alpha: i16,
        beta: i16,
        pvline: &mut List<Action, 64>,
    ) -> i16 {
        let count = if IS_ROOT { 2 } else { 1 };

        if depth > 1 && (self.board.is_draw() || self.board.is_repetition(count)) {
            if IS_ROOT {
                let mut movelist = MoveList::new();
                self.board.genmoves::<{ GenType::ALL }>(&mut movelist);
                if movelist.len() == 0 {
                    panic!("No legal moves!")
                } else {
                    pvline.push(*movelist[0])
                }
            }
            return 0;
        }

        self.nodecount += 1;
        if self.nodecount >= self.timer.max_nodes {
            self.timer.stopped = true;
        }

        if check_for_stop(self.nodecount) {
            let has_msg = match self.stop.try_recv() {
                Ok(value) => value,
                Err(TryRecvError::Empty) => false,
                Err(TryRecvError::Disconnected) => panic!("Channel disconnected!"),
            };
            self.timer.stopped |= has_msg;
        }

        if self.timer.is_timed && check_for_stop(self.nodecount) {
            self.timer.stopped |= self.timer.check_time();
        }

        if self.timer.stopped {
            return 0;
        }

        let in_check = self.board.in_check(self.board.active_color());
        if in_check {
            depth += 1;
        }

        let mut best_move = Action::default();

        let is_pv = beta - alpha != 1;
        let zobrist_key = self.board.zobrist();
        let tt_entry = self.tt.probe(zobrist_key);
        let tt_data = unsafe { *tt_entry };

        if tt_data.key_equals(zobrist_key) {
            best_move = tt_data.bestmove;
            let score = tt_data.score;
            let shoulduse = match tt_data.get_nodetype() {
                EXACT => true,
                ALPHA => score <= alpha,
                BETA => score > beta,
                _ => false,
            };

            // if we can use the score, then return that.
            if !IS_ROOT && tt_data.get_depth() as i16 >= depth && shoulduse {
                if best_move != Action::default() {
                    pvline.push(best_move);
                }
                // transpositions that have a mate score should be handled differently
                if is_mate(score) {
                    if score.is_positive() {
                        return score - (ply as i16);
                    } else {
                        return score + (ply as i16);
                    }
                }
                return score;
            }
        }

        if depth <= 0 {
            self.nodecount -= 1;
            let qvalue = self.quiesce(0, alpha, beta);
            if self.timer.stopped {
                return 0;
            } else {
                return qvalue;
            }
        }

        let eval = self.board.evaluate();

        // Null move pruning
        if !in_check && !is_pv && eval >= beta && !IS_ROOT && !self.board.is_kp() {
            self.board.make_nullmove();
            let reduction = 3 + depth / 6;
            let mut new_pvline = PVLine::new();
            let score = -self.alphabeta::<false>(
                depth - 1 - reduction,
                ply + 1,
                -beta,
                -beta + 1,
                &mut new_pvline,
            );

            self.board.unmake_nullmove();

            if self.timer.stopped {
                return 0;
            }

            if score >= beta {
                return beta;
            }
        }

        const FUTILITY_MARGIN: i16 = 300;
        let can_futility = depth == 1
            && !is_pv
            && !in_check
            && !is_mate(alpha)
            && !is_mate(beta)
            && eval + FUTILITY_MARGIN < alpha;

        let mut best_pvline = PVLine::new();
        let mut best_score = -CHECKMATE;

        let mut stored_move = false;
        let mut num_moves = 0;
        let mut raised_alpha = false;

        let mut generator = StagedGenerator::new(best_move, ply);
        while let Some((action, stage)) = generator.next_move(&self.ord, &mut self.board) {
            let lmp_margin = (depth + 1) * 4;
            if num_moves != 0
                && depth < 5
                && !in_check
                && num_moves >= lmp_margin
                && (stage == Stage::Quiets)
            {
                continue;
            }

            // try to prune futile moves
            if num_moves != 0 && can_futility && (stage == Stage::Quiets) {
                continue;
            }

            self.board.make_move(action);
            if stage == Stage::HashMove || stage == Stage::Killers {
                // make sure that it isn't an illegal move
                if self.board.in_check(!self.board.active_color()) {
                    self.board.unmake_move(action);
                    continue;
                }
            }

            let mut new_pv_line = PVLine::new();
            new_pv_line.push(action);
            num_moves += 1;

            let mut score: i16;

            // Search with a full window if we are in a pv node and this is the first move, or the depth is low
            if is_pv && num_moves == 1 {
                score =
                    -self.alphabeta::<false>(depth - 1, ply + 1, -beta, -alpha, &mut new_pv_line);
            } else {
                // Try to prune late moves.

                // Try to reduce
                let can_lmr =
                    !is_pv && !in_check && num_moves >= 4 && (stage == Stage::Quiets) && depth >= 3;

                let reduction = if can_lmr { 2 } else { 0 };

                score = -self.alphabeta::<false>(
                    depth - reduction - 1,
                    ply + 1,
                    -alpha - 1,
                    -alpha,
                    &mut new_pv_line,
                );

                if score > alpha {
                    new_pv_line.clear();
                    new_pv_line.push(action);
                    // If the score is within the bounds then we have to do a full window re-search to get the true score
                    score = -self.alphabeta::<false>(
                        depth - 1,
                        ply + 1,
                        -beta,
                        -alpha,
                        &mut new_pv_line,
                    );
                }
            }

            self.board.unmake_move(action);

            if self.timer.stopped {
                return 0;
            }

            if score > best_score {
                best_score = score;
                best_pvline = new_pv_line;
                best_move = action;
                if score > alpha {
                    raised_alpha = true;
                    alpha = score;
                }

                if score >= beta {
                    unsafe {
                        tt_entry.as_mut().unwrap().store(
                            zobrist_key,
                            action,
                            score,
                            depth as u8,
                            BETA,
                        );
                    }

                    if stage == Stage::Quiets || stage == Stage::Killers {
                        self.ord.update_history(action, depth as u8, &self.board);
                        self.ord.update_killer(action, ply);
                    }
                    stored_move = true;

                    break;
                }
            }
        }

        if num_moves == 0 {
            if in_check {
                best_score = mate_score(ply, 0);
            } else {
                best_score = 0;
            }
        }

        // nodetype for mate/stalemate
        let mut nodetype = if raised_alpha { EXACT } else { ALPHA };

        if best_score > alpha {
            nodetype = ALPHA;
            if best_score >= beta {
                nodetype = BETA;
            }
        }

        if !stored_move {
            unsafe {
                tt_entry.as_mut().unwrap().store(
                    zobrist_key,
                    best_move,
                    best_score,
                    depth as u8,
                    nodetype,
                );
            }
        }

        for value in best_pvline.iter() {
            pvline.push(*value);
        }

        best_score
    }

    fn quiesce(&mut self, ply: u16, mut alpha: i16, beta: i16) -> i16 {
        self.nodecount += 1;
        self.qnodecount += 1;
        if self.nodecount >= self.timer.max_nodes {
            self.timer.stopped = true;
        }

        if check_for_stop(self.nodecount) {
            let has_msg = match self.stop.try_recv() {
                Ok(value) => value,
                Err(TryRecvError::Empty) => false,
                Err(TryRecvError::Disconnected) => panic!("Channel disconnected!"),
            };
            self.timer.stopped |= has_msg;
        }

        if self.timer.is_timed && check_for_stop(self.nodecount) {
            self.timer.stopped |= self.timer.check_time()
        }

        if self.timer.stopped {
            return 0;
        }
        let bestscore = self.board.evaluate();

        if bestscore >= beta {
            return bestscore;
        }

        if bestscore > alpha {
            alpha = bestscore;
        }

        let captures = QSearchGenerator::new(&mut self.board);
        for action in captures {
            let seevalue = self.board.see(action);
            if seevalue + 200 < alpha || seevalue < 0 {
                continue;
            }
            self.board.make_move(action);

            let score = -self.quiesce(ply + 1, -beta, -alpha);
            self.board.unmake_move(action);
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

const fn check_for_stop(nodecount: u64) -> bool {
    const CHECK_TIME: u64 = 4096;
    nodecount % CHECK_TIME == 0
}

const fn is_mate(score: i16) -> bool {
    mated_in(score) < MAX_DEPTH as i16 && mated_in(score) > -(MAX_DEPTH as i16)
}

pub(super) const fn mate_score(ply: u16, starting_ply: u8) -> i16 {
    -CHECKMATE + (ply as i16 - starting_ply as i16)
}

pub(super) const fn mated_in(score: i16) -> i16 {
    if score.is_positive() {
        CHECKMATE - score
    } else {
        -CHECKMATE - score
    }
}
