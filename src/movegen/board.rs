use crate::eval::psqt::IncrementalEval;

use super::{
    bitboard::{self, Bitboard},
    types::{square, Color, Piece, Square},
    zobrist::{self, Zobrist},
};

#[derive(Clone, Debug)]
pub struct Board {
    piece_bbs: [Bitboard; 6],
    colors: [Bitboard; 2],
    piece_array: [Piece; 64],
    active_color: Color,
    info: Vec<BoardInfo>,
    evalinfo: IncrementalEval,
}

impl Default for Board {
    fn default() -> Self {
        Self {
            piece_bbs: [0; 6],
            colors: [0; 2],
            piece_array: [Piece::None; 64],
            active_color: Color::W,
            info: vec![BoardInfo::default(); 40],
            evalinfo: IncrementalEval::default(),
        }
    }
}

#[derive(Clone, Copy)]
pub enum Castling {
    WK,
    WQ,
    BK,
    BQ,
}

#[derive(Default, Copy, Clone, Debug)]
pub struct BoardInfo {
    passant_square: Option<Square>,
    captured_piece: Piece,
    halfmove_clock: u8,
    zobrist: Zobrist,
    castling_rights: [bool; 4],
}

impl Board {
    pub fn new() -> Self {
        let mut default = Board::default();
        default
            .parse_fen(&String::from(
                "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1",
            ))
            .unwrap();
        default
    }

    #[inline]
    pub fn evaluate(&self) -> i16 {
        const MULTIPLIERS: [i16; 2] = [1, -1];
        let eval = self.evalinfo.evaluate();
        eval * MULTIPLIERS[self.active_color() as usize]
    }

    pub fn set_evalinfo(&mut self) {
        self.evalinfo = self.generate_eval();
    }

    pub fn is_draw(&self) -> bool {
        // material draw
        let can_force_mate = self.piece_bbs[Piece::P as usize] > 0
            || self.piece_bbs[Piece::R as usize] > 0
            || self.piece_bbs[Piece::Q as usize] > 0
            || self.piece_bb(Piece::B, Color::W).count_ones() >= 2
            || self.piece_bb(Piece::B, Color::B).count_ones() >= 2
            || (self.piece_bb(Piece::B, Color::W).count_ones() >= 1
                && self.piece_bb(Piece::N, Color::W) >= 1)
            || (self.piece_bb(Piece::B, Color::B).count_ones() >= 1
                && self.piece_bb(Piece::N, Color::B) >= 1);

        !can_force_mate || self.halfmove_clock() >= 100
    }

    #[inline]
    fn current_info(&self) -> &BoardInfo {
        self.info.last().unwrap()
    }

    #[inline]
    fn current_info_mut(&mut self) -> &mut BoardInfo {
        self.info.last_mut().unwrap()
    }

    #[inline]
    pub fn is_color(&self, square: Square, color: Color) -> bool {
        bitboard::is_set(self.colors[color as usize], square)
    }

    #[inline]
    pub fn get_piece(&self, square: Square) -> Piece {
        self.piece_array[square as usize]
    }

    #[inline]
    pub fn color_bb(&self, color: Color) -> Bitboard {
        self.colors[color as usize]
    }

    #[inline]
    pub fn piece_bb(&self, piece: Piece, color: Color) -> Bitboard {
        self.piece_bbs[piece as usize] & self.colors[color as usize]
    }

    #[inline]
    pub fn occupancy(&self) -> Bitboard {
        self.colors[Color::W as usize] | self.colors[Color::B as usize]
    }

    #[inline]
    pub fn diagonal_sliders(&self, color: Color) -> Bitboard {
        (self.piece_bbs[Piece::B as usize] | self.piece_bbs[Piece::Q as usize])
            & self.colors[color as usize]
    }

    #[inline]
    pub fn orthogonal_sliders(&self, color: Color) -> Bitboard {
        (self.piece_bbs[Piece::R as usize] | self.piece_bbs[Piece::Q as usize])
            & self.colors[color as usize]
    }

    #[inline]
    pub fn pieces(&self) -> &[Bitboard; 6] {
        &self.piece_bbs
    }

    #[inline]
    pub(super) fn halfmove_clock(&self) -> u8 {
        self.current_info().halfmove_clock
    }

    #[inline]
    pub(super) fn passant_square(&self) -> Option<Square> {
        self.current_info().passant_square
    }

    #[inline]
    pub fn active_color(&self) -> Color {
        self.active_color
    }

    #[inline]
    pub(super) fn castling(&self, castling: Castling) -> &bool {
        &self.current_info().castling_rights[castling as usize]
    }

    #[inline]
    pub(super) fn castling_mut(&mut self, castling: Castling) -> &mut bool {
        &mut self.current_info_mut().castling_rights[castling as usize]
    }

    #[inline]
    pub fn zobrist(&self) -> Zobrist {
        self.current_info().zobrist
    }

    #[inline]
    pub(super) fn zobrist_mut(&mut self) -> &mut Zobrist {
        &mut self.current_info_mut().zobrist
    }

    #[inline]
    pub fn piecetype(&self, piecetype: Piece) -> Bitboard {
        self.piece_bbs[piecetype as usize]
    }

    pub fn is_repetition(&self, count: usize) -> bool {
        let hmc = self.halfmove_clock();
        let zobrist = self.zobrist();
        if hmc < 4 {
            return false;
        }

        let mut num_reps = 0;
        for i in (0..self.info.len())
            .rev()
            .take(hmc as usize + 1)
            .step_by(2)
            .skip(1)
        {
            if self.info[i].zobrist == zobrist {
                num_reps += 1;
                if num_reps >= count {
                    return true;
                }
            }
        }
        false
    }

