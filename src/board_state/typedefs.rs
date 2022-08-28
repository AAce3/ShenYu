pub type Piece = u8;
pub const NOPIECE: u8 = 0;
pub const PAWN: u8 = 1;
pub const KNIGHT: u8 = 2;
pub const BISHOP: u8 = 3;
pub const ROOK: u8 = 4;
pub const QUEEN: u8 = 5;
pub const KING: u8 = 6;

pub type Color = bool;
pub const WHITE: Color = false;
pub const BLACK: Color = true;

pub type Square = u8;
#[rustfmt::skip]
pub const SQUARE_NAMES: [&str; 64] = [
    "a1", "b1", "c1", "d1", "e1", "f1", "g1", "h1",
    "a2", "b2", "c2", "d2", "e2", "f2", "g2", "h2",
    "a3", "b3", "c3", "d3", "e3", "f3", "g3", "h3",
    "a4", "b4", "c4", "d4", "e4", "f4", "g4", "h4",
    "a5", "b5", "c5", "d5", "e5", "f5", "g5", "h5",
    "a6", "b6", "c6", "d6", "e6", "f6", "g6", "h6",
    "a7", "b7", "c7", "d7", "e7", "f7", "g7", "h7",
    "a8", "b8", "c8", "d8", "e8", "f8", "g8", "h8"
];

pub trait Sq {
    fn flip(&self) -> Self;
    fn algebraic_to_sqr(squarename: &str) -> Option<Square>;
}

impl Sq for Square {
    fn flip(&self) -> Self {
        *self ^ 56
    }

    fn algebraic_to_sqr(square: &str) -> Option<Square> {
        let mut chariter = square.chars();
        let file = match chariter.next()? {
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
        let rank = chariter.next()?;
        let rank = char::to_digit(rank, 10)? as u8;
        if !(1..=8).contains(&rank) {
            return None;
        }
        Some((rank - 1) * 8 + file)
    }
}

