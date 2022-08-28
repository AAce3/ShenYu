use shenyu::board_state::board::Board;

fn main() {
    let newb = Board::parse_fen("rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1").unwrap();
    newb.divide_perft(6);
}

