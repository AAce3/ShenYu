use crate::board_state::{
    typedefs::{Piece, Square, BISHOP, KNIGHT, QUEEN, ROOK, SQUARE_NAMES},
};

use super::makemove::PROMOTION;

// Move and Shortmove are two different ways of encoding moves.
// Move has more details, making it simpler. ShortMove is more compact, for TT storage.
// Generics are not necessary to use but it makes things simpler at times. Almost always Move is used.

// Both follow the same pattern: (16 bits to demonstrate)
// 0000 0000 0011 1111 <- Move From
// 0000 1111 1100 0000 <- Move To
// 0011 0000 0000 0000 <- Move Type
// 1100 0000 0000 0000  <- Promotion
// In addition, Move has a few extra details:
// 0000 0000 0000 0000 [0000 0000 0000 0000] Bracketed part is shortmove
// In the upper bits:
// 0000 0000 0000 0111 [0000 0000 0000 0000] <- moving piece
// 0000 0000 0000 1000 [0000 0000 0000 0000] <- capture flag
// 0000 0000 0001 0000 [0000 0000 0000 0000] <- doublemove flag

pub type Move = u32;


pub type MoveType = u8;

const PR_PIECES: [Piece; 4] = [KNIGHT, BISHOP, ROOK, QUEEN];
pub trait Action {
    fn move_from(&self) -> Square;
    fn move_to(&self) -> Square;
    fn move_type(&self) -> MoveType;
    fn promote_to(&self) -> Piece;
    fn piece_moved(&self) -> Piece;
    fn is_capture(&self) -> bool;
    fn is_pawn_doublepush(&self) -> bool;
    fn new_move(from: Square, to: Square, movetype: MoveType, piece: Piece) -> Self;

    fn set_pr_piece(&mut self, piece: Piece);
    fn set_capture(&mut self);
    fn set_doublemove(&mut self);
    fn to_algebraic(&self) -> String{
        let sqrfrom = SQUARE_NAMES[self.move_from() as usize];
        let sqrto = SQUARE_NAMES[self.move_to() as usize];
        let pr_val = if self.move_type() == PROMOTION{
            match self.promote_to(){
                KNIGHT => "n",
                BISHOP => "b",
                ROOK => "r",
                QUEEN => "q",
                _ => panic!("illegal piece")
            }
        } else {
            ""
        };
        let mut base_str = sqrfrom.to_owned();
        base_str += sqrto;
        base_str += pr_val;
        base_str
    }



}

impl Action for Move {
    #[inline]
    fn move_from(&self) -> Square {
        (self & 0b111111) as u8
    }
    #[inline]
    fn move_to(&self) -> Square {
        ((self >> 6) & 0b111111) as u8
    }
    #[inline]
    fn move_type(&self) -> MoveType {
        ((self >> 12) & 0b11) as u8
    }
    #[inline]
    fn promote_to(&self) -> Piece {
        let idx = (self >> 14) & 0b11;
        PR_PIECES[idx as usize]
    }
    #[inline]
    fn piece_moved(&self) -> Piece {
        ((self >> 16) & 0b111) as u8
    }
    #[inline]
    fn is_capture(&self) -> bool {
        ((self >> 19) & 1) != 0
    }
    #[inline]
    fn is_pawn_doublepush(&self) -> bool {
        ((self >> 20) & 1) != 0
    }
    #[inline]
    fn new_move(from: Square, to: Square, movetype: MoveType, moving_piece: Piece) -> Self {
        assert_ne!(moving_piece, 0);
        (from as u32) | ((to as u32) << 6) | ((movetype as u32) << 12) | ((moving_piece as u32) << 16)
    }
    
    #[inline]
    fn set_pr_piece(&mut self, piece: Piece) {
        let pval = (piece - 2) as u32;
        *self |= pval << 14;
    }
    #[inline]
    fn set_capture(&mut self) {
        *self |= 1 << 19;
    }
    #[inline]
    fn set_doublemove(&mut self) {
        *self |= 1 << 20;
    }





}


