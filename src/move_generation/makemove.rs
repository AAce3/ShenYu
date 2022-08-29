

use super::action::{Action, MoveType};
use crate::board_state::{
    board::Board,
    typedefs::{Square, KING, PAWN, ROOK},
};
pub static mut MAKEMOVE_TIME: u128 = 0;
pub const NORMAL: MoveType = 0;
pub const CASTLE: MoveType = 1;
pub const PASSANT: MoveType = 2;
pub const PROMOTION: MoveType = 3;

const CASTLING_SQUARES: [CastleData; 64] = initialize_castlevals();
const A1: Square = 0;
pub const C1: Square = 2;
const D1: Square = 3;
const F1: Square = 5;
pub const G1: Square = 6;
const H1: Square = 7;
const A8: Square = 56;
pub const C8: Square = 58;
const D8: Square = 59;
const F8: Square = 61;
pub const G8: Square = 62;
const H8: Square = 63;

const fn initialize_castlevals() -> [CastleData; 64] {
    let mut initval = [CastleData {
        fromsqr: 64,
        tosqr: 64,
    }; 64];
    let w_kingside = CastleData {
        fromsqr: H1,
        tosqr: F1,
    };
    let w_queenside = CastleData {
        fromsqr: A1,
        tosqr: D1,
    };

    let b_kingside = CastleData {
        fromsqr: H8,
        tosqr: F8,
    };
    let b_queenside = CastleData {
        fromsqr: A8,
        tosqr: D8,
    };

    initval[G1 as usize] = w_kingside;
    initval[C1 as usize] = w_queenside;
    initval[G8 as usize] = b_kingside;
    initval[C8 as usize] = b_queenside;
    initval
}

#[derive(Copy, Clone)]
struct CastleData {
    fromsqr: Square,
    tosqr: Square,
}

impl Board {
    pub fn do_move<T: Action>(&self, action: T) -> Board {
        let mut board = *self;
        board.halfmove_clock += 1;
        board.reset_ep();

        let from = action.move_from();
        let to = action.move_to();

        match action.move_type() {
            NORMAL => {
                if action.is_capture(&board) {
                    let capturedpiece = board.get_at_square(to);
                    board.remove_piece(to, capturedpiece, !board.tomove);
                    board.halfmove_clock = 0;
                }
                let moving_piece = action.piece_moved(&board);
                if moving_piece == PAWN {
                    if action.is_pawn_doublepush(&board) {
                        board.set_ep_sqr(to ^ 8);
                    }
                    board.halfmove_clock = 0;
                }

                board.move_piece(from, to, moving_piece, board.tomove);
            }
            CASTLE => {
                let castledata = CASTLING_SQUARES[to as usize];
                board.move_piece(castledata.fromsqr, castledata.tosqr, ROOK, board.tomove);
                board.move_piece(from, to, KING, board.tomove);
            }
            PASSANT => {
                let captured_sqr = to ^ 8;
                board.remove_piece(captured_sqr, PAWN, !board.tomove);
                board.move_piece(from, to, PAWN, board.tomove);
                board.halfmove_clock = 0;
            }
            PROMOTION => {
                if action.is_capture(&board) {
                    let capturedpiece = board.get_at_square(to);
                    board.remove_piece(to, capturedpiece, !board.tomove);
                }
                board.remove_piece(from, PAWN, board.tomove);
                board.set_piece(to, action.promote_to(), board.tomove);
                board.halfmove_clock = 0;
            }
            _ => panic!("Invalid move type"),
        }

        board.swap_sides();
        
        board.set_castling_rights(board.update_castlerights());
        board
    }
}

