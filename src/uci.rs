use std::{
    cmp,
    io::{self, stdin},
    sync::{
        mpsc::{self, Receiver, Sender},

    },
    thread,
    time::{Duration, Instant},
};

use crate::{
    board_state::{
        board::Board,
        typedefs::{Square, BISHOP, BLACK, KNIGHT, QUEEN, ROOK, WHITE},
    },
    move_generation::{action::Action, makemove::PROMOTION},
    search::{alphabeta::SearchControl, timer::Timer},
};
use crate::{go_next, search::transposition::TranspositionTable};

pub struct Communicator {
    pub search: SearchControl,
    pub comm: Option<Sender<bool>>,
}

impl Communicator {
    pub fn parse_commands(&'static mut self) {
        let mut cmd = String::new();
        let channel = mpsc::channel::<bool>();
        self.comm = Some(channel.0);
        self.search.searchdata.timer.recv = Some(channel.1);
        thread::spawn(move || {
            loop {
            
                cmd.clear();
                stdin().read_line(&mut cmd).unwrap();
                let cmd = cmd.clone();
                let first_word = cmd.split(' ').next().unwrap_or(&cmd);
    
                match first_word {
                    "uci" => Self::identify(),
                    "setoption" => self.parse_options(cmd),
                    "isready" => println!("readyok"),
                    "ucinewgame" => self.search.reset(),
                    "position" => self.parse_position(cmd),
                    "go" => self.go(cmd),
                    "stop" => self.comm.as_ref().unwrap().send(true).unwrap(),
                    "quit" => return,
                    _ => go_next!(),
                }
        }
    }
        );
        
        
    }
    pub fn identify() {
        println!("id name ShenYu");
        println!("id author Aaron Li");
        println!("option name Hash Size type spin default 32 min 1 max 8192");
        println!("option name Clear Hash type button");
        println!("uciok");
    }

    pub fn parse_options(&mut self, options: String) {
        let mut split = options.split(' ');
        let optiontype = split.nth(2).unwrap_or("");
        match optiontype {
            "Hash Size" => {
                let next = split.next().unwrap_or("");
                if let Ok(size) = str::parse::<usize>(next) {
                    self.search.searchdata.tt = TranspositionTable::new(size)
                }
            }
            "Clear Hash" => {
                self.search.searchdata.tt.clear();
            }
            _ => (),
        }
    }

    pub fn parse_position(&mut self, cmd: String) {
        let mut split = cmd.split(' ');
        let input_type = split.nth(1).unwrap_or("");
        match input_type {
            "fen" => {
                if let Some(fen) = split.next() {
                    if let Ok(nextb) = Board::parse_fen(fen) {
                        self.search.curr_board = nextb;
                    }
                }
            }
            "startpos" => {
                let mut newb =
                    Board::parse_fen("rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1")
                        .unwrap();
                for i in split {
                    match newb.do_input_move(i.to_owned()) {
                        Ok(board) => newb = board,
                        Err(_) => break,
                    }
                }
                self.search.curr_board = newb
            }
            _ => (),
        }
    }

