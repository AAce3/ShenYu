use std::{cmp::Ordering, intrinsics::unlikely};

use once_cell::sync::Lazy;

use crate::{
    board_state::{
        bitboard::{
            Bitboard,
            Direction::{E, W},
            BB,
        },
        board::Board,
        typedefs::{Color, Piece, Square, BISHOP, KING, KNIGHT, PAWN, QUEEN, ROOK},
    },
    move_generation::makemove::{PASSANT, PROMOTION},
};

use super::{
    action::{Action, Move},
    list::List,
    magic::{bishop_attacks, rook_attacks},
    makemove::NORMAL,
    masks::{KING_ATTACKS, KNIGHT_ATTACKS, PAWN_CAPTURES},
};
pub const BLOCK_OCCUPIED_KINGSIDE: [Bitboard; 2] = [0x60, 0x6000000000000000];
pub const BLOCK_OCCUPIED_QUEENSIDE: [Bitboard; 2] = [0xe, 0xe00000000000000];
pub const BLOCK_CHECKED_KINGSIDE: [Bitboard; 2] = [0x70, 0x7000000000000000];
pub const BLOCK_CHECKED_QUEENSIDE: [Bitboard; 2] = [0x1c, 0x1c00000000000000];

pub static INBETWEENS: Lazy<[[Bitboard; 64]; 64]> = Lazy::new(generate_inbetweens);
impl Board {
    pub fn generate_moves<const QUIETS: bool, const CAPTURES: bool>(&mut self) -> List<Move> {
        // Move generation is separated.
        // First, pins and legal squares are determined.
        // For a king, legal squares are where the enemy does not attack.
        // Sliding pieces have to "see through" the king, otherwise you could move backwards from a rook attack for example.
        // If we are in check, the only squares for other pieces are to block (if the checker is a slider)
        // or to capture the attacking piece, unless we are in double check.
        // Pinned pieces are calculated using X-ray attacks, and their attacks are generated separately.
        // In addition, captures are generated separately from quiets.

        let mut list = List::new();
        let occupancy = self.get_occupancy();
        let quiet_sqrs = if QUIETS { !occupancy } else { 0 };
        let capture_squares = if CAPTURES { self[!self.tomove] } else { 0 };

        let kingbb = self.get_pieces(KING, self.tomove);
        let blind_board = occupancy ^ kingbb; // ray-attackers "see through" the king
        let atk_mask = self.generate_atk_mask(!self.tomove, blind_board);

        let legal_squares = if atk_mask & kingbb == 0 {
            u64::MAX
        } else {
            self.get_movemask()
        };

        {
            let kingsqr = kingbb.lsb();
            let moves = KING_ATTACKS[kingsqr as usize] & !atk_mask;
            let mut capture_moves = moves & capture_squares;
            let mut quiet_moves = moves & quiet_sqrs;
            while capture_moves != 0 {
                let to = capture_moves.pop_lsb();
                let mut action = Move::new_move(kingsqr, to, NORMAL, KING);
                action.set_capture();
                list.push(action);
            }

            while quiet_moves != 0 {
                let to = quiet_moves.pop_lsb();
                let action = Move::new_move(kingsqr, to, NORMAL, KING);
                list.push(action);
            }

            // generate castling moves
            if QUIETS {
                let rights = self.get_castlerights(self.tomove);
                let is_kingside_legal = rights & 0b10 != 0;
                let is_queenside_legal = rights & 1 != 0;
                let blocking_castle = occupancy;
                let checking_castle = atk_mask;

                let is_kingside_blocked =
                    blocking_castle & BLOCK_OCCUPIED_KINGSIDE[self.tomove as usize] != 0
                        || checking_castle & BLOCK_CHECKED_KINGSIDE[self.tomove as usize] != 0;
                let is_queenside_blocked =
                    blocking_castle & BLOCK_OCCUPIED_QUEENSIDE[self.tomove as usize] != 0
                        || checking_castle & BLOCK_CHECKED_QUEENSIDE[self.tomove as usize] != 0;
                const KINGSIDE_CASTLES: [Move; 2] = [397700, 401340];
                const QUEENSIDE_CASTLES: [Move; 2] = [397444, 401084];
                if !is_kingside_blocked && is_kingside_legal {
                    list.push(KINGSIDE_CASTLES[self.tomove as usize]);
                }

                if !is_queenside_blocked && is_queenside_legal {
                    list.push(QUEENSIDE_CASTLES[self.tomove as usize]);
                }
            }
        }
        let rook_pinmask = self.get_rpinmask();
        let bishop_pinmask = self.get_bpinmask();
        {
            // Knights cannot move along rays, so we don't need to ever consider pinned knights
            let mut knights =
                self.get_pieces(KNIGHT, self.tomove) & !bishop_pinmask & !rook_pinmask;
            while knights != 0 {
                let square = knights.pop_lsb();
                let attacks = KNIGHT_ATTACKS[square as usize] & legal_squares;
                let mut capture_moves = attacks & capture_squares;
                let mut quiet_moves = attacks & quiet_sqrs;

                while capture_moves != 0 {
                    let to_square = capture_moves.pop_lsb();
                    let mut action = Move::new_move(square, to_square, NORMAL, KNIGHT);
                    action.set_capture();
                    list.push(action);
                }

                while quiet_moves != 0 {
                    let to_square = quiet_moves.pop_lsb();
                    let action = Move::new_move(square, to_square, NORMAL, KNIGHT);
                    list.push(action);
                }
            }
        }

        {
            // Bishops can move along diagonal rays.
            // We get diagonal sliders so we don't have to generate queen moves separately.
            let mut bishops = self.get_diagonal_sliders(self.tomove) & !rook_pinmask;
            let mut pinned_bishops = bishops & bishop_pinmask;
            bishops ^= pinned_bishops;
            while bishops != 0 {
                let square = bishops.pop_lsb();
                let attacks = bishop_attacks(square, occupancy) & legal_squares;
                let mut capture_moves = attacks & capture_squares;
                let mut quiet_moves = attacks & quiet_sqrs;
                let is_queen = ((Bitboard::new(square) & self[QUEEN]) >> square) as u8;
                let piece_moved = BISHOP + (is_queen * 2);
                while capture_moves != 0 {
                    let to_square = capture_moves.pop_lsb();
                    let mut action = Move::new_move(square, to_square, NORMAL, piece_moved);
                    action.set_capture();
                    list.push(action);
                }

                while quiet_moves != 0 {
                    let to_square = quiet_moves.pop_lsb();
                    let action = Move::new_move(square, to_square, NORMAL, piece_moved);
                    list.push(action);
                }
            }

            while pinned_bishops != 0 {
                let square = pinned_bishops.pop_lsb();
                let attacks = bishop_attacks(square, occupancy) & legal_squares & bishop_pinmask;
                let mut capture_moves = attacks & capture_squares;
                let mut quiet_moves = attacks & quiet_sqrs;
                let is_queen = ((Bitboard::new(square) & self[QUEEN]) >> square) as u8;
                let piece_moved = BISHOP + (is_queen * 2);
                while capture_moves != 0 {
                    let to_square = capture_moves.pop_lsb();
                    let mut action = Move::new_move(square, to_square, NORMAL, piece_moved);
                    
                    action.set_capture();
                    list.push(action);
                }

                while quiet_moves != 0 {
                    let to_square = quiet_moves.pop_lsb();
                    let action = Move::new_move(square, to_square, NORMAL, piece_moved);

                    list.push(action);
                }
            }
        }

        {
            // Like bishops, we get all orthogonal sliders so we don't have to separately generate queen moves.
            // Rooks can move along orthogonal rays.
            let mut rooks = self.get_orthogonal_sliders(self.tomove) & !bishop_pinmask;
            let mut pinned_rooks = rooks & rook_pinmask;
            rooks ^= pinned_rooks;
            while rooks != 0 {
                let square = rooks.pop_lsb();
                let attacks = rook_attacks(square, occupancy) & legal_squares;
                let mut capture_moves = attacks & capture_squares;
                let mut quiet_moves = attacks & quiet_sqrs;
                let is_queen = ((Bitboard::new(square) & self[QUEEN]) >> square) as u8;
                let piece_moved = ROOK + is_queen;
                while capture_moves != 0 {
                    let to_square = capture_moves.pop_lsb();
                    let mut action = Move::new_move(square, to_square, NORMAL, piece_moved);

                    action.set_capture();
                    list.push(action);
                }

                while quiet_moves != 0 {
                    let to_square = quiet_moves.pop_lsb();
                    let action = Move::new_move(square, to_square, NORMAL, piece_moved);
                    list.push(action);
                }
            }

            while pinned_rooks != 0 {
                let square = pinned_rooks.pop_lsb();
                let attacks = rook_attacks(square, occupancy) & legal_squares & rook_pinmask;
                let mut capture_moves = attacks & capture_squares;
                let mut quiet_moves = attacks & quiet_sqrs;
                let is_queen = ((Bitboard::new(square) & self[QUEEN]) >> square) as u8;
                let piece_moved = ROOK + is_queen;

                while capture_moves != 0 {
                    let to_square = capture_moves.pop_lsb();
                    let mut action = Move::new_move(square, to_square, NORMAL, piece_moved);
                    action.set_capture();
                    list.push(action);
                }

                while quiet_moves != 0 {
                    let to_square = quiet_moves.pop_lsb();
                    let action = Move::new_move(square, to_square, NORMAL, piece_moved);
                    list.push(action);
                }
            }
        }

        {
            const EIGTH_RANK: [Bitboard; 2] = [0xff00000000000000, 0xff];
            const FOURTH_RANK: [Bitboard; 2] = [0xff000000, 0xff00000000];
            const PIECES: [Piece; 4] = [QUEEN, ROOK, KNIGHT, BISHOP];
            // generate pawn pushes
            let mut pawns = self.get_pieces(PAWN, self.tomove);
            let mut rook_pinned = pawns & rook_pinmask;
            let mut bishop_pinned = pawns & bishop_pinmask;
            pawns ^= rook_pinned;
            pawns ^= bishop_pinned;
            while pawns != 0 {
                let bb = pawns.pop_bb();
                let from = bb.lsb();
                //generate forward push
                let forward =
                    bb.forward(self.tomove) & quiet_sqrs & !EIGTH_RANK[self.tomove as usize];

                if forward != 0 {
                    if forward & legal_squares != 0 {
                        let to = forward.lsb();
                        let onepush = Move::new_move(from, to, NORMAL, PAWN);
                        list.push(onepush);
                    }
                    let doublepush = forward.forward(self.tomove)
                        & quiet_sqrs
                        & FOURTH_RANK[self.tomove as usize]
                        & legal_squares;

                    if doublepush != 0 {
                        let double_to = doublepush.lsb();
                        let mut doublepush = Move::new_move(from, double_to, NORMAL, PAWN);
                        doublepush.set_doublemove();
                        list.push(doublepush);
                    }
                }

                let mut pr_push =
                    bb.forward(self.tomove) & EIGTH_RANK[self.tomove as usize] & !occupancy;
                pr_push &= legal_squares;

                if unlikely(pr_push != 0) {
                    let to = pr_push.lsb();
                    let action = Move::new_move(from, to, PROMOTION, PAWN);
                    if CAPTURES {
                        let mut pr_move = action;
                        pr_move.set_pr_piece(QUEEN);
                        list.push(pr_move);
                    }
                    if QUIETS {
                        for piecetype in PIECES.iter().skip(1) {
                            let mut pr_move = action;
                            pr_move.set_pr_piece(*piecetype);
                            list.push(pr_move);
                        }
                    }
                }

                let mut capture_moves = PAWN_CAPTURES[self.tomove as usize][from as usize]
                    & legal_squares
                    & capture_squares
                    & !EIGTH_RANK[self.tomove as usize];

                while capture_moves != 0 {
                    let to = capture_moves.pop_lsb();
                    let mut action = Move::new_move(from, to, NORMAL, PAWN);
                    
                    action.set_capture();
                    list.push(action);
                }

                let mut pr_captures = PAWN_CAPTURES[self.tomove as usize][from as usize]
                    & EIGTH_RANK[self.tomove as usize]
                    & self[!self.tomove]
                    & legal_squares;

                while pr_captures != 0 {
                    let to = pr_captures.pop_lsb();
                    let mut action = Move::new_move(from, to, PROMOTION, PAWN);
                    action.set_capture();
                    if CAPTURES {
                        let mut pr_move = action;
                        pr_move.set_pr_piece(QUEEN);
                        list.push(pr_move);
                    }
                    if QUIETS {
                        for piecetype in PIECES.iter().skip(1) {
                            let mut pr_move = action;
                            pr_move.set_pr_piece(*piecetype);
                            list.push(pr_move);
                        }
                    }
                }
            }

            while rook_pinned != 0 {
                let bb = rook_pinned.pop_bb();
                let from = bb.lsb();
                //generate forward push
                let forward = bb.forward(self.tomove)
                    & quiet_sqrs
                    & rook_pinmask
                    & legal_squares
                    & !EIGTH_RANK[self.tomove as usize];

                if forward != 0 {
                    if forward & legal_squares != 0 {
                        let to = forward.lsb();
                        let onepush = Move::new_move(from, to, NORMAL, PAWN);
                        list.push(onepush);
                    }
                    let doublepush = forward.forward(self.tomove)
                        & quiet_sqrs
                        & FOURTH_RANK[self.tomove as usize]
                        & legal_squares;

                    if doublepush != 0 {
                        let double_to = doublepush.lsb();
                        let mut doublepush = Move::new_move(from, double_to, NORMAL, PAWN);
                        doublepush.set_doublemove();
 
                        list.push(doublepush);
                    }
                }
            }
            while bishop_pinned != 0 {
                let from = bishop_pinned.pop_lsb();
                let mut capture_moves = PAWN_CAPTURES[self.tomove as usize][from as usize]
                    & capture_squares
                    & legal_squares
                    & bishop_pinmask
                    & !EIGTH_RANK[self.tomove as usize];

                while capture_moves != 0 {
                    let to = capture_moves.pop_lsb();
                    let mut action = Move::new_move(from, to, NORMAL, PAWN);
                    action.set_capture();
                    list.push(action);
                }

                let mut pr_captures = PAWN_CAPTURES[self.tomove as usize][from as usize]
                    & EIGTH_RANK[self.tomove as usize]
                    & self[!self.tomove]
                    & legal_squares
                    & bishop_pinmask;

                while pr_captures != 0 {
                    let to = pr_captures.pop_lsb();
                    let mut action = Move::new_move(from, to, PROMOTION, PAWN);
                    action.set_capture();
                    if CAPTURES {
                        let mut pr_move = action;
                        pr_move.set_pr_piece(QUEEN);
                        list.push(pr_move);
                    }
                    if QUIETS {
                        for piecetype in PIECES.iter().skip(1) {
                            let mut pr_move = action;
                            pr_move.set_pr_piece(*piecetype);
                            list.push(pr_move);
                        }
                    }
                }
            }
            {
                if CAPTURES {
                    let passant_bb: Bitboard;
                    let square: Square;
                    match self.passant_square {
                        None => {
                            passant_bb = 0;
                            square = 64;
                        }
                        Some(sqr) => {
                            passant_bb = PAWN_CAPTURES[!self.tomove as usize][sqr as usize];
                            square = sqr;
                        } // generate passant attacks by "reverse"
                    };

                    let mut possible_pawns = passant_bb & self.get_pieces(PAWN, self.tomove);
                    while possible_pawns != 0 {
                        let from = possible_pawns.pop_lsb();
                        let to = square;
                        let newpassant = Move::new_move(from, to, PASSANT, PAWN);
                        let newb = self.do_move(newpassant);
                        if !newb.incheck(self.tomove) {
                            list.push(newpassant);
                        }
                    }
                }
            }
        }

        list
    }

