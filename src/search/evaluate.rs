use std::cmp;

use crate::board_state::{
    bitboard::BB,
    board::Board,
    typedefs::{Color, Piece, Sq, Square, BISHOP, BLACK, KNIGHT, PAWN, QUEEN, ROOK, WHITE},
};

// pesto psqts
const PAWN_PHASE: i16 = 0;
const KNIGHT_PHASE: i16 = 1;
const BISHOP_PHASE: i16 = 1;
const ROOK_PHASE: i16 = 2;
const QUEEN_PHASE: i16 = 4;
const TOTAL_PHASE: i16 = 24;
pub const PHASES: [i16; 6] = [
    PAWN_PHASE,
    KNIGHT_PHASE,
    BISHOP_PHASE,
    ROOK_PHASE,
    QUEEN_PHASE,
    0,
];
#[rustfmt::skip]
const MG_PAWN: [i16; 64] = [
    82, 82, 82, 82, 82, 82, 82, 82,
    180, 216, 143, 177, 150, 208, 116, 71,
    76, 89, 108, 113, 147, 138, 107, 62,
    68, 95, 88, 103, 105, 94, 99, 59,
    55, 80, 77, 94, 99, 88, 92, 57,
    56, 78, 78, 72, 85, 85, 115, 70,
    47, 81, 62, 59, 67, 106, 120, 60,
    82, 82, 82, 82, 82, 82, 82, 82,
];
#[rustfmt::skip]
const MG_KNIGHT: [i16; 64] = [
    170, 248, 303, 288, 398, 240, 322, 230,
    264, 296, 409, 373, 360, 399, 344, 320,
    290, 397, 374, 402, 421, 466, 410, 381,
    328, 354, 356, 390, 374, 406, 355, 359,
    324, 341, 353, 350, 365, 356, 358, 329,
    314, 328, 349, 347, 356, 354, 362, 321,
    308, 284, 325, 334, 336, 355, 323, 318,
    232, 316, 279, 304, 320, 309, 318, 314,
];
#[rustfmt::skip]
const MG_BISHOP: [i16; 64] = [
    336, 369, 283, 328, 340, 323, 372, 357,
    339, 381, 347, 352, 395, 424, 383, 318,
    349, 402, 408, 405, 400, 415, 402, 363,
    361, 370, 384, 415, 402, 402, 372, 363,
    359, 378, 378, 391, 399, 377, 375, 369,
    365, 380, 380, 380, 379, 392, 383, 375,
    369, 380, 381, 365, 372, 386, 398, 366,
    332, 362, 351, 344, 352, 353, 326, 344,
];
#[rustfmt::skip]
const MG_ROOK: [i16; 64] = [
    509, 519, 509, 528, 540, 486, 508, 520,
    504, 509, 535, 539, 557, 544, 503, 521,
    472, 496, 503, 513, 494, 522, 538, 493,
    453, 466, 484, 503, 501, 512, 469, 457,
    441, 451, 465, 476, 486, 470, 483, 454,
    432, 452, 461, 460, 480, 477, 472, 444,
    433, 461, 457, 468, 476, 488, 471, 406,
    458, 464, 478, 494, 493, 484, 440, 451,
];
#[rustfmt::skip]
const MG_QUEEN: [i16; 64] = [
    997, 1025, 1054, 1037, 1084, 1069, 1068, 1070,
    1001, 986, 1020, 1026, 1009, 1082, 1053, 1079,
    1012, 1008, 1032, 1033, 1054, 1081, 1072, 1082,
    998, 998, 1009, 1009, 1024, 1042, 1023, 1026,
    1016, 999, 1016, 1015, 1023, 1021, 1028, 1022,
    1011, 1027, 1014, 1023, 1020, 1027, 1039, 1030,
    990, 1017, 1036, 1027, 1033, 1040, 1022, 1026,
    1024, 1007, 1016, 1035, 1010, 1000, 994, 975,
];
#[rustfmt::skip]
const MG_KING: [i16; 64] = [
    -65, 23, 16, -15, -56, -34, 2, 13,
    29, -1, -20, -7, -8, -4, -38, -29,
    -9, 24, 2, -16, -20, 6, 22, -22,
    -17, -20, -12, -27, -30, -25, -14, -36,
    -49, -1, -27, -39, -46, -44, -33, -51,
    -14, -14, -22, -46, -44, -30, -15, -27,
    1, 7, -8, -64, -43, -16, 9, 8,
    -15, 36, 12, -54, 8, -28, 24, 14,
];

#[rustfmt::skip]
const EG_PAWN: [i16; 64] = [
    94, 94, 94, 94, 94, 94, 94, 94,
    272, 267, 252, 228, 241, 226, 259, 281,
    188, 194, 179, 161, 150, 147, 176, 178,
    126, 118, 107, 99, 92, 98, 111, 111,
    107, 103, 91, 87, 87, 86, 97, 93,
    98, 101, 88, 95, 94, 89, 93, 86,
    107, 102, 102, 104, 107, 94, 96, 87,
    94, 94, 94, 94, 94, 94, 94, 94,
];

