use once_cell::sync::Lazy;

use crate::board_state::{
    bitboard::BB,
    board::Board,
    typedefs::{Color, Piece, Sq, Square, BISHOP, BLACK, KNIGHT, PAWN, QUEEN, ROOK, WHITE},
};



// pesto psqts
const PAWN_PHASE: u16 = 0;
const KNIGHT_PHASE: u16 = 1;
const BISHOP_PHASE: u16 = 1;
const ROOK_PHASE: u16 = 2;
const QUEEN_PHASE: u16 = 4;
const TOTAL_PHASE: u16 =
    PAWN_PHASE * 16 + KNIGHT_PHASE * 4 + BISHOP_PHASE * 4 + ROOK_PHASE * 4 + QUEEN_PHASE * 2;
pub const PHASES: [u16; 6] = [
    PAWN_PHASE,
    KNIGHT_PHASE,
    BISHOP_PHASE,
    ROOK_PHASE,
    QUEEN_PHASE,
    0,
];
const MIDDLEGAME_PAWN: i16 = 82;
const MIDDLEGAME_KNIGHT: i16 = 337;
const MIDDLEGAME_BISHOP: i16 = 365;
const MIDDLEGAME_ROOK: i16 = 477;
const MIDDLEGAME_QUEEN: i16 = 1025;

const ENDGAME_PAWN: i16 = 94;
const ENDGAME_KNIGHT: i16 = 281;
const ENDGAME_BISHOP: i16 = 297;
const ENDGAME_ROOK: i16 = 512;
const ENDGAME_QUEEN: i16 = 936;

#[rustfmt::skip]
static MIDDLEGAME_PAWNTABLE: [i16; 64] = [
    0,   0,   0,   0,   0,   0,  0,   0,
    98, 134,  61,  95,  68, 126, 34, -11,
    -6,   7,  26,  31,  65,  56, 25, -20,
   -14,  13,   6,  21,  23,  12, 17, -23,
   -27,  -2,  -5,  12,  17,   6, 10, -25,
   -26,  -4,  -4, -10,   3,   3, 33, -12,
   -35,  -1, -20, -23, -15,  24, 38, -22,
     0,   0,   0,   0,   0,   0,  0,   0,
];

#[rustfmt::skip]
static ENDGAME_PAWNTABLE: [i16; 64] = [
    0,   0,   0,   0,   0,   0,   0,   0,
    178, 173, 158, 134, 147, 132, 165, 187,
    94, 100,  85,  67,  56,  53,  82,  84,
    32,  24,  13,   5,  -2,   4,  17,  17,
    13,   9,  -3,  -7,  -7,  -8,   3,  -1,
    4,   7,  -6,   1,   0,  -5,  -1,  -8,
    13,   8,   8,  10,  13,   0,   2,  -7,
    0,   0,   0,   0,   0,   0,   0,   0,
];

#[rustfmt::skip]
static MIDDLEGAME_KNIGHTTABLE: [i16; 64] = [
    -167, -89, -34, -49,  61, -97, -15, -107,
     -73, -41,  72,  36,  23,  62,   7,  -17,
     -47,  60,  37,  65,  84, 129,  73,   44,
      -9,  17,  19,  53,  37,  69,  18,   22,
     -13,   4,  16,  13,  28,  19,  21,   -8,
     -23,  -9,  12,  10,  19,  17,  25,  -16,
     -29, -53, -12,  -3,  -1,  18, -14,  -19,
    -105, -21, -58, -33, -17, -28, -19,  -23,
];

#[rustfmt::skip]
static ENDGAME_KNIGHTTABLE: [i16; 64] = [
    -58, -38, -13, -28, -31, -27, -63, -99,
    -25,  -8, -25,  -2,  -9, -25, -24, -52,
    -24, -20,  10,   9,  -1,  -9, -19, -41,
    -17,   3,  22,  22,  22,  11,   8, -18,
    -18,  -6,  16,  25,  16,  17,   4, -18,
    -23,  -3,  -1,  15,  10,  -3, -20, -22,
    -42, -20, -10,  -5,  -2, -20, -23, -44,
    -29, -51, -23, -15, -22, -18, -50, -64,
];

#[rustfmt::skip]
static MIDDLEGAME_BISHOPTABLE: [i16; 64] = [
    -29,   4, -82, -37, -25, -42,   7,  -8,
    -26,  16, -18, -13,  30,  59,  18, -47,
    -16,  37,  43,  40,  35,  50,  37,  -2,
     -4,   5,  19,  50,  37,  37,   7,  -2,
     -6,  13,  13,  26,  34,  12,  10,   4,
      0,  15,  15,  15,  14,  27,  18,  10,
      4,  15,  16,   0,   7,  21,  33,   1,
    -33,  -3, -14, -21, -13, -12, -39, -21,
];

