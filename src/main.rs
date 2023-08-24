use uci::gameloop;

use crate::movegen::board::Board;

mod eval;
mod movegen;
pub mod search;
mod uci;

fn main() {
   // let mut newb = Board::new();
    //println!("{}", newb.evaluate());
    gameloop();
}

fn perft_debug() {
    let args: Vec<String> = std::env::args().collect();

    if args.len() < 3 || args.len() > 4 {
        eprintln!("Usage: {} <depth> <fen> [moves]", args[0]);
        std::process::exit(1);
    }

    let depth: u8 = args[1].parse().expect("Invalid depth");
    let fen = &args[2];

    let mut board = Board::default();
    board.parse_fen(fen).unwrap();

    if args.len() == 4 {
        let moves = args[3].split_whitespace();
        board.parse_moves(moves);
    }

    board.divide_perft(depth);
}
