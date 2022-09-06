use std::time::Instant;

use shenyu::{
    board_state::board::Board,
    move_generation::{action::Action, list::List},
    search::{
        alphabeta::SearchData,
        moveorder::{History, Killer},
        transposition::TranspositionTable,
    },
    testing::{log::clear_logs, perft::PerftEntry},
};

fn main() {
    
    clear_logs();
    let newb =
        Board::parse_fen("r3k2r/p1ppqpb1/bn2pnp1/3PN3/1p2P3/2N2Q1p/PPPBBPPP/R3K2R w KQkq - 0 1").unwrap();
    let mut tt = TranspositionTable::new(16);
    let mut data = SearchData {
        killers: Killer {
            table: [[0; 2]; 256],
        },
        history: History {
            history: [[[0; 64]; 6]; 2],
        },
        bestmove: 0,
        nodecount: 0,
        qnodecount: 0,
        tt: &mut tt,
    };
    let start = Instant::now();
    let mut prev = 0;
    for i in 1..9 {
        let alpha = if prev == 0{
            -10_000
        } else {
            prev - 50
        };
        let beta = if prev == 0{
            10_000
        } else {
            prev + 50
        };
        let mut pv = List::new();
        let score = newb.negamax(i, alpha, beta, &mut data, 0, &mut pv);
        prev = score;
        print!(
            "depth: {}, score: {}, nodes: {}, qnodes: {}, time: {}ms, pv",
            i,
            score,
            data.nodecount,
            data.qnodecount,
            start.elapsed().as_millis()
        );
        for i in 0..pv.length{
            print!(" {}", pv[i as usize].to_algebraic());
        }
        println!();
    }
    
}
