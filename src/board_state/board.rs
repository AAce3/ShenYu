use std::ops::{Index, IndexMut};

use crate::{board_state::typedefs::KING, search::evaluate::IncrementalEval};

use super::{
    bitboard::{Bitboard, BB},
    typedefs::{Color, Piece, Square, BISHOP, BLACK, QUEEN, ROOK, WHITE},
    zobrist::{ZobristKey, ZOBRIST},
};
// Bitboards represent a boardstate in terms of a 64 bit integer. 
// A 1 represents the presence of a piece at that square, a 0 represents an absence.
// Board representation is one bitboard for each piece type (PNBRQK) and two color bitboards.

#[derive(Clone, PartialEq, Eq, Debug)]
pub struct Board {
    pub pieces: [Bitboard; 6], // pieces are indexed in this order: Pawn, Knight, Bishop, Rook, Queen, King
    pub colors: [Bitboard; 2],
    pub tomove: Color,
    pub zobrist_key: ZobristKey,
    pub passant_square: Option<Square>,
    pub castling_rights: u8, // KQkq
    pub halfmove_clock: u8,
    pub evaluator: IncrementalEval,
}

impl Default for Board {
    fn default() -> Self {
        Self::new()
    }
}
impl Board {
    pub fn new() -> Self {
        Board {
            pieces: [0; 6],
            colors: [0; 2],
            tomove: WHITE,
            zobrist_key: 0,
            passant_square: None,
            castling_rights: 0,
            halfmove_clock: 0,
            evaluator: IncrementalEval::new()
        }
    }
    #[inline]
    pub fn get_pieces(&self, piecetype: Piece, color: Color) -> Bitboard {
        assert_ne!(piecetype, 0);
        self[piecetype] & self[color]
    }

    #[inline]
    pub fn get_diagonal_sliders(&self, color: Color) -> Bitboard {
        (self[QUEEN] | self[BISHOP]) & self[color]
    }

    #[inline]
    pub fn get_orthogonal_sliders(&self, color: Color) -> Bitboard {
        (self[QUEEN] | self[ROOK]) & self[color]
    }

    #[inline]
    pub fn get_occupancy(&self) -> Bitboard {
        self[WHITE] | self[BLACK]
    }

    #[inline]
    pub fn get_castlerights(&self, color: Color) -> u8 {
        const SHIFTRIGHTS: [u8; 2] = [2, 0];
        self.castling_rights >> SHIFTRIGHTS[color as usize]
    }

    #[inline]
    pub fn get_at_square(&self, square: Square) -> Piece {
        (((self.pieces[0] >> square) & 1)
            + ((self.pieces[1] >> square) & 1) * 2
            + ((self.pieces[2] >> square) & 1) * 3
            + ((self.pieces[3] >> square) & 1) * 4
            + ((self.pieces[4] >> square) & 1) * 5
            + ((self.pieces[5] >> square) & 1) * 6) as u8
    }

    #[inline]
    pub fn is_color(&self, square: Square, color: Color) -> bool {
        let relevant_color = self.colors[color as usize];
        ((relevant_color >> square) & 1) != 0
    }

    #[inline]
    pub fn set_piece(&mut self, square: Square, piecetype: Piece, color: Color) {
        let square_bb = Bitboard::new(square);
        self[piecetype] |= square_bb;
        self[color] |= square_bb;
        let relevant_zobrist = ZOBRIST.get_piece_zob(square, color, piecetype);
        self.zobrist_key ^= relevant_zobrist;
        self.evaluator.set_piece(square, piecetype, color)
    }

    #[inline]
    pub fn move_piece(&mut self, from: Square, to: Square, piecetype: Piece, color: Color) {
        let bbfrom = Bitboard::new(from);
        let bbto = Bitboard::new(to);
        let square_bb = bbfrom | bbto;
        debug_assert!(self[piecetype] & bbfrom != 0);
        debug_assert!(self[color] & bbfrom != 0);
        self[piecetype] ^= square_bb;
        self[color] ^= square_bb;
        let fromzob = ZOBRIST.get_piece_zob(from, color, piecetype);
        let tozob = ZOBRIST.get_piece_zob(to, color, piecetype);
        self.zobrist_key ^= fromzob;
        self.zobrist_key ^= tozob;
        self.evaluator.move_piece(from, to, piecetype, color);
    }

