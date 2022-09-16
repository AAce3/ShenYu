
use once_cell::sync::Lazy;
use rand::{rngs::StdRng, RngCore, SeedableRng};

use super::{
    board::Board,
    typedefs::{Color, Piece, Square, BLACK, NOPIECE, WHITE},
};
// Zobrist hashing "hashes" a board into a 64 bit integer by XORing its various features.
// For each feature on the board, the hash is XORed with a random 64 bit integer to determine hash codes
// Inevitably there will be collisions but those are rare.
pub type ZobristKey = u64;

pub static ZOBRIST: Lazy<Zobrist> = Lazy::new(Zobrist::new);

pub struct Zobrist {
    pub piece_squares: [[[ZobristKey; 6]; 2]; 64],
    pub castling_rights: [ZobristKey; 16],
    pub ep_square: [ZobristKey; 64],
    pub active_color: ZobristKey,
}

const SEED: [u8; 32] = [133; 32];

impl Default for Zobrist {
    fn default() -> Self {
        Self::new()
    }
}

impl Zobrist {
    #[allow(clippy::needless_range_loop)]
    pub fn new() -> Self {
        let mut random = StdRng::from_seed(SEED);
        let mut piecesquares = [[[0; 6]; 2]; 64];
        for square in 0..64 {
            for color in 0..2 {
                for piece in 0..6 {
                    piecesquares[square][color][piece] = random.next_u64();
                }
            }
        }

        let mut castlerights = [0; 16];
        for right in 0..16 {
            castlerights[right] = random.next_u64();
        }

        let mut ep_squares = [0; 64];
        for square in 0..64 {
            ep_squares[square] = random.next_u64();
        }

        let stm = random.next_u64();

        Zobrist {
            piece_squares: piecesquares,
            castling_rights: castlerights,
            ep_square: ep_squares,
            active_color: stm,
        }
    }

    pub fn get_piece_zob(&self, square: Square, color: Color, piece: Piece) -> ZobristKey {
        debug_assert_ne!(piece, NOPIECE);
        let pieceindex = piece - 1;
        self.piece_squares[square as usize][color as usize][pieceindex as usize]
    }
}

impl Board {
    pub fn generate_zobrist(&self) -> ZobristKey {
        let stm_zob = self.get_stm_zob();
        let ep_zob = self.get_ep_zob();
        let castling_zob = self.get_castling_zob();
        let mut psq_zob = 0;
        for square in 0..64 {
            let iswhite = self.is_color(square, WHITE);
            let isblack = self.is_color(square, BLACK);

            if iswhite {
                let piece = self.get_at_square(square);
                let val = ZOBRIST.get_piece_zob(square, WHITE, piece);
                psq_zob ^= val;
            }

            if isblack {
                let piece = self.get_at_square(square);
                let val = ZOBRIST.get_piece_zob(square, BLACK, piece);
                psq_zob ^= val;
            }
        }
        psq_zob ^ ep_zob ^ stm_zob ^ castling_zob
    }

    pub fn get_stm_zob(&self) -> ZobristKey {
        if self.tomove == BLACK {
            0
        } else {
            ZOBRIST.active_color
        }
    }

    pub fn get_ep_zob(&self) -> ZobristKey {
        match self.passant_square {
            None => 0,
            Some(square) => ZOBRIST.ep_square[square as usize],
        }
    }

    pub fn get_castling_zob(&self) -> ZobristKey {
        ZOBRIST.castling_rights[self.castling_rights as usize]
    }
}