    #[inline]
    pub fn generate_atk_mask(&self, color: Color, occupancy_mask: Bitboard) -> Bitboard {
        let mut base = 0;
        let pawnbitboards = self.get_pieces(PAWN, color);
        let shifted_forward = pawnbitboards.forward(color);
        base |= shifted_forward.shift::<{ E }>() | shifted_forward.shift::<{ W }>();

        let mut knightbitboards = self.get_pieces(KNIGHT, color);
        while knightbitboards != 0 {
            let sqr = knightbitboards.pop_lsb();
            base |= KNIGHT_ATTACKS[sqr as usize];
        }

        let mut bishop_bb = self.get_diagonal_sliders(color);
        while bishop_bb != 0 {
            let sqr = bishop_bb.pop_lsb();
            base |= bishop_attacks(sqr, occupancy_mask);
        }

        let mut rook_bb = self.get_orthogonal_sliders(color);
        while rook_bb != 0 {
            let sqr = rook_bb.pop_lsb();
            base |= rook_attacks(sqr, occupancy_mask);
        }

        let kingmask = self.get_pieces(KING, color);
        let sqr = kingmask.lsb();
        base |= KING_ATTACKS[sqr as usize];

        base
    }
    // if we are in check, we generate a mask that gives all the squares that we can get out of check with
    #[inline]
    pub fn check_for_legalmoves(&self) -> Bitboard {
        let occupancy = self.get_occupancy();
        let myking = self.get_pieces(KING, self.tomove);

        let curr_king_sqr = myking.lsb();
        let enemy = !self.tomove;
        let knight_attackers =
            KNIGHT_ATTACKS[curr_king_sqr as usize] & self.get_pieces(KNIGHT, enemy);
        let pawn_attackers = PAWN_CAPTURES[self.tomove as usize][curr_king_sqr as usize]
            & self.get_pieces(PAWN, enemy);

        let jumpers = knight_attackers | pawn_attackers;

        let bishop_attackers =
            bishop_attacks(curr_king_sqr, occupancy) & self.get_diagonal_sliders(enemy);

        let rook_attackers =
            rook_attacks(curr_king_sqr, occupancy) & self.get_orthogonal_sliders(enemy);

        let sliders = bishop_attackers | rook_attackers;

        let num_checkers = (jumpers | sliders).count_ones();
        match num_checkers.cmp(&1) {
            Ordering::Greater => 0,
            Ordering::Equal => {
                if sliders != 0 {
                    let btwn = INBETWEENS[sliders.lsb() as usize][curr_king_sqr as usize];
                    btwn | sliders
                } else {
                    jumpers
                }
            }
            Ordering::Less => u64::MAX,
        }
    }