#[rustfmt::skip]
const EG_KNIGHT: [i16; 64] = [
    223, 243, 268, 253, 250, 254, 218, 182,
    256, 273, 256, 279, 272, 256, 257, 229,
    257, 261, 291, 290, 280, 272, 262, 240,
    264, 284, 303, 303, 303, 292, 289, 263,
    263, 275, 297, 306, 297, 298, 285, 263,
    258, 278, 280, 296, 291, 278, 261, 259,
    239, 261, 271, 276, 279, 261, 258, 237,
    252, 230, 258, 266, 259, 263, 231, 217,
];

#[rustfmt::skip]
const EG_BISHOP: [i16; 64] = [
    283, 276, 286, 289, 290, 288, 280, 273,
    289, 293, 304, 285, 294, 284, 293, 283,
    299, 289, 297, 296, 295, 303, 297, 301,
    294, 306, 309, 306, 311, 307, 300, 299,
    291, 300, 310, 316, 304, 307, 294, 288,
    285, 294, 305, 307, 310, 300, 290, 282,
    283, 279, 290, 296, 301, 288, 282, 270,
    274, 288, 274, 292, 288, 281, 292, 280,
];

#[rustfmt::skip]
const EG_ROOK: [i16; 64] = [
    525, 522, 530, 527, 524, 524, 520, 517,
    523, 525, 525, 523, 509, 515, 520, 515,
    519, 519, 519, 517, 516, 509, 507, 509,
    516, 515, 525, 513, 514, 513, 511, 514,
    515, 517, 520, 516, 507, 506, 504, 501,
    508, 512, 507, 511, 505, 500, 504, 496,
    506, 506, 512, 514, 503, 503, 501, 509,
    503, 514, 515, 511, 507, 499, 516, 492,
];

#[rustfmt::skip]
const EG_QUEEN: [i16; 64] = [
    927, 958, 958, 963, 963, 955, 946, 956,
    919, 956, 968, 977, 994, 961, 966, 936,
    916, 942, 945, 985, 983, 971, 955, 945,
    939, 958, 960, 981, 993, 976, 993, 972,
    918, 964, 955, 983, 967, 970, 975, 959,
    920, 909, 951, 942, 945, 953, 946, 941,
    914, 913, 906, 920, 920, 913, 900, 904,
    903, 908, 914, 893, 931, 904, 916, 895,
];

#[rustfmt::skip]
const EG_KING: [i16; 64] = [
    -74, -35, -18, -18, -11, 15, 4, -17,
    -12, 17, 14, 17, 17, 38, 23, 11,
    10, 17, 23, 15, 20, 45, 44, 13,
    -8, 22, 24, 27, 26, 33, 26, 3,
    -18, -4, 21, 24, 27, 23, 9, -11,
    -19, -3, 11, 21, 23, 16, 7, -9,
    -27, -11, 4, 13, 14, 4, -5, -17,
    -53, -34, -21, -11, -28, -14, -24, -43,
];

const MG_TABLES: [[i16; 64]; 6] = [MG_PAWN, MG_KNIGHT, MG_BISHOP, MG_ROOK, MG_QUEEN, MG_KING];

const EG_TABLES: [[i16; 64]; 6] = [EG_PAWN, EG_KNIGHT, EG_BISHOP, EG_ROOK, EG_QUEEN, EG_KING];

// Uses PeSTO tables for evaluation.
// Tapered evaluation forms a gradient between middle game and endgame evaluation parameters.
// e.g. Keep the king safe during middlegame, develop it during endgame.
// Phase is calculated by material

impl Board {
    pub fn do_eval(&self) -> i16 {
        let mut white_mg_material = 0;
        let mut black_mg_material = 0;
        let mut white_eg_material = 0;
        let mut black_eg_material = 0;

        let mut phase = 0;
        let whites = self[WHITE];
        let blacks = self[BLACK];
        for (piecetype, bb) in self.pieces.iter().enumerate() {
            let mut white_pieces = bb & whites;
            let mut black_pieces = bb & blacks;
            while white_pieces > 0 {
                let square = white_pieces.pop_lsb().flip();
                let mg_value = MG_TABLES[piecetype as usize][square as usize];
                let eg_value = EG_TABLES[piecetype as usize][square as usize];
                white_mg_material += mg_value;
                white_eg_material += eg_value;
                phase += PHASES[piecetype as usize];
            }
            while black_pieces > 0 {
                let square = black_pieces.pop_lsb();
                let mg_value = MG_TABLES[piecetype as usize][square as usize];
                let eg_value = EG_TABLES[piecetype as usize][square as usize];
                black_mg_material += mg_value;
                black_eg_material += eg_value;
                phase += PHASES[piecetype as usize]; 
            }
        }

        let mg_score = white_mg_material as i32 - black_mg_material as i32;
        let eg_score = white_eg_material as i32 - black_eg_material as i32;
        let mg_phase = cmp::min(phase, TOTAL_PHASE) as i32;
        let eg_phase = cmp::max(TOTAL_PHASE as i32 - mg_phase, 0) as i32;
        (((mg_score * mg_phase).checked_add(eg_score * eg_phase).unwrap()) / (TOTAL_PHASE as i32)) as i16
    }
    pub fn generate_eval(&self) -> IncrementalEval {
        let mut white_mg_material = 0;
        let mut black_mg_material = 0;
        let mut white_eg_material = 0;
        let mut black_eg_material = 0;

        let mut phase = 0;
        let whites = self[WHITE];
        let blacks = self[BLACK];
        for (piecetype, bb) in self.pieces.iter().enumerate() {
            let mut white_pieces = bb & whites;
            let mut black_pieces = bb & blacks;
            while white_pieces > 0 {
                let square = white_pieces.pop_lsb().flip();
                let mg_value = MG_TABLES[piecetype as usize][square as usize];
                let eg_value = EG_TABLES[piecetype as usize][square as usize];
                white_mg_material += mg_value;
                white_eg_material += eg_value;
                phase += PHASES[piecetype as usize];
            }
            while black_pieces > 0 {
                let square = black_pieces.pop_lsb();
                let mg_value = MG_TABLES[piecetype as usize][square as usize];
                let eg_value = EG_TABLES[piecetype as usize][square as usize];
                black_mg_material += mg_value;
                black_eg_material += eg_value;
                phase += PHASES[piecetype as usize];
            }
        }
        IncrementalEval {
            phase,
            mg_material: [white_mg_material, black_mg_material],
            eg_material: [white_eg_material, black_eg_material],
        }
    }

