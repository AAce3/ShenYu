use std::cmp;

use crate::{
    board_state::board::Board,
    move_generation::action::{Action, Move},
    search::{
        moveorder::MovePicker,
        transposition::{Entry, BETA, EXACT},
    },
};

use super::{moveorder::OrderData, transposition::TranspositionTable};

impl Board {
    pub fn alphabeta_search<
        const ISPV: bool,
        const IS_ROOTNODE: bool,
        /*
        const razor: bool,
        const lmr: bool,
        const nullmove: bool,*/
    >(
        &self,
        mut alpha: i16,
        beta: i16,
        depth: u8,
        ply: u16,
        search: &mut SearchControl,
        //searchcontrol
    ) -> i16 {
        if depth == 0 {
            return self.quiesce(alpha, beta);
        }
        let mut bestmove;
        let entry = search.tt.probe(self.zobrist_key);
        let data = unsafe { *entry };
        {
            bestmove = data.bestmove;
            if data.key_equals(self.zobrist_key) && self.check_move_legal(bestmove) {
                todo!("check node flags and depth")
            } else {
                // iid
                let mut newsearch = SearchControl {
                    tt: search.tt,
                    orderdata: search.orderdata,
                    bestmove: 0,
                };
                let _i = self.alphabeta_search::<true, true>(
                    alpha,
                    beta,
                    depth - 2,
                    ply,
                    &mut newsearch,
                );
                bestmove = newsearch.bestmove as u16;
            }
        }

        let mut movepicker = MovePicker::new(self, ply, bestmove);

        let mut bestscore = i16::MIN;
        let mut firstmove = true;

        while let Some(action) = movepicker.next(search.orderdata) {
            let newb = self.do_move(action);
            let mut score: i16;
            if ISPV && firstmove {
                // pv search
                score = -newb.alphabeta_search::<ISPV, false>(
                    -beta,
                    -alpha,
                    depth - 1,
                    ply + 1,
                    search,
                );
                firstmove = false;
            } else {
                score = -newb.alphabeta_search::<false, false>(
                    -alpha - 1,
                    -alpha,
                    depth - 1,
                    ply + 1,
                    search,
                ); // zero window search
                if score > alpha && score < beta {
                    // re-search, our prediction that it will fail outside the window failed
                    score = -newb.alphabeta_search::<ISPV, false>(
                        -beta,
                        -alpha,
                        depth - 1,
                        ply + 1,
                        search,
                    )
                }
            }
            bestscore = cmp::max(score, bestscore);
            alpha = cmp::max(score, alpha);
            
            if IS_ROOTNODE && bestscore == score {
                search.bestmove = action;
            }

            if bestscore >= beta {
                unsafe {
                    *entry = Entry::create(self.zobrist_key, action as u16, bestscore, depth, BETA)
                };
                if !action.is_capture(self) {
                    search.orderdata.update_killer(ply, action);
                    search.orderdata.increment_history(
                        self.tomove,
                        action.piece_moved(self),
                        action.move_to(),
                        depth,
                    );
                }
                break;
            } else {
                unsafe {
                    *entry = Entry::create(self.zobrist_key, action as u16, bestscore, depth, EXACT)
                };
                if score < alpha {
                    // decrement history
                }
            }
        }

        return bestscore;

        todo!()
    }

    pub fn quiesce(&self, alpha: i16, beta: i16) -> i16 {
        todo!()
    }
}

pub struct SearchControl<'a> {
    pub tt: &'a mut TranspositionTable,
    pub orderdata: &'a mut OrderData,
    pub bestmove: Move,
}
