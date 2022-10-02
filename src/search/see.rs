use std::cmp;

use crate::{
    board_state::{
        bitboard::{Bitboard, BB},
        board::Board,
        typedefs::{Color, Square, BISHOP, BLACK, KING, KNIGHT, PAWN, QUEEN, ROOK, WHITE},
    },
    move_generation::{
        action::{Action, Move},
        magic::{bishop_attacks, rook_attacks},
        masks::{KING_ATTACKS, KNIGHT_ATTACKS, PAWN_CAPTURES}, makemove::{PASSANT, PROMOTION},
    },
};
pub const SEEVALUES: [i16; 7] = [0, 100, 300, 300, 500, 900, 10_000];

impl Board {
    // SEE for capture pruning and ordering.
    // SEE evaluates the result of many captures on the same square, 
    // by swapping sides and picking the least valuable attacker for each side.
    // At any point if it is advantageous for a side to terminate the exchange it will do so
    pub fn see(&self, action: Move) -> i16 {
        if action.move_type() == PASSANT{
            return 100;
        }
        if action.move_type() == PROMOTION{
            return SEEVALUES[action.promote_to() as usize];
        }
        let from = action.move_from();
        let mut from_bb = Bitboard::new(from);
        let seesquare = action.move_to();
        let attacker = action.piece_moved();
        let target = self.get_at_square(seesquare);

        let could_expose_xray = self[PAWN] | self[BISHOP] | self[ROOK] | self[QUEEN];
        let mut gain = [0; 32];
        let mut depth = 0;
        let mut occupancy = self.get_occupancy();
        let mut attacks_defenders = self.attackers_on_square(seesquare, occupancy);

        let mut color = self.tomove;
        gain[depth] = SEEVALUES[target as usize];
        loop {
            color = !color;
            depth += 1;
            gain[depth] = SEEVALUES[attacker as usize] - gain[depth - 1]; // if not defended we can get the trophy

            if cmp::max(-gain[depth - 1], gain[depth]) < 0 {
                break;
            }
            attacks_defenders ^= from_bb;
            occupancy ^= from_bb;
            if from_bb & could_expose_xray > 0 {
                attacks_defenders |= self.xray_attackers(seesquare, occupancy);
            }
            from_bb = self.get_least_valuable_piece(attacks_defenders, color);
            if from_bb == 0 {
                break;
            }
        }

        while depth > 1 {
            depth -= 1;
            gain[depth as usize - 1] = -cmp::max(-gain[depth - 1], gain[depth]);
        }
        gain[0]
    }

    #[inline]
    fn attackers_on_square(&self, square: Square, occupancy: Bitboard) -> Bitboard {
        self.attackers_for_side(square, WHITE, occupancy)
            | self.attackers_for_side(square, BLACK, occupancy)
    }

    #[inline]
    fn attackers_for_side(
        &self,
        square: Square,
        attacking_color: Color,
        occupancy: Bitboard,
    ) -> Bitboard {
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

        (orthogonals & rookattack)
            | (diagonals & bishopattack)
            | (knights & knightattack)
            | (pawns & pawnattack)
            | (kings & kingattack)
    }

    #[inline]
    fn xray_attackers(&self, square: Square, occupancy: Bitboard) -> Bitboard {
        let diagonal_sliders = self[QUEEN] | self[BISHOP];
        let orthogonal_sliders = self[QUEEN] | self[ROOK];

        let bishop_atks = bishop_attacks(square, occupancy);
        let rook_atks = rook_attacks(square, occupancy);

        ((rook_atks & orthogonal_sliders) | (bishop_atks & diagonal_sliders)) & occupancy
    }

    #[inline]
    fn get_least_valuable_piece(&self, valid_places: Bitboard, color: Color) -> Bitboard {
        for piecetype in 1..=6 {
            let mut subset = self.get_pieces(piecetype, color) & valid_places;
            if subset > 0 {
                return subset.pop_bb();
            }
        }
        0
    }
}