    pub fn evaluate(&self) -> i16 {
        const MULTIPLIERS: [i16; 2] = [1, -1];
        let eval = self.evaluator.evaluate();
        eval * MULTIPLIERS[self.tomove as usize]
    }

    // material only eval for debugging
    pub fn beancount(&self) -> i16 {
        const VALUES: [i16; 6] = [100, 315, 325, 500, 900, 0];
        let mut value = 0;
        let wpieces = self[WHITE];
        let bpieces = self[BLACK];
        for (idx, piece) in self.pieces.iter().enumerate() {
            let w = wpieces & piece;
            let b = bpieces & piece;
            value += w.count_ones() as i16 * VALUES[idx];
            value -= b.count_ones() as i16 * VALUES[idx];
        }
        value
    }
    pub fn is_draw(&self) -> bool {
        // material draw
        let can_force_mate = self[PAWN] > 0
            || self[ROOK] > 0
            || self[QUEEN] > 0
            || self.get_pieces(BISHOP, WHITE).count_ones() >= 2
            || self.get_pieces(BISHOP, BLACK).count_ones() >= 2
            || (self.get_pieces(BISHOP, WHITE).count_ones() >= 1
                && self.get_pieces(KNIGHT, WHITE) >= 1)
            || (self.get_pieces(BISHOP, BLACK).count_ones() >= 1
                && self.get_pieces(KNIGHT, BLACK) >= 1);
        !can_force_mate || self.halfmove_clock >= 100
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct IncrementalEval {
    pub phase: i16,
    pub mg_material: [i16; 2],
    pub eg_material: [i16; 2],
}

impl Default for IncrementalEval {
    fn default() -> Self {
        Self::new()
    }
}

const FLIPS: [u8; 2] = [56, 0];
impl IncrementalEval {
    pub fn new() -> Self {
        Self {
            phase: TOTAL_PHASE,
            mg_material: [0, 0],
            eg_material: [0, 0],
        }
    }
    #[inline]
    pub fn set_piece(&mut self, square: Square, piecetype: Piece, color: Color) {
        self.phase += PHASES[piecetype as usize - 1];
        let square = square ^ FLIPS[color as usize];
        let mg_value = MG_TABLES[piecetype as usize - 1][square as usize];
        let eg_value = EG_TABLES[piecetype as usize - 1][square as usize];
        self.mg_material[color as usize] += mg_value;
        self.eg_material[color as usize] += eg_value;
    }

    #[inline]
    pub fn remove_piece(&mut self, square: Square, piecetype: Piece, color: Color) {
        self.phase -= PHASES[piecetype as usize - 1];
        let square = square ^ FLIPS[color as usize];
        let mg_value = MG_TABLES[piecetype as usize - 1][square as usize];
        let eg_value = EG_TABLES[piecetype as usize - 1][square as usize];
        self.mg_material[color as usize] -= mg_value;
        self.eg_material[color as usize] -= eg_value;
    }

    #[inline]
    pub fn move_piece(&mut self, from: Square, to: Square, piecetype: Piece, color: Color) {
        self.remove_piece(from, piecetype, color);
        self.set_piece(to, piecetype, color)
    }

    #[inline]
    pub fn evaluate(&self) -> i16 {
        let mg_score = self.mg_material[0] as i32 - self.mg_material[1] as i32;
        let eg_score = self.eg_material[0] as i32 - self.eg_material[1] as i32;
        let mg_phase = cmp::min(self.phase, TOTAL_PHASE) as i32;
        let eg_phase = cmp::max(TOTAL_PHASE as i32 - mg_phase, 0);
        (((mg_score * mg_phase) + (eg_score * eg_phase)) / (TOTAL_PHASE as i32)) as i16
    }
}
