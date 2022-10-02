use std::{
    cmp,
    io::stdin,
    sync::mpsc::{self, Sender},
    thread,
    time::Duration,
};

use crate::{
    board_state::{
        board::Board,
        typedefs::{Square, BISHOP, BLACK, KNIGHT, QUEEN, ROOK, WHITE},
    },
    move_generation::{
        action::{Action, Move},
        makemove::PROMOTION,
    },
    search::{
        alphabeta::{SearchControl, SearchData},
        timer::Timer,
        transposition::TranspositionTable,
    },
};
use crate::{go_next, send};

pub fn gameloop() {
    let newb =
        Board::parse_fen("rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1").unwrap();
    let newdata = SearchData::new(Timer::new());

    let mut control = SearchControl {
        searchdata: newdata,
        curr_ply: 0,
        curr_board: newb,
    };
    let mut comm = Communicator { comm: None };
    let (tx, rx) = mpsc::channel::<Control>();
    control.searchdata.message_recv = Some(rx);
    println!("Shen Yu by Aaron Li");
    comm.comm = Some(tx);
    thread::spawn(move || {
        control.parse_commands();
    });
    comm.parse_commands();
}

pub struct Communicator {
    pub comm: Option<Sender<Control>>,
}

#[derive(PartialEq, Eq)]
pub enum Control {
    Go,
    Stop,
    Quit,
    Reset,
    Show,
    WTimeSet(u64),
    BTimeSet(u64),
    IsTimedW(bool),
    IsTimedB(bool),
    NodeSet(u64),
    DepthSet(u8),
    SetBoard(Board),
    SetOption(OptionType),
}

#[derive(PartialEq, Eq, Debug)]
pub enum OptionType {
    HashSet(usize),
    HashClear,
}

impl SearchControl {
    pub fn parse_commands(&mut self) {
        loop {
            let data = self.get_recv();
            if let Some(msg) = data {
                match msg {
                    Control::Go => {
                        self.go_search();
                    }
                    Control::Stop => {
                        self.searchdata.timer.stopped = true;
                    }
                    Control::Quit => {
                        return;
                    }
                    Control::Reset => {
                        self.reset();
                    }
                    Control::WTimeSet(time) => {
                        if self.curr_board.tomove == WHITE {
                            self.searchdata.timer.time_alloted = time;
                        }
                    }
                    Control::BTimeSet(time) => {
                        if self.curr_board.tomove == BLACK {
                            self.searchdata.timer.time_alloted = time;
                        }
                    }
                    Control::IsTimedW(istimed) => {
                        if self.curr_board.tomove == WHITE {
                            self.searchdata.timer.is_timed = istimed
                        }
                    }
                    Control::IsTimedB(istimed) => {
                        if self.curr_board.tomove == BLACK {
                            self.searchdata.timer.is_timed = istimed
                        }
                    }
                    Control::NodeSet(maxnodes) => self.searchdata.timer.max_nodes = maxnodes,
                    Control::DepthSet(maxdepth) => self.searchdata.timer.maxdepth = maxdepth,
                    Control::SetBoard(board) => self.curr_board = board,
                    Control::SetOption(option) => match option {
                        OptionType::HashSet(num) => {
                            self.searchdata.tt = TranspositionTable::new(num)
                        }
                        OptionType::HashClear => self.searchdata.tt.clear(),
                    },
                    Control::Show => {
                        println!("{}", self.curr_board);
                    }
                }
            }
            thread::sleep(Duration::from_millis(1));
        }
    }
}
impl Communicator {
    pub fn parse_commands(&mut self) {
        let mut cmd = String::new();
        loop {
            cmd.clear();
            stdin().read_line(&mut cmd).unwrap();
            let cmd = cmd.clone();
            let first_word = cmd.split_whitespace().next().unwrap_or(&cmd);

            match first_word {
                "uci" => Self::identify(),
                "setoption" => self.parse_options(cmd),
                "isready" => println!("readyok"),
                "ucinewgame" => {
                    let reset = Control::Reset;
                    send!(self, reset);
                }
                "position" => self.parse_position(cmd),
                "go" => self.go(cmd),
                "stop" => {
                    let stop = Control::Stop;
                    send!(self, stop);
                }
                "quit" => {
                    let quit = Control::Quit;
                    send!(self, quit);
                    break;
                }
                "show" => {
                    let show = Control::Show;
                    send!(self, show);
                }
                _ => go_next!(),
            }
            thread::sleep(Duration::from_millis(1));
        }
    }
    pub fn identify() {
        println!("id name ShenYu");
        println!("id author Aaron Li");
        println!("option name Hash type spin default 32 min 0 max 8192");
        println!("option name Clear Hash type button");
        println!("uciok");
    }

