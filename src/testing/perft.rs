use std::{time::Instant, path::Path, fs::File, io::{BufReader, BufRead}};

use crate::{board_state::board::Board, move_generation::action::Action};

pub fn perft_testing(){
    // thank you to leorik for test positions
    let path = Path::new(r"C:\Users\aaron\VSCode Projects\shenyu\src\testing\perft_test_positions.txt");
    let file = File::open(path).unwrap();
    let reader = BufReader::new(file);
    for (number, line) in reader.lines().enumerate(){
        let data = line.unwrap();
        let data = data.split("; ").collect::<Vec<&str>>();
        let fen = data[0];
        let depth = data[1].parse::<u8>().unwrap_or_else(|_| panic!("Bad depth! Error on line {}", number));
        let result = data[2].replace(';', "").parse::<u32>().unwrap_or_else(|_| panic!("Bad result! Error on line {}", number));
        let board = Board::parse_fen(fen).unwrap_or_else(|_| panic!("Bad fen! Error on line {}", number));
        let perftval = board.perft(depth);
        if perftval != result{
            println!("{}", fen);
            panic!("Fen failed.");
        }
    }
    println!("Finished!");
}

impl Board {
    
    pub fn perft(&self, depth: u8) -> u32 {
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
                let newb = self.do_move(action);
                accum += newb.perft(depth - 1);
            }
            accum
        }
    }

    pub fn divide_perft(&self, depth: u8) {
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
}
