use crate::{board_state::board::Board, move_generation::action::{Move, Action}};

use super::moveorder::{Killer, History, MoveSorter};

impl Board {
    pub fn alphabeta<const ISROOT: bool>(
        &self,
        mut alpha: i16,
        beta: i16,
        depth: u8,
        data: &mut SearchData,
        ply: u16
    ) -> i16 {
        if depth == 0 {
            self.quiesce(alpha, beta, data)
        } else {
            data.nodecount += 1;
            let mut moves_generated = MoveSorter::new(self, 0);

            let mut bestscore = i16::MIN;
            while let Some(action) = moves_generated.next(&data.history, &data.killers.table[ply as usize]){

                let newb = self.do_move(action);

                let score = -newb.alphabeta::<false>(-beta, -alpha, depth - 1, data, ply + 1);
                if score >= beta {
                    if !action.is_capture(self){
                        data.history.increment_history(self.tomove, action.piece_moved(self), action.move_to(), depth);
                        data.killers.update_killer(ply, action);
                    }
                    return score;
                } 
                if score > bestscore{
                    bestscore = score;
                }
                if score > alpha {
                    alpha = score;
                    if ISROOT{
                        data.bestmove = action;
                    }
                } 
            }
            alpha
        }
    }

    pub fn quiesce(&self, mut alpha: i16, beta: i16, data: &mut SearchData) -> i16{
        data.nodecount += 1;
        let mut bestscore = self.evaluator.evaluate();
        if bestscore >= beta{
            return bestscore;
        } 
        if bestscore > alpha{
            alpha = bestscore;
        }
        let moves = self.generate_moves::<false, true>();
        
        for i in 0..moves.length{
            let action = moves[i as usize];
            let newb = self.do_move(action);
            let score = -newb.quiesce(-beta, -alpha, data);
            if score >= beta{
                return score;
            }
            if score > bestscore{
                bestscore = score
            }
            if score > alpha{
                alpha = score
            }
        }
        bestscore
    }
}

pub struct SearchData{
    pub killers: Killer,
    pub history: History,
    pub bestmove: Move,
    pub nodecount: u64
}