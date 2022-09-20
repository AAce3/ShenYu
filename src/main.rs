

use std::{thread, sync::mpsc};

use shenyu::{
    board_state::{board::Board},
    search::{
        alphabeta::{SearchData, SearchControl}, timer::Timer,
    }, uci::{Communicator, Control},
};

fn main() {


    let newb =
        Board::parse_fen("rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1").unwrap();
    let newdata = SearchData::new(Timer::new());

    let mut control = SearchControl{
        searchdata: newdata,
        curr_ply: 0,
        curr_board: newb,
    };
    let mut comm = Communicator{
        comm: None,
    };
    let (tx, rx) = mpsc::channel::<Control>();
    control.searchdata.message_recv = Some(rx);
    comm.comm = Some(tx);
    thread::spawn(move || {
        control.parse_commands();
    });
    comm.parse_commands();
}
