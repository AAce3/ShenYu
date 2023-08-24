use std::{fmt, mem, ops};

#[derive(Default, Copy, Clone, PartialEq, Eq, Debug)]
pub enum Piece {
    P,
    N,
    B,
    R,
    Q,
    K,
    #[default]
    None,
}

impl From<u8> for Piece {
    fn from(value: u8) -> Self {
        unsafe { mem::transmute::<u8, Piece>(value) }
    }
}

impl Piece {
    pub fn name(&self) -> char {
        const PIECE_NAMES: [char; 7] = ['p', 'n', 'b', 'r', 'q', 'k', '.'];
        PIECE_NAMES[*self as usize]
    }
}

#[derive(PartialEq, Eq, Default, Copy, Clone, Debug)]
pub enum Color {
    #[default]
    W,
    B,
}

impl From<u8> for Color {
    fn from(value: u8) -> Self {
        unsafe { mem::transmute::<u8, Color>(value) }
    }
}

impl ops::Not for Color {
    type Output = Color;

    fn not(self) -> Self::Output {
        let bit_bool = self as u8 != 0; // bitwise cast to bool
        Color::from((!bit_bool) as u8)
    }
}

impl fmt::Display for Color {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let name = match self {
            Color::W => "White",
            Color::B => "Black",
        };
        f.write_str(name)?;
        Ok(())
    }
}
pub type Square = u8;

#[allow(dead_code)]
pub mod square {
    use crate::movegen::bitboard::Direction;

    use super::Square;

    pub const fn rank_of(square: Square) -> u8 {
        square / 8
    }

    pub const fn file_of(square: Square) -> u8 {
        square % 8
    }

    pub const fn new_sq(rank: u8, file: u8) -> Square {
        rank * 8 + file
    }

    pub const fn flip_v(square: Square) -> Square {
        square ^ 56
    }

    pub const fn flip_h(square: Square) -> Square {
        square ^ 7
    }

    pub(in super::super) const fn shift(square: Square, direction: Direction) -> Square {
        match direction {
            Direction::N => square + 8,
            Direction::S => square - 8,
            Direction::E => square + 1,
            Direction::W => square - 1,
            Direction::NE => square + 9,
            Direction::NW => square + 7,
            Direction::SE => square - 7,
            Direction::SW => square - 9,
        }
    }

    pub fn from_algebraic(string: &str) -> Option<Square> {
        let bytes = string.as_bytes();
        if bytes.len() == 2 {
            let file_name = bytes[0];
            let rank_name = bytes[1];
            if (b'a'..=b'h').contains(&file_name) && (b'1'..=b'8').contains(&rank_name) {
                let file = file_name - b'a';
                let rank = rank_name - b'1';
                return Some(new_sq(rank, file));
            }
        }
        None
    }

    pub const fn name(square: Square) -> &'static str {
        #[rustfmt::skip]
        const SQUARE_NAMES: [&str; 64] = [
            "a1", "b1", "c1", "d1", "e1", "f1", "g1", "h1",
            "a2", "b2", "c2", "d2", "e2", "f2", "g2", "h2",
            "a3", "b3", "c3", "d3", "e3", "f3", "g3", "h3",
            "a4", "b4", "c4", "d4", "e4", "f4", "g4", "h4",
            "a5", "b5", "c5", "d5", "e5", "f5", "g5", "h5",
            "a6", "b6", "c6", "d6", "e6", "f6", "g6", "h6",
            "a7", "b7", "c7", "d7", "e7", "f7", "g7", "h7",
            "a8", "b8", "c8", "d8", "e8", "f8", "g8", "h8"
        ];

        SQUARE_NAMES[square as usize]
    }


    pub const A1: Square = 0;
    pub const B1: Square = 1;
    pub const C1: Square = 2;
    pub const D1: Square = 3;
    pub const E1: Square = 4;
    pub const F1: Square = 5;
    pub const G1: Square = 6;
    pub const H1: Square = 7;
    pub const A2: Square = 8;
    pub const B2: Square = 9;
    pub const C2: Square = 10;
    pub const D2: Square = 11;
    pub const E2: Square = 12;
    pub const F2: Square = 13;
    pub const G2: Square = 14;
    pub const H2: Square = 15;
    pub const A3: Square = 16;
    pub const B3: Square = 17;
    pub const C3: Square = 18;
    pub const D3: Square = 19;
    pub const E3: Square = 20;
    pub const F3: Square = 21;
    pub const G3: Square = 22;
    pub const H3: Square = 23;
    pub const A4: Square = 24;
    pub const B4: Square = 25;
    pub const C4: Square = 26;
    pub const D4: Square = 27;
    pub const E4: Square = 28;
    pub const F4: Square = 29;
    pub const G4: Square = 30;
    pub const H4: Square = 31;
    pub const A5: Square = 32;
    pub const B5: Square = 33;
    pub const C5: Square = 34;
    pub const D5: Square = 35;
    pub const E5: Square = 36;
    pub const F5: Square = 37;
    pub const G5: Square = 38;
    pub const H5: Square = 39;
    pub const A6: Square = 40;
    pub const B6: Square = 41;
    pub const C6: Square = 42;
    pub const D6: Square = 43;
    pub const E6: Square = 44;
    pub const F6: Square = 45;
    pub const G6: Square = 46;
    pub const H6: Square = 47;
    pub const A7: Square = 48;
    pub const B7: Square = 49;
    pub const C7: Square = 50;
    pub const D7: Square = 51;
    pub const E7: Square = 52;
    pub const F7: Square = 53;
    pub const G7: Square = 54;
    pub const H7: Square = 55;
    pub const A8: Square = 56;
    pub const B8: Square = 57;
    pub const C8: Square = 58;
    pub const D8: Square = 59;
    pub const E8: Square = 60;
    pub const F8: Square = 61;
    pub const G8: Square = 62;
    pub const H8: Square = 63;
}
