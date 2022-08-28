use super::typedefs::{Color, Square};

pub type Bitboard = u64;
pub const NOT_A: Bitboard = 0xfefefefefefefefe;
pub const NOT_H: Bitboard = 0x7f7f7f7f7f7f7f7f;

#[derive(PartialEq, Eq)]
pub enum Direction {
    N,
    E,
    S,
    W,
    NE,
    SE,
    NW,
    SW,
}

pub trait BB {
    fn shift<const D: Direction>(&self) -> Bitboard;
    fn new(sqr: Square) -> Self;
    fn pop_lsb(&mut self) -> Square;
    fn pop_bb(&mut self) -> Bitboard;
    fn lsb(&self) -> Square;
    fn forward(&self, color: Color) -> Bitboard;
}

impl BB for Bitboard {
    #[inline(always)]
    fn new(sqr: Square) -> Self {
        1 << sqr
    }

    #[inline(always)]
    fn pop_lsb(&mut self) -> Square {
        debug_assert_ne!(*self, 0);
        let lsb = self.trailing_zeros();
        *self &= *self - 1;
        lsb as u8
    }

    #[inline(always)]
    fn pop_bb(&mut self) -> Bitboard {
        debug_assert_ne!(*self, 0);
        let val = *self;
        *self &= *self - 1;
        *self ^ val
    }

    #[inline(always)]
    fn lsb(&self) -> Square {
        debug_assert_ne!(*self, 0);
        self.trailing_zeros() as u8
    }

    #[inline(always)]
    fn forward(&self, color: Color) -> Bitboard {
        const ROTATE_OFFSET: [i8; 2] = [8, -8];
        let rotate_by = (64 + ROTATE_OFFSET[color as usize]) as u32;
        self.rotate_left(rotate_by)
    }

    fn shift<const D: Direction>(&self) -> Bitboard {
        match D {
            Direction::N => self << 8,
            Direction::E => (self << 1) & NOT_A,
            Direction::S => self >> 8,
            Direction::W => (self >> 1) & NOT_H,
            Direction::NE => (self << 9) & NOT_A,
            Direction::SE => (self >> 7) & NOT_A,
            Direction::NW => (self << 7) & NOT_H,
            Direction::SW => (self >> 9) & NOT_H,
        }
    }
}
