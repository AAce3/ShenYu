use crate::movegen::genmoves::GenType;

use super::{board::Board, movelist::MoveList, zobrist::Zobrist};

#[allow(dead_code)]
pub fn perft_debug() {
    let args: Vec<String> = std::env::args().collect();
 
    if args.len() < 3 || args.len() > 4 {
        eprintln!("Usage: {} <depth> <fen> [moves]", args[0]);
        std::process::exit(1);
    }

    let depth = args[1].parse().expect("Invalid depth");
   let fen = &args[2];

    let mut board = Board::new();
    board.parse_fen(fen).unwrap();

    if args.len() == 4 {
        let moves = args[3].split_whitespace();
        board.parse_moves(moves);
    }

    board.divide_perft(depth);
}

#[derive(Clone, Copy, Default)]
struct PerftEntry {
    key: Zobrist,
    depth: u8,
    num_nodes: u64,
}

impl Board {
    pub fn divide_perft(&mut self, depth: u8) {
        let mut start_list = MoveList::new();
        self.genmoves::<{ GenType::ALL }>(&mut start_list);
        let mut total_nodes = 0;

        for action in start_list.iter() {
            self.make_move(**action);

            let perft = self.perft(depth - 1);
            self.unmake_move(**action);
            total_nodes += perft;
            println!("{} {perft}", **action);
        }
        println!("\n{}", total_nodes);
    }

    pub fn hashed_divide_perft(&mut self, depth: u8, hash_size: usize) {
        let mut hashtable = vec![PerftEntry::default(); hash_size];
        let mut start_list = MoveList::new();
        self.genmoves::<{ GenType::ALL }>(&mut start_list);
        let mut total_nodes = 0;

        for action in start_list.iter() {
            self.make_move(**action);

            let perft = self.hashed_perft(depth - 1, &mut hashtable);
            self.unmake_move(**action);
            total_nodes += perft;
            println!("{} {perft}", **action);
        }
        println!("\n{}", total_nodes);
    }

    fn hashed_perft(&mut self, depth: u8, hashtable: &mut Vec<PerftEntry>) -> u64 {
        if depth == 0 {
            return 1;
        }
        let len = hashtable.len();
        let perft_entry = &mut hashtable[self.zobrist() as usize % len];
        if self.zobrist() == perft_entry.key && depth == perft_entry.depth {
            perft_entry.num_nodes
        } else {
            let mut nodes = 0;
            let mut list = MoveList::new();
            self.genmoves::<{ GenType::ALL }>(&mut list);
            if depth == 1 {
                return list.len() as u64
            }

            for action in list.iter() {
                self.make_move(**action);
                nodes += self.perft(depth - 1);
                self.unmake_move(**action)
            }
            
            perft_entry.num_nodes = nodes;
            perft_entry.key = self.zobrist();
            perft_entry.depth = depth;
            nodes
        }
    }
    fn perft(&mut self, depth: u8) -> u64 {
        if depth == 0 {
            return 1;
        }
        let mut nodes = 0;
        let mut list = MoveList::new();
        self.genmoves::<{ GenType::ALL }>(&mut list);

     //   if depth == 1 {
     //       return list.len() as u64;
     //   }

        for action in list.iter() {
            self.make_move(**action);
            nodes += self.perft(depth - 1);
            self.unmake_move(**action)
        }

        nodes
    }
}
