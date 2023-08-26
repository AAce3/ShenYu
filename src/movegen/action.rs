use std::{mem, fmt::{self, Write}, ops::Deref};

use super::{
    board::Board,
    types::{square, Piece, Square},
};

#[derive(Clone, Copy, Default, Debug, PartialEq, Eq)]
pub struct Action(pub u16);

#[derive(Clone, Copy, Default, Debug, PartialEq, Eq)]
pub struct ScoredMove {
    action: Action,
    score: i16,
}

impl ScoredMove {

    pub fn score(&self) -> i16 {
        self.score
    }

    pub fn set_score(&mut self, value: i16) {
        self.score = value
    }
}

impl From<Action> for ScoredMove {
    fn from(action: Action) -> Self {
        Self {
            action,
            score: i16::MIN,
        }
    }
}

impl Deref for ScoredMove {
    type Target = Action;

    fn deref(&self) -> &Self::Target {
        &self.action
    }
}
// 0000 0000 0011 1111 <- Move From
// 0000 1111 1100 0000 <- Move To
// 0011 0000 0000 0000 <- Move Type
// 1100 0000 0000 0000  <- Promotion


// promotion: N = 0, B = 1, R = 2, Q = 3. Exactly 2 bits
// to retrive original piece, just add 1. Normally, N = 1, B = 2, etc.

#[derive(PartialEq, Eq, Debug)]
pub enum MoveType {
    #[allow(dead_code)]
    Normal,
    Castle,
    Promotion,
    Passant,
}

impl fmt::Display for Action {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", square::name(self.from()))?;
        write!(f, "{}", square::name(self.to()))?;
        if self.move_type() == MoveType::Promotion {
            let pr_char = match self.pr_piece() {
                Piece::N => 'n',
                Piece::B => 'b',
                Piece::R => 'r',
                Piece::Q => 'q',
                _ => panic!("This shouldn't happen"),
            };
            f.write_char(pr_char)?;
        }
        Ok(())
    }
}

impl Action {
    pub fn new(from: Square, to: Square) -> Self {
        let num = (from as u16) | (to as u16) << 6;
        Self(num)
    }

    pub fn new_type(from: Square, to: Square, movetype: MoveType) -> Self {
        let num = (from as u16) | (to as u16) << 6 | (movetype as u16) << 12;
        Self(num)
    }

    pub fn new_pr(from: Square, to: Square, pr_piece: Piece) -> Self {
        let num = (from as u16) | (to as u16) << 6 | (MoveType::Promotion as u16) << 12 | (pr_piece as u16 - 1) << 14;
        Self(num)
    }

    pub fn from(&self) -> Square {
        (self.0 & 0b111111) as u8
    }

    pub fn to(&self) -> Square {
        ((self.0 >> 6) & 0b111111) as u8
    }

    pub fn move_type(&self) -> MoveType {
        unsafe { mem::transmute::<u8, MoveType>(((self.0 >> 12) & 0b11) as u8) }
    }

    pub fn pr_piece(&self) -> Piece {
        Piece::from(((self.0 >> 14) & 0b11) as u8 + 1)
    }
}

impl Board {
    pub fn make_nullmove(&mut self) {
        self.push_info();
        self.reset_fifty();
        self.reset_passant();
        self.swap_sides();
    }

    pub fn unmake_nullmove(&mut self) {
        self.swap_sides();
        self.pop_info();
    }
    
    pub fn make_move(&mut self, action: Action) {
        self.push_info(); // create a new info store
        self.increment_fifty();
        self.reset_passant();
        let from = action.from();
        let to = action.to();
        let move_type = action.move_type();
        let us = self.active_color();
        
        let them = !us;
        let mut update_castle = false; // flag to update castle, used if rook or king moves

        match move_type {
            MoveType::Normal => {
              
                // check if the move is a capture
                if self.is_color(to, them) {
                    let captured_piece = self.get_piece(to);
                    self.capture_piece(to, captured_piece, them);

                    // if rook has been removed, try to refresh castle
                    if captured_piece == Piece::R {
                        update_castle = true
                    }

                    self.reset_fifty();
                }

                let moving_piece = self.get_piece(from);
                if moving_piece == Piece::P {
                    // check if it's a doublemove
                    if from.abs_diff(to) == 16 {
                        self.set_ep(to ^ 8) // set a new ep square based on flags
                    }
                    self.reset_fifty()
                } else if moving_piece == Piece::K || moving_piece == Piece::R {
                    // if moving piece affects castle rights, update flag
                    update_castle = true
                }

                self.move_piece(from, to, moving_piece, us)
            }
            MoveType::Castle => {
                update_castle = true;
                let (rook_from, rook_to) = get_castling_squares(to);

                self.move_piece(from, to, Piece::K, us);
                self.move_piece(rook_from, rook_to, Piece::R, us);
            }
            MoveType::Promotion => {
                // again, check for capture
                if self.is_color(to, them) {
                    let captured_piece = self.get_piece(to);
                    self.capture_piece(to, captured_piece, them);
                    if captured_piece == Piece::R {
                        update_castle = true
                    }
                }
                self.reset_fifty();
                self.remove_piece(from, Piece::P, us);
                self.add_piece(to, action.pr_piece(), us);
            }
            MoveType::Passant => {
                self.reset_fifty();
                let captured_square = to ^ 8;
                self.remove_piece(captured_square, Piece::P, them);
                self.move_piece(from, to, Piece::P, us);
            }
        }

        if update_castle {
            self.update_castle();
        }
        self.swap_sides();
    }

    pub fn unmake_move(&mut self, action: Action) {
        self.swap_sides();
        let us = self.active_color(); // sides have been swapped so "us" is from previously
        let them = !us;
        let from = action.from();
        let to = action.to();
        let move_type = action.move_type();

        match move_type {
            MoveType::Normal => {
                let moving_piece = self.get_piece(to);
                self.move_piece(to, from, moving_piece, us);
                self.restore_piece(to, them);
            }
            MoveType::Castle => {
                let (rook_from, rook_to) = get_castling_squares(to);
                self.move_piece(to, from, Piece::K, us);
                self.move_piece(rook_to, rook_from, Piece::R, us);
            }
            MoveType::Promotion => {
                self.remove_piece(to, action.pr_piece(), us);
                self.add_piece(from, Piece::P, us);
                self.restore_piece(to, them);
            },
            MoveType::Passant => {
                let captured_square = to ^ 8;
                self.add_piece(captured_square, Piece::P, them);
                self.move_piece(to, from, Piece::P, us);
            },
        }
        self.pop_info()
    }
}
// based on destination squares of castling, get rook from and rook to
fn get_castling_squares(destination: Square) -> (Square, Square) {
    let rook_from: Square;
    let rook_to: Square;

    match destination {
        square::G1 => {
            rook_from = square::H1;
            rook_to = square::F1;
        }
        square::G8 => {
            rook_from = square::H8;
            rook_to = square::F8;
        }
        square::C1 => {
            rook_from = square::A1;
            rook_to = square::D1;
        }
        square::C8 => {
            rook_from = square::A8;
            rook_to = square::D8;
        }
        _ => panic!("Invalid castling"),
    }

    (rook_from, rook_to)
}
