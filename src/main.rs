use shenyu::board_state::board::Board;

fn main() {
    let newb = Board::parse_fen("8/2p5/3p4/KP5r/1R3p1k/8/4P1P1/8 w - - 0 0").unwrap();
    newb.divide_perft(7);
}

