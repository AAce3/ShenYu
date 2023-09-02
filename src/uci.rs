use std::{
    cmp, io,
    sync::{Arc, Mutex},
    thread,
};

use crate::{
    movegen::{board::Board, genmoves::GenType, movelist::MoveList, types::Color},
    search::{searchcontrol::Searcher, timer::Timer},
};

const VERSION: &str = "2.0.0";

impl Board {
    pub fn parse_moves<'a, T>(&mut self, actions: T) -> Option<()>
    where
        T: Iterator<Item = &'a str>,
    {
        for movestring in actions {
            self.parse_move(movestring)?;
        }
        Some(())
    }

    fn parse_move(&mut self, movestring: &str) -> Option<()> {
        let mut list = MoveList::new();
        self.genmoves::<{ GenType::ALL }>(&mut list);
        let action = list
            .iter()
            .find(|&action| action.to_string() == movestring)?;
        self.make_move(**action);
        Some(())
    }
}

pub fn gameloop() {
    let (tx, rx) = crossbeam::channel::unbounded::<bool>();
    let searchdata_ptr = Arc::new(Mutex::new(Searcher::new(rx)));
    let mut cmd = String::new();
    loop {
        cmd.clear();
        io::stdin().read_line(&mut cmd).unwrap();
        let mut split = cmd.split_ascii_whitespace();
        let command_type = split.next().unwrap_or(&cmd);

        match command_type {
            "uci" => {
                identify();
                continue;
            }
            "isready" => {
                println!("readyok");
                continue;
            }
            "stop" => {
                tx.send(true).expect("Error: Search Thread Disconnected");
                continue;
            }
            "quit" => return,
            _ => (),
        }

        let searchdata_clone = searchdata_ptr.clone();
        let mut searchdata = searchdata_clone.lock().unwrap();

        match command_type {
            "setoption" => set_option(&mut searchdata, split),
            "ucinewgame" => searchdata.reset(),
            "position" => parse_position(&mut searchdata, split),
            "go" => {
                if parse_go(&mut searchdata, split) {
                    continue;
                }
                drop(searchdata);
                thread::spawn(move || {
                    let mut searcher = searchdata_clone.lock().unwrap();
                    searcher.search();
                });
            }
            _ => continue,
        }
    }
}

fn identify() {
    println!("id name ShenYu {VERSION}");
    println!("id author Aaron Li");
    println!("option name Hash type spin default 64 min 0 max 65536");
    println!("option name Clear Hash type button");
    println!("uciok");
}

fn set_option<'a, T>(searchdata: &mut Searcher, mut string_iter: T)
where
    T: Iterator<Item = &'a str>,
{
    if string_iter.next().unwrap_or_default() != "name" {
        println!("Invalid uci command");
        return;
    }

    let option_type = string_iter.next().unwrap_or_default();
    match option_type {
        "Hash" | "hash" => {
            let mut next = string_iter.next().unwrap();
            if next != "value" {
                return;
            }
            next = string_iter.next().unwrap();
            if let Ok(size) = str::parse::<usize>(next) {
                searchdata.hash_resize(size);
            }
        }
        "Clear Hash" | "clear hash" => searchdata.clear_hash(),
        _ => (),
    }
}

fn parse_position<'a, T>(searchdata: &mut Searcher, string_iter: T)
where
    T: Iterator<Item = &'a str>,
{
    let board = searchdata.get_board();
    let mut fen_string = String::new();
    enum Type {
        Fen,
        Moves,
        None,
    }

    let mut currtype = Type::None;
    let mut has_moves = false;

    for value in string_iter {
        match value {
            "startpos" => {
                fen_string.push_str("rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1");
            }

            "fen" => currtype = Type::Fen,
            "moves" => {
                has_moves = true;
                board.parse_fen(&fen_string).unwrap();
                currtype = Type::Moves;
            }
            _ => match currtype {
                Type::Fen => {
                    fen_string += value;
                    fen_string += " "
                }
                Type::Moves => {
                    if board.parse_move(value).is_none() {
                        return;
                    }
                }
                Type::None => return,
            },
        }
    }
    if !has_moves {
        let res = board.parse_fen(&fen_string);
        if res.is_err() {
            println!("Invalid fen!");
        }
    }
}