    // Pin squares are generated using X-ray attacks, by "seeing through" our pieces
    #[inline]
    pub fn generate_bishop_pins(&self) -> Bitboard {
        let kingsquare = self.get_pieces(KING, self.tomove).lsb();

        let mut mypieces = self[self.tomove];
        let enemypieces = self[!self.tomove];
        let occ = self.get_occupancy();

        let xray_bishopattacks = bishop_attacks(kingsquare, occ);
        mypieces &= !xray_bishopattacks;

        let mut new_bishopattackers = bishop_attacks(kingsquare, mypieces | enemypieces)
            & self.get_diagonal_sliders(!self.tomove);
        let mut pinmask = new_bishopattackers;
        while new_bishopattackers != 0 {
            let sqr2 = new_bishopattackers.pop_lsb();
            pinmask |= INBETWEENS[kingsquare as usize][sqr2 as usize];
        }

        pinmask
    }

    #[inline]
    pub fn generate_rook_pins(&self) -> Bitboard {
        let kingsquare = self.get_pieces(KING, self.tomove).lsb();

        let mut mypieces = self[self.tomove];
        let enemypieces = self[!self.tomove];
        let occ = self.get_occupancy();

        let xray_rookattacks = rook_attacks(kingsquare, occ);
        mypieces &= !xray_rookattacks;

        let mut new_rookattackers = rook_attacks(kingsquare, mypieces | enemypieces)
            & self.get_orthogonal_sliders(!self.tomove);
        let mut pinmask = new_rookattackers;
        while new_rookattackers != 0 {
            let sqr2 = new_rookattackers.pop_lsb();
            pinmask |= INBETWEENS[kingsquare as usize][sqr2 as usize];
        }
        pinmask
    }

