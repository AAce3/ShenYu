use std::{time::Instant, fmt::Write};

use crossbeam::channel::Receiver;

use crate::movegen::{action::Action, board::Board, movelist::List};

use super::{
    alphabeta::{mated_in, MAX_DEPTH},
    hashtable::TranspositionTable,
    moveorder::OrderData,
    timer::Timer,
};

pub struct Searcher {
    pub(super) nodecount: u64,
    pub(super) qnodecount: u64,
    pub timer: Timer,
    pub(super) tt: TranspositionTable,
    pub(super) stop: Receiver<bool>,
    pub(super) board: Board,
    pub(super) ord: OrderData,
}


pub(super) const CHECKMATE: i16 = 10_000;

pub(super) type PVLine = List<Action, 64>;
impl Searcher {
    pub fn search(&mut self) {
        let global_time = Instant::now();
        self.refresh();
        self.timer.start_time = Instant::now();
        let mut best_move = Action::default();
        let mut pv = PVLine::new();
        let mut depth = 0;
        let alpha = -CHECKMATE;
        let beta = CHECKMATE;
        loop {
            depth += 1;
            if depth >= MAX_DEPTH as i16 {
                break;
            }

            pv.clear();
            let score = self.alphabeta::<true>(depth , 0, alpha, beta, &mut pv);

            
            if self.timer.stopped {
                break;
            }

            let mut scoretype = "cp";
            let mut reported_score = score;
            if mated_in(score) < 64 && mated_in(score) > -64 {
                scoretype = "mate";
                let mate_score = mated_in(score);
                reported_score = if mate_score % 2 == 0 {
                    mate_score / 2
                } else {
                    (mate_score / 2) + 1
                };
            }

            let elapsed = global_time.elapsed().as_millis() as u64;
            let nps = if elapsed == 0 {
                0
            } else {
                self.nodecount * 1000 / elapsed
            };

            println!(
                "info depth {} score {} {} nodes {} nps {} time {} pv{}",
                depth,
                scoretype,
                reported_score,
                self.nodecount,
                nps,
                elapsed,
                format_pv(&pv)
            );
            
            best_move = pv[0];
            if depth as u8 >= self.timer.max_depth
                || elapsed > self.timer.time_alloted
            {
                break;
            }
        }

        println!("bestmove {}", best_move);
        self.timer.refresh();
    }

    pub fn new(recv: Receiver<bool>) -> Self {
        Searcher {
            nodecount: 0,
            qnodecount: 0,
            timer: Timer::default(),
            tt: TranspositionTable::new(32),
            stop: recv,
            board: Board::new(),
            ord: OrderData::new(),
        }
    }

    pub fn reset(&mut self) {
        self.board = Board::new();
        self.tt.clear();
        self.ord.clear();
        self.nodecount = 0;
        self.qnodecount = 0;
        self.timer = Timer::new();
    }

    pub fn hash_resize(&mut self, new_size: usize) {
        self.tt = TranspositionTable::new(new_size);
    }

    pub fn clear_hash(&mut self) {
        self.tt.clear()
    }

    pub fn get_board(&mut self) -> &mut Board {
        &mut self.board
    }

    fn refresh(&mut self) {
        self.ord.age_history();
        self.nodecount = 0;
        self.qnodecount = 0;
    }
}

pub fn format_pv(pv: &PVLine) -> String {
    let mut starting_str = String::new();
    for action in pv.iter() {
        write!(&mut starting_str, " {}", action).unwrap();
    }
    starting_str
}