fn parse_go<'a, T>(searchdata: &mut Searcher, string_iter: T) -> bool
where
    T: Iterator<Item = &'a str>,
{
    enum Type {
        Ponder,
        WTime,
        BTime,
        WInc,
        BInc,
        MovesToGo,
        Depth,
        Mate,
        Nodes,
        MoveTime,
        Infinite,
        Perft,
    }

    let mut wtime = u64::MAX;
    let mut winc = u64::MAX;
    let mut btime = u64::MAX;
    let mut binc = u64::MAX;
    let mut max_depth = u8::MAX;
    let mut max_nodes = u64::MAX;

    let mut movetime = u64::MAX;
    let mut curr_type = Type::Infinite;

    for part in string_iter {
        match part {
            "go" => continue,
            "ponder" => curr_type = Type::Ponder,
            "wtime" => curr_type = Type::WTime,
            "btime" => curr_type = Type::BTime,
            "winc" => curr_type = Type::WInc,
            "binc" => curr_type = Type::BInc,
            "movestogo" => curr_type = Type::MovesToGo,
            "depth" => curr_type = Type::Depth,
            "mate" => curr_type = Type::Mate,
            "movetime" => curr_type = Type::MoveTime,
            "infinite" => curr_type = Type::Infinite,
            "nodes" => curr_type = Type::Nodes,
            "perft" => curr_type = Type::Perft,
            _ => match curr_type {
                Type::Ponder => {
                    curr_type = Type::Infinite;
                }
                Type::WTime => {
                    let num = str::parse::<u64>(part).unwrap_or(0);
                    wtime = num;

                    curr_type = Type::Infinite;
                }
                Type::BTime => {
                    let num = str::parse::<u64>(part).unwrap_or(0);
                    btime = num;

                    curr_type = Type::Infinite;
                }
                Type::WInc => {
                    let num = str::parse::<u64>(part).unwrap_or(0);
                    winc = num;

                    curr_type = Type::Infinite;
                }
                Type::BInc => {
                    let num = str::parse::<u64>(part).unwrap_or(0);
                    binc = num;

                    curr_type = Type::Infinite;
                }
                Type::MovesToGo => {
                    curr_type = Type::Infinite;
                }
                Type::Depth => {
                    let num = str::parse::<u8>(part).unwrap();
                    max_depth = num;
                }
                Type::Mate => {
                    curr_type = Type::Infinite;
                }
                Type::MoveTime => {
                    let num = str::parse::<u64>(part).unwrap_or(0);
                    movetime = num;
                }
                Type::Nodes => {
                    let num = str::parse::<u64>(part).unwrap_or(0);
                    max_nodes = num;
                }
                Type::Infinite => continue,
                Type::Perft => {
                    let depth = str::parse::<u8>(part).unwrap_or(0);
                    searchdata.get_board().divide_perft(depth);
                    return true;
                }
            },
        }
    }

    let w_best_time = cmp::min(Timer::allocate_time(wtime, winc), movetime);
    let is_timed_w = w_best_time < 500_000;
    if searchdata.get_board().active_color() == Color::W {
        searchdata.timer.is_timed = is_timed_w;
        searchdata.timer.time_alloted = w_best_time;
    }
    let b_best_time = cmp::min(Timer::allocate_time(btime, binc), movetime);
    let is_timed_b = b_best_time < 500_000;
    if searchdata.get_board().active_color() == Color::B {
        searchdata.timer.is_timed = is_timed_b;
        searchdata.timer.time_alloted = b_best_time;
    }
    searchdata.timer.max_nodes = max_nodes;
    searchdata.timer.max_depth = max_depth;
    false
}
