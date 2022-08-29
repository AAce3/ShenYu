use crate::{board_state::board::Board, search::moveorder::MovePicker, move_generation::action};

use super::{transposition::{TranspositionTable}, moveorder::OrderData};


impl Board {
    pub fn alphabeta_search<
        const ISPV: bool,
        /*
        const razor: bool,
        const lmr: bool,
        const nullmove: bool,*/
    >(
        &self,
        alpha: i16,
        beta: i16,
        depth: u8,
        ply: u16,
        search: &mut SearchControl,
        //searchcontrol
    ) -> i16 {

        if depth == 0 {
            return self.quiesce(alpha, beta)
        }

        todo!("Check draw and win conditions");
        
        let entry = search.tt.probe(self.zobrist_key);
        let mut bestmove = entry.bestmove;
        if entry.key_equals(self.zobrist_key) && self.check_move_legal(bestmove){
            todo!("check node flags and depth")
        } else {
            todo!("Internal Iterative Deepening because we don't have TT")
        }
        
        let movepicker = MovePicker::new(self, &search.ordering_data, ply, bestmove);
        for action in movepicker{
            if ISPV{
                
            }
        }
        todo!()
    }

    pub fn quiesce(&self, alpha: i16, beta: i16) -> i16 {
        todo!()
    }
}

pub struct SearchControl{
    pub tt: TranspositionTable,
    pub ordering_data: OrderData,
}
