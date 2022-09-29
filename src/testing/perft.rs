use std::{
    fs::File,
    io::{BufRead, BufReader},
    path::Path,
    time::Instant,
};

use crate::{
    board_state::{board::Board, zobrist::ZobristKey},
    move_generation::action::Action,
};

pub fn perft_testing() {
    // Perft tool plays out all moves for white, then black, then white, etc. and only evaluates leaf nodes.
    // Perft uses bulk counting, i.e. rather than playing out moves at frontier nodes it counts the number of moves generated
    // thank you to leorik for test positions
    let path =
        Path::new(r"C:\Users\aaron\VSCode Projects\shenyu\src\testing\perft_test_positions.txt");
    let file = File::open(path).unwrap();
    let reader = BufReader::new(file);
    for (number, line) in reader.lines().enumerate() {
        let data = line.unwrap();
        let data = data.split("; ").collect::<Vec<&str>>();
        let fen = data[0];
        let depth = data[1]
            .parse::<u8>()
            .unwrap_or_else(|_| panic!("Bad depth! Error on line {}", number));
        let result = data[2]
            .replace(';', "")
            .parse::<u32>()
            .unwrap_or_else(|_| panic!("Bad result! Error on line {}", number));
        let mut board =
            Board::parse_fen(fen).unwrap_or_else(|_| panic!("Bad fen! Error on line {}", number));
        let perftval = board.perft(depth);
        if perftval != result {
            println!("{}", fen);
            panic!("Fen failed.");
        }
    }
    println!("Finished!");
}

impl Board {
    pub fn perft(&mut self, depth: u8) -> u32 {
        if depth == 0 {
            1
        } else if depth == 1 {
            let moves = self.generate_moves::<true, true>();
            moves.length as u32
        } else {
            let mut accum = 0;
            let moves = self.generate_moves::<true, true>();
            for i in 0..moves.length {
                let action = moves[i as usize];
                let mut newb = self.do_move(action);
                accum += newb.perft(depth - 1);
            }
            accum
        }
    }

    pub fn divide_perft(&mut self, depth: u8) {
        let moves = self.generate_moves::<true, true>();

        assert_ne!(depth, 0);
        let mut nodes_searched = 0;
        let starting_time = Instant::now();
        for i in 0..moves.length {
            let action = moves[i as usize];
            let nextperft = self.do_move(action).perft(depth - 1);
            nodes_searched += nextperft;
            println!("{}: {}", action.to_algebraic(), nextperft);
        }
        let then = starting_time.elapsed();
        println!("\nNodes searched: {}", nodes_searched);
        println!("Elapsed: {}ms", then.as_millis());
    }

    pub fn hashed_perft(&mut self, depth: u8, tt: &mut Vec<PerftEntry>) -> u32 {
        if depth == 0 {
            1
        } else {
            let key = self.zobrist_key;
            let idx = key as usize % tt.len();
            let entry = &tt[idx];
            if entry.depth == depth && entry.key == key {
                return entry.nodecount;
            }
            let mut accum = 0;
            let moves = self.generate_moves::<true, true>();
            for i in 0..moves.length {
                let action = moves[i as usize];
                let mut newb = self.do_move(action);
                accum += newb.hashed_perft(depth - 1, tt);
            }
            tt[idx].key = key;
            tt[idx].depth = depth;
            tt[idx].nodecount = accum;
            accum
        }
    }
}

#[derive(Clone, Copy)]
pub struct PerftEntry {
    pub key: ZobristKey,
    pub depth: u8,
    pub nodecount: u32,
}
