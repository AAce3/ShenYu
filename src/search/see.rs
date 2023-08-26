use std::cmp;

use crate::movegen::{
    action::{Action, MoveType},
    atks,
    bitboard::{self, Bitboard, Direction},
    board::Board,
    types::{Color, Piece, Square},
};

pub const SEEVALUES: [i16; 7] = [100, 300, 300, 500, 900, 10_000, 0];

impl Board {
    pub fn mvv_lva(&self, action: Action) -> i16 {
        let victim = self.get_piece(action.to());
        let attacker = self.get_piece(action.from());
        let attackervalue =  SEEVALUES[attacker as usize] / 8 ;
        SEEVALUES[victim as usize] - attackervalue
    }
}

impl Board {
    // SEE for capture pruning and ordering.
    // SEE evaluates the result of many captures on the same square,
    // by swapping sides and picking the least valuable attacker for each side.
    // At any point if it is advantageous for a side to terminate the exchange it will do so
    pub fn see(&self, action: Action) -> i16 {
        if action.move_type() == MoveType::Passant {
            return 100;
        }

        if action.move_type() == MoveType::Promotion {
            return SEEVALUES[action.pr_piece() as usize];
        }

        let from = action.from();
        let mut from_bb = bitboard::new_bb(from);
        let seesquare = action.to();
        let attacker = self.get_piece(action.from());
        let target = self.get_piece(seesquare);

        let could_expose_xray = self.piecetype(Piece::P)
            | self.piecetype(Piece::B)
            | self.piecetype(Piece::R)
            | self.piecetype(Piece::Q);
        let mut gain = [0; 32];
        let mut depth = 0;
        let mut occupancy = self.occupancy();
        let mut attacks_defenders = self.attackers_on_square(seesquare, occupancy);

        let mut color = self.active_color();
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
            gain[depth - 1] = -cmp::max(-gain[depth - 1], gain[depth]);
        }
        gain[0]
    }

    #[inline]
    fn attackers_on_square(&self, square: Square, occupancy: Bitboard) -> Bitboard {
        self.attackers_for_side(square, Color::W, occupancy)
            | self.attackers_for_side(square, Color::B, occupancy)
    }

    #[inline]
    fn attackers_for_side(&self, square: Square, color: Color, occupancy: Bitboard) -> Bitboard {
        let pawn_forwards = bitboard::forward(bitboard::new_bb(square), !color);

        let pawn_atks = bitboard::shift(pawn_forwards, Direction::E)
            | bitboard::shift(pawn_forwards, Direction::W);
        let king_atks = atks::king_attacks(square);
        let knight_atks = atks::knight_attacks(square);
        let rook_atks = atks::rook_attacks(square, occupancy);
        let bishop_atks = atks::bishop_attacks(square, occupancy);

        let pawns = self.piece_bb(Piece::P, color);
        let kings = self.piece_bb(Piece::K, color);
        let knights = self.piece_bb(Piece::N, color);
        let orthogonals = self.orthogonal_sliders(color);
        let diagonals = self.diagonal_sliders(color);

        (orthogonals & rook_atks)
            | (diagonals & bishop_atks)
            | (knights & knight_atks)
            | (pawns & pawn_atks)
            | (kings & king_atks)
    }

    #[inline]
    fn xray_attackers(&self, square: Square, occupancy: Bitboard) -> Bitboard {
        let diagonal_sliders = self.diagonal_sliders(Color::W) | self.diagonal_sliders(Color::B);
        let orthogonal_sliders =
            self.orthogonal_sliders(Color::W) | self.orthogonal_sliders(Color::B);

        let bishop_atks = atks::bishop_attacks(square, occupancy);
        let rook_atks = atks::rook_attacks(square, occupancy);

        ((rook_atks & orthogonal_sliders) | (bishop_atks & diagonal_sliders)) & occupancy
    }

    #[inline]
    fn get_least_valuable_piece(&self, valid_places: Bitboard, color: Color) -> Bitboard {
        for piecetype in 0..6 {
            let mut subset = self.piece_bb(Piece::from(piecetype), color) & valid_places;
            if subset > 0 {
                return bitboard::pop_bb(&mut subset);
            }
        }
        0
    }
}