    #[inline]
    pub fn remove_piece(&mut self, square: Square, piecetype: Piece, color: Color) {
        let square_bb = Bitboard::new(square);
        debug_assert!(self[piecetype] & square_bb != 0);
        debug_assert!(self[color] & square_bb != 0);
        self[piecetype] ^= square_bb;
        self[color] ^= square_bb;
        let relevant_zobrist = ZOBRIST.get_piece_zob(square, color, piecetype);
        self.zobrist_key ^= relevant_zobrist;
        self.evaluator.remove_piece(square, piecetype, color)
    }

    #[inline]
    pub fn set_castling_rights(&mut self, castlingrights: u8) {
        let init_zob = ZOBRIST.castling_rights[self.castling_rights as usize];
        let next_zob = ZOBRIST.castling_rights[castlingrights as usize];
        self.zobrist_key ^= init_zob;
        self.zobrist_key ^= next_zob;
        self.castling_rights = castlingrights;
    }

    #[inline]
    pub fn update_castlerights(&self) -> u8 {
        const E1_BB: Bitboard = 0x10;
        const E8_BB: Bitboard = 0x1000000000000000;
        const A1_BB: Bitboard = 1;
        const H1_BB: Bitboard = 0x80;
        const A8_BB: Bitboard = 0x100000000000000;
        const H8_BB: Bitboard = 0x8000000000000000;

        let whiterooks = self.get_pieces(ROOK, WHITE);
        let wq = whiterooks & A1_BB;
        let wk = whiterooks & H1_BB;

        let blackrooks = self.get_pieces(ROOK, BLACK);
        let bq = blackrooks & A8_BB;
        let bk = blackrooks & H8_BB;

        let w_king = self.get_pieces(KING, WHITE) & E1_BB;
        let b_king = self.get_pieces(KING, BLACK) & E8_BB;

        let mut rookrights = (wq << 2) | (wk >> 4) | (bk >> 62) | (bq >> 56);

        let wkingrights = (w_king >> 1) | (w_king >> 2);

        let bkingrights = (b_king >> 59) | (b_king >> 60);

        rookrights &= wkingrights | bkingrights;

        (rookrights as u8) & self.castling_rights
    }

    // doesn't clear ep square
    #[inline]
    pub fn set_ep_sqr(&mut self, sqr: Square) {
        self.zobrist_key ^= ZOBRIST.ep_square[sqr as usize];
        self.passant_square = Some(sqr);
    }

    #[inline]
    pub fn reset_ep(&mut self) {
        let curr_ep_zob = match self.passant_square {
            None => 0,
            Some(square) => ZOBRIST.ep_square[square as usize],
        };

        self.zobrist_key ^= curr_ep_zob;
        self.passant_square = None;
    }

    #[inline]
    pub fn swap_sides(&mut self) {
        self.tomove = !self.tomove;
        self.zobrist_key ^= ZOBRIST.active_color;
    }

    #[inline]
    pub fn is_empty(&self, square: Square) -> bool{
        self.get_occupancy() & Bitboard::new(square) == 0
    }
}

impl IndexMut<Piece> for Board {
    fn index_mut(&mut self, index: Piece) -> &mut Self::Output {
        assert_ne!(index, 0);
        &mut self.pieces[(index - 1) as usize]
    }
}

impl IndexMut<Color> for Board {
    fn index_mut(&mut self, index: Color) -> &mut Self::Output {
        &mut self.colors[index as usize]
    }
}
impl Index<Piece> for Board {
    type Output = Bitboard;

    fn index(&self, index: Piece) -> &Self::Output {
        assert_ne!(index, 0);
        &self.pieces[(index - 1) as usize]
    }
}

impl Index<Color> for Board {
    type Output = Bitboard;

    fn index(&self, index: Color) -> &Self::Output {
        &self.colors[index as usize]
    }
}
