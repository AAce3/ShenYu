use std::io::{self, BufRead};

use colored::Colorize;

use crate::{board_state::board::Board, move_generation::action::Action};

pub fn perft_debug() {
    let mut mode = Mode::ReadFen;
    let mut curr_board =
        Board::parse_fen("rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1").unwrap();
    let mut my_list: Vec<MoveData> = Vec::new();
    let mut sf_list: Vec<MoveData> = Vec::new();
    loop {
        match mode {
            Mode::ReadFen => {
                my_list.clear();
                sf_list.clear();
                println!("Fen:");
                let command = get_command();
                crate::check_stop!(command);
                match Board::parse_fen(command.as_str()) {
                    Ok(board) => {
                        curr_board = board;
                        mode = Mode::DoPerft;
                    }
                    Err(_) => crate::bad!(),
                }
            }
            Mode::DoPerft => {
                println!("Depth:");
                let command = get_command();
                crate::check_stop!(command);
                let depth: u8 = match command.parse::<u8>() {
                    Ok(val) => val,
                    Err(_) => crate::bad!(),
                };
                my_list = curr_board.debug_perft(depth);
                mode = Mode::SFData;
                println!("SF:")
            }
            Mode::SFData => {
                let command = get_command();
                crate::check_stop!(command);
                if command.as_str().is_empty() {
                    // we are at a new line, so we have finished parsing sf data
                    for sf_move in &sf_list {
                        let movestring = &sf_move.name;
                        let count = sf_move.count;
                        let my_move = my_list.iter().find(|movedata| movedata.name == *movestring);
                        match my_move {
                            None => println!(
                                "missing {}: ({})",
                                movestring.bright_red(),
                                format!("{}", sf_move.count).bright_red()
                            ),
                            Some(movedata) => {
                                if movedata.count == count {
                                    println!(
                                        "{}: {}",
                                        movestring.green(),
                                        format!("{}", movedata.count).green()
                                    );
                                } else {
                                    println!(
                                        "{}: {} ({})",
                                        movestring.purple(),
                                        format!("{}", count).purple(),
                                        format!("{}", movedata.count).red().strikethrough()
                                    );
                                }
                            }
                        }
                    }
                    for my_move in &my_list {
                        let movestring = &my_move.name;
                        let sf_move = sf_list.iter().find(|movedata| movedata.name == *movestring);
                        match sf_move {
                            None => println!("{}", movestring.red().strikethrough()),
                            Some(_) => continue,
                        }
                    }
                    mode = Mode::ReadFen;
                } else {
                    let v = command.split(": ").collect::<Vec<&str>>();
                    let name = v[0].to_owned();
                    let count = v[1].parse::<u32>().unwrap();
                    let new_movedata = MoveData { name, count };
                    sf_list.push(new_movedata);
                }
            }
        }
    }
}
fn get_command() -> String {
    let stdin = io::stdin();
    let mut i = stdin.lock().lines();

    i.next().unwrap().unwrap()
}

#[macro_export]
macro_rules! bad {
    () => {{
        eprintln!("Bad command");
        continue;
    }};
}
#[macro_export]
macro_rules! check_stop {
    ($command_name: ident) => {
        if $command_name.as_str() == "stop"{
            return;
        }
    };
}
enum Mode {
    ReadFen,
    DoPerft,
    SFData,
}
#[derive(Clone)]
struct MoveData {
    name: String,
    count: u32,
}

impl Board {
    fn debug_perft(&self, depth: u8) -> Vec<MoveData> {
        let moves = self.generate_moves(true, true);
        let mut newmove_vec = vec![
            MoveData {
                name: String::new(),
                count: 0
            };
            moves.length as usize
        ];
        assert_ne!(depth, 0);

        for i in 0..moves.length {
            let action = moves[i as usize];
            let nextperft = self.do_move(action).perft(depth - 1);
            newmove_vec[i as usize].name = action.to_algebraic();
            newmove_vec[i as usize].count = nextperft;
        }
        newmove_vec
    }
}
