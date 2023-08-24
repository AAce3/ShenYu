use crate::movegen::genmoves::GenType;

use super::{
    action::Action,
    board::Board,
    movelist::{MoveList},
    zobrist::Zobrist,
};



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




    fn perft(&mut self, depth: u8) -> u64 {
        if depth == 0 {
            return 1;
        }
        let mut nodes = 0;
        let mut list = MoveList::new();
        self.genmoves::<{ GenType::ALL }>(&mut list);

        if depth == 1 {
            return list.len() as u64;
        }

        for action in list.iter() {
            self.make_move(**action);
            nodes += self.perft(depth - 1);
            self.unmake_move(**action)
        }

        nodes
    }
}