    #[inline]
    pub fn is_kp(&self) -> bool {
        self.piecetype(Piece::N) == 0
            && self.piecetype(Piece::B) == 0
            && self.piecetype(Piece::R) == 0
            && self.piecetype(Piece::Q) == 0
    }
}

// these methods all involve changing the zobrist hash.
impl Board {
    #[inline]
    pub(super) fn add_piece<const CHANGE_ZOBRIST: bool>(
        &mut self,
        square: Square,
        piece: Piece,
        color: Color,
    ) {
        bitboard::set_bit(&mut self.piece_bbs[piece as usize], square);
        bitboard::set_bit(&mut self.colors[color as usize], square);
        self.piece_array[square as usize] = piece;
        if CHANGE_ZOBRIST {
            *self.zobrist_mut() ^= zobrist::psqt_zobrist(piece, square, color);
        }
        self.evalinfo.set_piece(square, piece, color)
    }

    #[inline]
    pub(super) fn remove_piece<const CHANGE_ZOBRIST: bool>(
        &mut self,
        square: Square,
        piece: Piece,
        color: Color,
    ) {
        bitboard::clear_bit(&mut self.piece_bbs[piece as usize], square);
        bitboard::clear_bit(&mut self.colors[color as usize], square);
        self.piece_array[square as usize] = Piece::None;
        if CHANGE_ZOBRIST {
            *self.zobrist_mut() ^= zobrist::psqt_zobrist(piece, square, color);
        }
        self.evalinfo.remove_piece(square, piece, color)
    }

    #[inline]
    pub(super) fn move_piece<const CHANGE_ZOBRIST: bool>(&mut self, from: Square, to: Square, piece: Piece, color: Color) {
        self.remove_piece::<CHANGE_ZOBRIST>(from, piece, color);
        self.add_piece::<CHANGE_ZOBRIST>(to, piece, color)
    }

    #[inline]
    pub(super) fn swap_sides(&mut self) {
        self.active_color = !self.active_color;
        *self.zobrist_mut() ^= zobrist::turn_zobrist();
    }

    #[inline]
    pub(super) fn set_castling(&mut self, castling: Castling, value: bool) {
        let current_castling = self.current_info().castling_rights[castling as usize];
        *self.castling_mut(castling) = value;

        *self.zobrist_mut() ^= zobrist::castling_zobrist(current_castling, castling);
        *self.zobrist_mut() ^= zobrist::castling_zobrist(value, castling);
    }

    #[inline]
    pub(super) fn set_fifty(&mut self, value: u8) {
        self.current_info_mut().halfmove_clock = value;
    }

    #[inline]
    pub(super) fn reset_fifty(&mut self) {
        self.set_fifty(0)
    }

    #[inline]
    pub(super) fn increment_fifty(&mut self) {
        self.set_fifty(self.halfmove_clock() + 1)
    }

    #[inline]
    fn current_ep_zob(&self) -> Zobrist {
        match self.passant_square() {
            Some(square) => zobrist::passant_zobrist(square),
            None => 0,
        }
    }

    #[inline]
    pub(super) fn set_ep(&mut self, square: Square) {
        *self.zobrist_mut() ^= self.current_ep_zob();
        *self.zobrist_mut() ^= zobrist::passant_zobrist(square);
        self.current_info_mut().passant_square = Some(square)
    }

    #[inline]
    pub(super) fn reset_passant(&mut self) {
        *self.zobrist_mut() ^= self.current_ep_zob();
        self.current_info_mut().passant_square = None
    }

    #[inline]
    pub(super) fn update_castle(&mut self) {
        let white_rooks = self.piece_bb(Piece::R, Color::W);
        let black_rooks = self.piece_bb(Piece::R, Color::B);

        let wk_rook = bitboard::is_set(white_rooks, square::H1 as Square);
        let wq_rook = bitboard::is_set(white_rooks, square::A1 as Square);

        let bk_rook = bitboard::is_set(black_rooks, square::H8 as Square);
        let bq_rook = bitboard::is_set(black_rooks, square::A8 as Square);

        let white_king = self.piece_bb(Piece::K, Color::W);
        let black_king = self.piece_bb(Piece::K, Color::B);

        let w_king = bitboard::is_set(white_king, square::E1 as Square);
        let b_king = bitboard::is_set(black_king, square::E8 as Square);

        self.set_castling(
            Castling::WK,
            *self.castling(Castling::WK) && w_king && wk_rook,
        );
        self.set_castling(
            Castling::WQ,
            *self.castling(Castling::WQ) && w_king && wq_rook,
        );
        self.set_castling(
            Castling::BK,
            *self.castling(Castling::BK) && b_king && bk_rook,
        );
        self.set_castling(
            Castling::BQ,
            *self.castling(Castling::BQ) && b_king && bq_rook,
        );
    }
}

// these methods involve storing and restoring for undoing
impl Board {
    // this is different from "remove_piece." This captures a piece and stores it so it can be restored later.
    #[inline]
    pub(super) fn capture_piece(&mut self, square: Square, piece: Piece, color: Color) {
        self.remove_piece::<true>(square, piece, color);
        self.current_info_mut().captured_piece = piece;
    }

    #[inline]
    pub(super) fn restore_piece(&mut self, square: Square, color: Color) {
        if self.current_info().captured_piece != Piece::None {
            self.add_piece::<false>(square, self.current_info().captured_piece, color)
        }
    }

    #[inline]
    pub(super) fn push_info(&mut self) {
        self.info.push(*self.current_info());
        self.current_info_mut().captured_piece = Piece::None;
    }

    #[inline]
    pub(super) fn pop_info(&mut self) {
        self.info.pop();
        assert!(!self.info.is_empty());
    }
}