    pub fn go(&mut self, cmd: String) {
        let mut mytime = u64::MAX;
        let mut myinc = u64::MAX;
        let mut maxdepth = u8::MAX;
        let mut maxnodes = u64::MAX;
        let mut maxtime = u64::MAX;
        let parts = cmd.split(' ');
        let mut curr_type = Type::Infinite;
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
        }
        for part in parts {
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
                _ => match curr_type {
                    Type::Ponder => {
                        curr_type = Type::Infinite;
                    }
                    Type::WTime => {
                        if self.search.curr_board.tomove == WHITE {
                            let num = str::parse::<u64>(part).unwrap_or(0);
                            mytime = num;
                        }
                        curr_type = Type::Infinite;
                    }
                    Type::BTime => {
                        if self.search.curr_board.tomove == BLACK {
                            let num = str::parse::<u64>(part).unwrap_or(0);
                            mytime = num;
                        }
                        curr_type = Type::Infinite;
                    }
                    Type::WInc => {
                        if self.search.curr_board.tomove == WHITE {
                            let num = str::parse::<u64>(part).unwrap_or(0);
                            myinc = num;
                        }
                        curr_type = Type::Infinite;
                    }
                    Type::BInc => {
                        if self.search.curr_board.tomove == BLACK {
                            let num = str::parse::<u64>(part).unwrap_or(0);
                            myinc = num;
                        }
                        curr_type = Type::Infinite;
                    }
                    Type::MovesToGo => {
                        curr_type = Type::Infinite;
                    }
                    Type::Depth => {
                        let num = str::parse::<u8>(part).unwrap_or(0);
                        maxdepth = num;
                    }
                    Type::Mate => {
                        curr_type = Type::Infinite;
                    }
                    Type::MoveTime => {
                        let num = str::parse::<u64>(part).unwrap_or(0);
                        maxtime = num;
                    }
                    Type::Nodes => {
                        let num = str::parse::<u64>(part).unwrap_or(0);
                        maxnodes = num;
                    }
                    Type::Infinite => continue,
                },
            }
        }
        let best_time = cmp::min(Timer::allocate_time(mytime, myinc), maxtime);
        let istimed = best_time < 500_000;
        self.search.searchdata.timer.time_alloted = best_time;
        self.search.searchdata.timer.max_nodes = maxnodes;
        self.search.searchdata.timer.maxdepth = maxdepth;
        self.search.searchdata.timer.start_time = Instant::now();
        self.search.searchdata.timer.is_timed = istimed;
        self.search.searchdata.timer.stopped = false;
        self.search.searchdata.timer.time_alloted = best_time;
       
        self.search.go_search();
    }
}

#[macro_export]
macro_rules! go_next {
    () => {{
        thread::sleep(Duration::from_millis(10));
        continue;
    }};
}
pub fn spawn_stdin_channel() -> Receiver<String> {
    let (tx, rx) = mpsc::channel::<String>();
    thread::spawn(move || loop {
        let mut buffer = String::new();
        io::stdin().read_line(&mut buffer).unwrap();
        tx.send(buffer).unwrap();
    });
    rx
}

impl Board {
    pub fn do_input_move(&self, movestring: String) -> Result<Board, u8> {
        let moves = self.generate_moves::<true, true>();
        let from = &movestring[..2];
        let to = &movestring[2..4];
        let fromsqr = match str_to_sqr(from) {
            Some(sqr) => sqr,
            None => return Err(0),
        };
        let tosqr = match str_to_sqr(to) {
            Some(sqr) => sqr,
            None => return Err(0),
        };

        if movestring.len() == 5 {
            // promotion
            let piece_str = &movestring[4..];
            let pr_piece = match piece_str {
                "q" => QUEEN,
                "n" => KNIGHT,
                "b" => BISHOP,
                "r" => ROOK,
                _ => return Err(0),
            };

            for i in 0..moves.length {
                let action = moves[i as usize];
                if action.move_type() == PROMOTION
                    && action.promote_to() == pr_piece
                    && action.move_from() == fromsqr
                    && action.move_to() == tosqr
                {
                    return Ok(self.do_move(action));
                }
            }
            Err(0)
        } else {
            for i in 0..moves.length {
                let action = moves[i as usize];
                if action.move_from() == fromsqr && action.move_to() == tosqr {
                    return Ok(self.do_move(action));
                }
            }
            Err(0)
        }
    }
}

fn str_to_sqr(squarename: &str) -> Option<Square> {
    let squarename = squarename.chars().collect::<Vec<char>>();
    if squarename.len() != 2 {
        None
    } else {
        let file = match squarename[0] {
            'a' => 0,
            'b' => 1,
            'c' => 2,
            'd' => 3,
            'e' => 4,
            'f' => 5,
            'g' => 6,
            'h' => 7,
            _ => return None,
        };

        let rank = match char::to_digit(squarename[1], 10) {
            Some(sqr) => sqr as u8 - 1,
            None => return None,
        };
        Some(rank * 8 + file)
    }
}
