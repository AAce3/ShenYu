use once_cell::sync::Lazy;

use crate::board_state::{bitboard::{Bitboard, BB, Direction::{N, E, S, W, NE, NW, SE, SW}}, typedefs::Square};


pub static KING_ATTACKS: Lazy<[Bitboard; 64]> = Lazy::new(initialize_king_attacks);
pub static KNIGHT_ATTACKS: Lazy<[Bitboard; 64]> = Lazy::new(initialize_knight_attacks);
pub static PAWN_CAPTURES: Lazy<[[Bitboard; 64]; 2]> = Lazy::new(initialize_pawn_attacks);

// Precalculated attack tables for non sliding pieces
fn initialize_king_attacks() -> [Bitboard; 64] {
    let mut arr = [0; 64];
    for (square, val) in arr.iter_mut().enumerate() {
        let bb = Bitboard::new(square as u8);
        *val = bb.shift::<{N}>()
            | bb.shift::<{E}>()
            | bb.shift::<{S}>()
            | bb.shift::<{W}>()
            | bb.shift::<{NE}>()
            | bb.shift::<{SE}>()
            | bb.shift::<{NW}>()
            | bb.shift::<{SW}>();
    }
    arr
}

fn initialize_knight_attacks() -> [Bitboard; 64] {
    let mut arr = [0; 64];
    for (square, val) in arr.iter_mut().enumerate() {
        let bb = Bitboard::new(square as u8);
        *val = bb.shift::<{N}>().shift::<{NE}>()
            | bb.shift::<{N}>().shift::<{NW}>()
            | bb.shift::<{S}>().shift::<{SW}>()
            | bb.shift::<{S}>().shift::<{SE}>()
            | bb.shift::<{E}>().shift::<{NE}>()
            | bb.shift::<{E}>().shift::<{SE}>()
            | bb.shift::<{W}>().shift::<{NW}>()
            | bb.shift::<{W}>().shift::<{SW}>();
    }
    arr
}

fn initialize_pawn_attacks() -> [[Bitboard; 64]; 2] {
    let mut base = [[0; 64]; 2];
    for (color, arr) in base.iter_mut().enumerate() {
        for (square, val) in arr.iter_mut().enumerate() {
            let bb = Bitboard::new(square as u8);
            if color == 0 {
                // white
                *val = bb.shift::<{NW}>() | bb.shift::<{NE}>();
            } else {
                *val = bb.shift::<{SW}>() | bb.shift::<{SE}>();
            }
        }
    }
    base
}

// masking off rook endpoints
#[inline]
pub (crate) fn rook_endpoints(square: Square) -> Bitboard {
    let x_val = square & (7);
    let y_val = square >> 3;
    Bitboard::new(x_val)
        | Bitboard::new(x_val + 56)
        | Bitboard::new(y_val * 8)
        | Bitboard::new((y_val + 1) * 8 - 1)
}

// dumb fill attack masks
#[inline]
pub (crate) fn initialize_rook_atkmask(occupancy: Bitboard, square: Square) -> Bitboard {
    let mut north = Bitboard::new(square);
    let mut east = Bitboard::new(square);
    let mut south = Bitboard::new(square);
    let mut west = Bitboard::new(square);
    let free = !occupancy;
    let mut moves = 0;
    while north | east | south | west != 0 {
        north = north.shift::<{N}>();
        east = east.shift::<{E}>();
        west = west.shift::<{W}>();
        south = south.shift::<{S}>();
        moves |= north | east | south | west;

        north &= free;
        east &= free;
        west &= free;
        south &= free;
    }
    moves
}

#[inline]
pub (crate) fn initialize_bishop_atkmask(occupancy: Bitboard, square: Square) -> Bitboard {
    let mut ne = Bitboard::new(square);
    let mut nw = Bitboard::new(square);
    let mut se = Bitboard::new(square);
    let mut sw = Bitboard::new(square);
    let free = !occupancy;
    let mut moves = 0;
    while ne | nw | se | sw != 0 {
        ne = ne.shift::<{NE}>();
        nw = nw.shift::<{NW}>();
        sw = sw.shift::<{SW}>();
        se = se.shift::<{SE}>();

        moves |= ne | nw | se | sw;

        ne &= free;
        nw &= free;
        sw &= free;
        se &= free;
    }
    moves
}
