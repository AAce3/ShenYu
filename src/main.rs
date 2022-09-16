

use shenyu::{
    board_state::{board::Board},
    search::{
        alphabeta::{SearchData, SearchControl},
    },
};

fn main() {


    let newb =
        Board::parse_fen("rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1").unwrap();
    let newdata = SearchData::new();

    let mut control = SearchControl{
        searchdata: newdata,
        curr_ply: 0,
        curr_board: newb,
    };
    control.parse_commands();
    
}
