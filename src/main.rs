

use shenyu::{
    board_state::{board::Board},
    search::{
        alphabeta::{SearchData, SearchControl}, timer::Timer,
    }, uci::{spawn_stdin_channel, Communicator},
};

fn main() {


    let newb =
        Board::parse_fen("2q1Rk1r/5p2/1ppp1P2/6Pp/3p1B2/3P3P/PPP1Q3/6K1 b - - 0 1").unwrap();
    let mut newdata = SearchData::new(Timer::new());

    let mut control = SearchControl{
        searchdata: newdata,
        curr_ply: 0,
        curr_board: newb,
    };
    let mut comm = Communicator{
        search: control,
        comm: None,
    };
    comm.parse_commands();
}
