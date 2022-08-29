
use shenyu::{board_state::board::Board, search::moveorder::{OrderData, MovePicker}, move_generation::{action::{ShortMove, Action}, makemove::NORMAL}};


fn main() {

   let board = Board::parse_fen("r3k2r/p1ppqpb1/bn2pnp1/3PN3/1p2P3/2N2Q1p/PPPBBPPP/R3K2R b KQkq - 0 0").unwrap();
   let neword = OrderData{
      history: [[[0; 64]; 6]; 2],
      killers: [[ShortMove::new_move(45, 30, NORMAL), ShortMove::new_move(42, 35, NORMAL)]; 256],
   };
   
   for action in MovePicker::new(&board, &neword, 0, 0){
      println!("{}", action.to_algebraic());
   }
}

