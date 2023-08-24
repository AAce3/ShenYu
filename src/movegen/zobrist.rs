use self::zobrists::ZOBRISTS;

use super::{
    board::Castling,
    types::{square, Color, Piece, Square},
};

pub type Zobrist = u64;

pub fn psqt_zobrist(piece: Piece, square: Square, color: Color) -> Zobrist {
    //ZOBRISTS.piece_squares[(color as usize * 6 + piece as usize) * 64 + square as usize]
    ZOBRISTS.piece_squares[color as usize][piece as usize][square as usize]
}

pub fn passant_zobrist(ep_square: Square) -> Zobrist {
    ZOBRISTS.passant_file[square::file_of(ep_square) as usize]
}

pub fn castling_zobrist(value: bool, castle_dir: Castling) -> Zobrist {
    ZOBRISTS.castling_rights[castle_dir as usize][value as usize]
}

pub fn turn_zobrist() -> Zobrist {
    ZOBRISTS.active_color
}

pub fn halfmove_zobrist(value: u8) -> Zobrist {
    ZOBRISTS.halfmove_clock[value as usize]
}

mod zobrists {
    use std::array;

    use rand::rngs::StdRng;
    use rand::Rng;
    use rand::SeedableRng;
    use static_init::dynamic;

    use crate::movegen::zobrist::Zobrist;

    #[dynamic]
    pub(super) static ZOBRISTS: ZobristContainer = ZobristContainer::new();

    pub(super) struct ZobristContainer {
        pub piece_squares: [[[Zobrist; 64]; 6]; 2],
        pub castling_rights: [[Zobrist; 2]; 4],
        pub passant_file: [Zobrist; 8],
        pub halfmove_clock: [Zobrist; 50],
        pub active_color: Zobrist,
    }

    impl ZobristContainer {
        fn new() -> Self {
            let mut generator: StdRng = SeedableRng::from_seed([125_u8; 32]);
            ZobristContainer {
                piece_squares: array::from_fn(|_| {
                    array::from_fn(|_| array::from_fn(|_| generator.gen()))
                }),
                castling_rights: array::from_fn(|_| array::from_fn(|_| generator.gen())),
                passant_file: array::from_fn(|_| generator.gen()),
                active_color: generator.gen(),
                halfmove_clock: array::from_fn(|_| generator.gen()),
            }
        }
    }
}
