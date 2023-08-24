pub const FULL: u64 = u64::MAX;
use super::types::{square, Color, Square};

pub type Bitboard = u64;

pub(crate) const fn new_bb(square: Square) -> Bitboard {
    1 << square
}

pub(super) const fn rank_bb(number: u8) -> Bitboard {
    0xff << (number * 8)
}

pub(super) const fn rank_bb_of(square: Square) -> Bitboard {
    rank_bb(square::rank_of(square))
}

pub(super) const fn file_bb_of(square: Square) -> Bitboard {
    0x101010101010101 << square::file_of(square)
}

pub(crate) const fn diagonal_bb(square: Square) -> Bitboard {
    const MAIN_DIAGONAL: Bitboard = 0x8040201008040201;
    let distance = (square::rank_of(square) as i32) - (square::file_of(square) as i32);
    shl(MAIN_DIAGONAL, distance * 8)
}

pub(crate) const fn antidiagonal_bb(square: Square) -> Bitboard {
    const MAIN_DIAGONAL: Bitboard = 0x102040810204080;
    let distance = (square::rank_of(square) as i32) + (square::file_of(square) as i32) - 7;
    shl(MAIN_DIAGONAL, distance * 8)
}

pub fn pop_bb(bitboard: &mut Bitboard) -> Bitboard {
    let val = *bitboard;
    *bitboard &= *bitboard - 1;
    *bitboard ^ val
}

pub(super) const fn popcount(bitboard: Bitboard) -> u8 {
    bitboard.count_ones() as u8
}

pub(super) const fn lsb(bitboard: Bitboard) -> Square {
    assert!(bitboard != 0);
    bitboard.trailing_zeros() as Square
}

pub fn pop_lsb(bitboard: &mut Bitboard) -> Square {
    let lsb = lsb(*bitboard);
    *bitboard &= *bitboard - 1;
    lsb
}

pub(super) const fn is_set(bitboard: Bitboard, square: Square) -> bool {
    (bitboard >> square) & 1 != 0
}

pub(super) fn set_bit(bitboard: &mut Bitboard, square: Square) {
    *bitboard |= new_bb(square)
}

pub(super) fn clear_bit(bitboard: &mut Bitboard, square: Square) {
    *bitboard &= !new_bb(square)
}

// left shift for integer values. Right shift if it is negative. Wrapping.
const fn shl(value: Bitboard, shift_amount: i32) -> Bitboard {
    if shift_amount >= 0 {
        value << (shift_amount as u32)
    } else {
        value >> (-shift_amount as u32)
    }
}

#[derive(Clone, Copy)]
pub(crate) enum Direction {
    N,
    S,
    E,
    W,
    NE,
    NW,
    SE,
    SW,
}

pub(crate) fn shift(bitboard: Bitboard, direction: Direction) -> Bitboard {
    const A_FILE: Bitboard = file_bb_of(square::A1 as Square);
    const H_FILE: Bitboard = file_bb_of(square::H1 as Square);
    match direction {
        Direction::N => bitboard << 8,
        Direction::S => bitboard >> 8,
        Direction::E => (bitboard << 1) & !A_FILE,
        Direction::W => (bitboard >> 1) & !H_FILE,
        Direction::NE => (bitboard << 9) & !A_FILE,
        Direction::NW => (bitboard << 7) & !H_FILE,
        Direction::SE => (bitboard >> 7) & !A_FILE,
        Direction::SW => (bitboard >> 9) & !H_FILE,
    }
}

pub(crate) fn forward(bitboard: Bitboard, color: Color) -> Bitboard {
    const ROTATE_OFFSET: [i8; 2] = [8, -8];
    let rotate_by = (64 + ROTATE_OFFSET[color as usize]) as u32;
    bitboard.rotate_left(rotate_by)
}