#[rustfmt::skip]
static ENDGAME_BISHOPTABLE: [i16; 64] = [
    -14, -21, -11,  -8, -7,  -9, -17, -24,
    -8,  -4,   7, -12, -3, -13,  -4, -14,
     2,  -8,   0,  -1, -2,   6,   0,   4,
    -3,   9,  12,   9, 14,  10,   3,   2,
    -6,   3,  13,  19,  7,  10,  -3,  -9,
   -12,  -3,   8,  10, 13,   3,  -7, -15,
   -14, -18,  -7,  -1,  4,  -9, -15, -27,
   -23,  -9, -23,  -5, -9, -16,  -5, -17,
];

#[rustfmt::skip]
static MIDDLEGAME_ROOKTABLE: [i16; 64] = [
    32,  42,  32,  51, 63,  9,  31,  43,
    27,  32,  58,  62, 80, 67,  26,  44,
    -5,  19,  26,  36, 17, 45,  61,  16,
   -24, -11,   7,  26, 24, 35,  -8, -20,
   -36, -26, -12,  -1,  9, -7,   6, -23,
   -45, -25, -16, -17,  3,  0,  -5, -33,
   -44, -16, -20,  -9, -1, 11,  -6, -71,
   -19, -13,   1,  17, 16,  7, -37, -26,
];

#[rustfmt::skip]
static ENDGAME_ROOKTABLE: [i16; 64] = [
    13, 10, 18, 15, 12,  12,   8,   5,
    11, 13, 13, 11, -3,   3,   8,   3,
     7,  7,  7,  5,  4,  -3,  -5,  -3,
     4,  3, 13,  1,  2,   1,  -1,   2,
     3,  5,  8,  4, -5,  -6,  -8, -11,
    -4,  0, -5, -1, -7, -12,  -8, -16,
    -6, -6,  0,  2, -9,  -9, -11,  -3,
    -9,  2,  3, -1, -5, -13,   4, -20,
];

#[rustfmt::skip]
static MIDDLEGAME_QUEENTABLE: [i16; 64] = [
    -28,   0,  29,  12,  59,  44,  43,  45,
    -24, -39,  -5,   1, -16,  57,  28,  54,
    -13, -17,   7,   8,  29,  56,  47,  57,
    -27, -27, -16, -16,  -1,  17,  -2,   1,
     -9, -26,  -9, -10,  -2,  -4,   3,  -3,
    -14,   2, -11,  -2,  -5,   2,  14,   5,
    -35,  -8,  11,   2,   8,  15,  -3,   1,
     -1, -18,  -9,  10, -15, -25, -31, -50,
];

#[rustfmt::skip]
static ENDGAME_QUEENTABLE: [i16; 64] = [
    -9,  22,  22,  27,  27,  19,  10,  20,
    -17,  20,  32,  41,  58,  25,  30,   0,
    -20,   6,   9,  49,  47,  35,  19,   9,
      3,  22,  24,  45,  57,  40,  57,  36,
    -18,  28,  19,  47,  31,  34,  39,  23,
    -16, -27,  15,   6,   9,  17,  10,   5,
    -22, -23, -30, -16, -16, -23, -36, -32,
    -33, -28, -22, -43,  -5, -32, -20, -41,
];

#[rustfmt::skip]
static MIDDLEGAME_KINGTABLE: [i16; 64] = [
    -65,  23,  16, -15, -56, -34,   2,  13,
     29,  -1, -20,  -7,  -8,  -4, -38, -29,
     -9,  24,   2, -16, -20,   6,  22, -22,
    -17, -20, -12, -27, -30, -25, -14, -36,
    -49,  -1, -27, -39, -46, -44, -33, -51,
    -14, -14, -22, -46, -44, -30, -15, -27,
      1,   7,  -8, -64, -43, -16,   9,   8,
    -15,  36,  12, -54,   8, -28,  24,  14,
];
#[rustfmt::skip]
static ENDGAME_KINGTABLE: [i16; 64] = [
    -74, -35, -18, -18, -11,  15,   4, -17,
    -12,  17,  14,  17,  17,  38,  23,  11,
     10,  17,  23,  15,  20,  45,  44,  13,
     -8,  22,  24,  27,  26,  33,  26,   3,
    -18,  -4,  21,  24,  27,  23,   9, -11,
    -19,  -3,  11,  21,  23,  16,   7,  -9,
    -27, -11,   4,  13,  14,   4,  -5, -17,
    -53, -34, -21, -11, -28, -14, -24, -43
];

pub static MG_PSQTS: [[i16; 64]; 6] = [
    MIDDLEGAME_PAWNTABLE,
    MIDDLEGAME_KNIGHTTABLE,
    MIDDLEGAME_BISHOPTABLE,
    MIDDLEGAME_ROOKTABLE,
    MIDDLEGAME_QUEENTABLE,
    MIDDLEGAME_KINGTABLE,
];

pub static EG_PSQTS: [[i16; 64]; 6] = [
    ENDGAME_PAWNTABLE,
    ENDGAME_KNIGHTTABLE,
    ENDGAME_BISHOPTABLE,
    ENDGAME_ROOKTABLE,
    ENDGAME_QUEENTABLE,
    ENDGAME_KINGTABLE,
];

