use shenyu::{board_state::board::Board, move_generation::action::Action, search::{alphabeta::SearchData, moveorder::{Killer, History}}};

const STARTING_DATA: SearchData = SearchData{
    killers: Killer{
        table: [[0; 2]; 256],
    },
    history: History{
        history: [[[0; 64]; 6]; 2],
    },
    bestmove: 0,
    nodecount: 0,
};
fn main() {
   let newb = Board::parse_fen("rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1").unwrap();

   let mut data = STARTING_DATA;
   let score = newb.alphabeta::<true>(i16::MIN, i16::MAX, 6, &mut data, 0);
   println!("{}", score);
   println!("{}", data.bestmove.to_algebraic());
   println!("{}", data.nodecount);
}