    pub fn parse_options(&mut self, options: String) {
        let mut split = options.split(' ');
        let optiontype = split.nth(2).unwrap_or("");
        match optiontype {
            "Hash" => {
                let next = split.next().unwrap_or("");
                if let Ok(size) = str::parse::<usize>(next) {
                    let option = Control::SetOption(OptionType::HashSet(size));
                    send!(self, option);
                }
            }
            "Clear Hash" => {
                self.comm
                    .as_ref()
                    .unwrap()
                    .send(Control::SetOption(OptionType::HashClear))
                    .unwrap();
            }
            _ => (),
        }
    }

    pub fn parse_position(&mut self, cmd: String) {
        let mut split = cmd.split_whitespace();
        let input_type = split.nth(1).unwrap_or("");
        match input_type {
            "fen" => {
                if let Some(fen) = split.next() {
                    if let Ok(newb) = Board::parse_fen(fen) {
                        let board = Control::SetBoard(newb);
                        send!(self, board);
                    }
                }
            }
            "startpos" => {
                let mut newb =
                    Board::parse_fen("rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1")
                        .unwrap();
                for i in split {
                    if i == "moves" {
                        continue;
                    }
                    match newb.do_input_move(i.to_owned()) {
                        Ok(action) => {
                            newb = newb.do_move(action);
                        }
                        Err(_) => panic!("What"),
                    }
                }
                let board = Control::SetBoard(newb);
                send!(self, board);
            }
            _ => (),
        }
    }

    pub fn go(&mut self, cmd: String) {
        let mut wtime = u64::MAX;
        let mut winc = u64::MAX;
        let mut btime = u64::MAX;
        let mut binc = u64::MAX;
        let mut maxdepth = u8::MAX;
        let mut maxnodes = u64::MAX;

        let mut movetime = u64::MAX;
        let parts = cmd.split_whitespace();
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
                        maxdepth = num;
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
                        maxnodes = num;
                    }
                    Type::Infinite => continue,
                },
            }
        }
        let w_best_time = cmp::min(Timer::allocate_time(wtime, winc), movetime);
        let istimed_w = w_best_time < 500_000;
        if istimed_w {
            let time = Control::WTimeSet(w_best_time);
            send!(self, time);
        }
        let istimemsg = Control::IsTimedW(istimed_w);
        send!(self, istimemsg);

        let b_best_time = cmp::min(Timer::allocate_time(btime, binc), movetime);
        let istimed_b = b_best_time < 500_000;
        if istimed_b {
            let time = Control::BTimeSet(b_best_time);
            send!(self, time);
        }
        let istimemsg = Control::IsTimedB(istimed_b);
        send!(self, istimemsg);

        let maxnodes_msg = Control::NodeSet(maxnodes);
        send!(self, maxnodes_msg);

        let maxdepth_msg = Control::DepthSet(maxdepth);
        send!(self, maxdepth_msg);
        let go = Control::Go;
        send!(self, go);
    }
}

#[macro_export]
macro_rules! go_next {
    () => {{
        thread::sleep(Duration::from_millis(10));
        continue;
    }};
}
#[macro_export]
macro_rules! send {
    ($from:ident, $msg:ident) => {
        ($from).comm.as_ref().unwrap().send($msg).unwrap();
    };
}

impl Board {
    pub fn do_input_move(&mut self, movestring: String) -> Result<Move, u8> {
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
                    return Ok(action);
                }
            }
            Err(0)
        } else {
            for i in 0..moves.length {
                let action = moves[i as usize];
                if action.move_from() == fromsqr && action.move_to() == tosqr {
                    return Ok(action);
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