pub static MG_MATERIAL: [i16; 6] = [
    MIDDLEGAME_PAWN,
    MIDDLEGAME_KNIGHT,
    MIDDLEGAME_BISHOP,
    MIDDLEGAME_ROOK,
    MIDDLEGAME_QUEEN,
    0,
];

pub static EG_MATERIAL: [i16; 6] = [
    ENDGAME_PAWN,
    ENDGAME_KNIGHT,
    ENDGAME_BISHOP,
    ENDGAME_ROOK,
    ENDGAME_QUEEN,
    0,
];
// Uses PeSTO tables for evaluation.
// Tapered evaluation forms a gradient between middle game and endgame evaluation parameters.
// e.g. Keep the king safe during middlegame, develop it during endgame.
// Phase is calculated by material

pub static EVAL_PARAMS: Lazy<EvaluationParams> = Lazy::new(generate_eval_params);

pub fn generate_eval_params() -> EvaluationParams {
    let mut eg = [[0; 64]; 6];
    let mut mg = [[0; 64]; 6];
    for ptype in 0..6 {
        let material_mg = MG_MATERIAL[ptype];
        let material_eg = EG_MATERIAL[ptype];
        for square in 0..64 {
            let psqt_mg = MG_PSQTS[ptype][square];
            let psqt_eg = EG_PSQTS[ptype][square];
            mg[ptype][square] = material_mg + psqt_mg;
            eg[ptype][square] = material_eg + psqt_eg;
        }
    }
    EvaluationParams {
        mg_values: mg,
        eg_values: eg,
    }
}

impl Board {
    
    pub fn generate_eval(&self) -> IncrementalEval {
        let mut white_mg_material = 0;
        let mut black_mg_material = 0;
        let mut white_eg_material = 0;
        let mut black_eg_material = 0;

        let mut phase = TOTAL_PHASE;
        for piecetype in 1..=6 {
            let mut white_pieces = self.get_pieces(piecetype, WHITE);
            let mut black_pieces = self.get_pieces(piecetype, BLACK);
            while white_pieces != 0 {
                phase -= PHASES[piecetype as usize - 1];
                let square = white_pieces.pop_lsb().flip();
                white_mg_material += EVAL_PARAMS.get_mg_tables(piecetype)[square as usize];
                white_eg_material += EVAL_PARAMS.get_eg_tables(piecetype)[square as usize];
            }
            while black_pieces != 0 {
                phase -= PHASES[piecetype as usize - 1];
                let square = black_pieces.pop_lsb();
                black_mg_material += EVAL_PARAMS.get_mg_tables(piecetype)[square as usize];
                black_eg_material += EVAL_PARAMS.get_eg_tables(piecetype)[square as usize];
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
    pub phase: u16,
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
        let square = square ^ FLIPS[color as usize];
        let material_val_mg = EVAL_PARAMS.get_mg_tables(piecetype)[square as usize];
        let material_val_eg = EVAL_PARAMS.get_eg_tables(piecetype)[square as usize];
        self.mg_material[color as usize] += material_val_mg;
        self.eg_material[color as usize] += material_val_eg;

        self.phase -= PHASES[piecetype as usize - 1]
    }

    #[inline]
    pub fn remove_piece(&mut self, square: Square, piecetype: Piece, color: Color) {
        let square = square ^ FLIPS[color as usize];
        let material_val_mg = EVAL_PARAMS.get_mg_tables(piecetype)[square as usize];
        let material_val_eg = EVAL_PARAMS.get_eg_tables(piecetype)[square as usize];
        self.mg_material[color as usize] -= material_val_mg;
        self.eg_material[color as usize] -= material_val_eg;
        self.phase += PHASES[piecetype as usize - 1]
    }

    #[inline]
    pub fn move_piece(&mut self, from: Square, to: Square, piecetype: Piece, color: Color) {
        self.remove_piece(from, piecetype, color);
        self.set_piece(to, piecetype, color);
    }

    #[inline]
    pub fn evaluate(&self) -> i16 {
        let mg = self.mg_material[0] - self.mg_material[1];
        let eg = self.eg_material[0] - self.eg_material[1];
        let egphase = self.phase as i16;
        let mgphase = (TOTAL_PHASE - self.phase) as i16;
        (egphase * eg + mgphase * mg) / (TOTAL_PHASE as i16)
    }
}
pub struct EvaluationParams {
    pub mg_values: [[i16; 64]; 6],
    pub eg_values: [[i16; 64]; 6],
}

impl EvaluationParams {
    pub fn get_mg_tables(&self, piecetype: Piece) -> &[i16; 64] {
        &self.mg_values[piecetype as usize - 1]
    }

    pub fn get_eg_tables(&self, piecetype: Piece) -> &[i16; 64] {
        &self.eg_values[piecetype as usize - 1]
    }
}