    #[inline]
    pub fn incheck(&self, color: Color) -> bool {
        let relevant_king_square = self.get_pieces(KING, color).lsb();
        self.is_attacked(relevant_king_square, !color)
    }
    #[inline]
    pub fn is_attacked(&self, square: Square, attacking_color: Color) -> bool {
        let occupancy = self.get_occupancy();

        let kingattack = KING_ATTACKS[square as usize];
        let pawnattack = PAWN_CAPTURES[!attacking_color as usize][square as usize];
        let knightattack = KNIGHT_ATTACKS[square as usize];
        let bishopattack = bishop_attacks(square, occupancy);
        let rookattack = rook_attacks(square, occupancy);

        let kings = self.get_pieces(KING, attacking_color);
        let pawns = self.get_pieces(PAWN, attacking_color);
        let knights = self.get_pieces(KNIGHT, attacking_color);
        let orthogonals = self.get_orthogonal_sliders(attacking_color);
        let diagonals = self.get_diagonal_sliders(attacking_color);

        (kings & kingattack)
            | (pawns & pawnattack)
            | (knights & knightattack)
            | (orthogonals & rookattack)
            | (diagonals & bishopattack)
            != 0
    }
}

#[allow(clippy::needless_range_loop)]
fn generate_inbetweens() -> [[Bitboard; 64]; 64] {
    let mut inbetweens = [[0; 64]; 64];
    for sqr1 in 0..64 {
        for sqr2 in 0..64 {
            inbetweens[sqr1][sqr2] = in_between_rays(sqr1 as u8, sqr2 as u8);
        }
    }
    inbetweens
}

// complicated method from CPW to determine the mask in between two squares
fn in_between_rays(square1: Square, square2: Square) -> Bitboard {
    const M1: u64 = u64::MAX;
    const A2A7: u64 = 0x0001010101010100;
    const B2G7: u64 = 0x0040201008040200;
    const H1B7: u64 = 0x0002040810204080;
    let (mut line, btwn, rank, file): (u64, u64, u64, u64);
    btwn = (M1 << square1) ^ (M1 << square2);
    file = ((square2 & 7) - (square1 & 7)) as u64;
    rank = (((square2 | 7) - square1) >> 3) as u64;
    line = ((file & 7) - 1) & A2A7;
    line += 2 * (((rank & 7) - 1) >> 58);
    line += (((rank - file) & 15) - 1) & B2G7;
    line += (((rank + file) & 15) - 1) & H1B7;
    line *= btwn & btwn.wrapping_neg();
    line & btwn
}
