use crate::movegen::types::{square, Color, Piece};

use super::board::{Board, Castling};
use anyhow::{bail, Context, Result};

impl Board {
    pub fn parse_fen(&mut self, fen: &str) -> Result<()> {
        *self = Self::default();
        let mut split_fen = fen.split_whitespace();
        let mut square = 56;
        let piece_placement_str = split_fen.next().context("Invalid fen")?;
        // parse piece placement part of fen
        for character in piece_placement_str.chars() {
            if character.is_ascii_digit() {
                // if it is a number, advance by that amount
                square += character as u8 - b'0'
            } else if character == '/' {
                // go to next row
                square -= 16;
                if square >= 64 || square % 8 != 0 {
                    bail!("Invalid fen")
                }
            } else {
                // determine piece to be placed
                const PIECE_CHARS: [Piece; 6] =
                    [Piece::P, Piece::N, Piece::B, Piece::R, Piece::Q, Piece::K];
                let piece = PIECE_CHARS
                    .iter()
                    .find(|a| a.name() == character.to_ascii_lowercase())
                    .context("Invalid fen")?;
                let piece_color = if character.is_uppercase() {
                    Color::W
                } else {
                    Color::B
                };

                self.add_piece::<true>(square, *piece, piece_color);
                square += 1
            }
        }
        let to_move_str = split_fen.next().context("Invalid fen")?;

        if to_move_str == "b" {
            self.swap_sides()
        } else if to_move_str != "w" {
            bail!("Invalid fen")
        } // if it is black, swap sides. otherwise, it had better be white

        let castling_str = split_fen.next().context("Invalid fen")?;
         
        for character in castling_str.chars() {
            match character {
                'K' => self.set_castling(Castling::WK, true),
                'Q' => self.set_castling(Castling::WQ, true),
                'k' => self.set_castling(Castling::BK, true),
                'q' => self.set_castling(Castling::BQ, true),
                '-' => break,
                _ => bail!("Invalid fen"),
            }
        }

        let passant_str = split_fen.next().context("Invalid fen")?;
        if passant_str != "-" {
            let ep_square = square::from_algebraic(passant_str).context("Invalid fen")?;
            self.set_ep(ep_square);
        }

        let fifty_str = split_fen.next().context("Invalid fen")?;
        self.set_fifty(fifty_str.parse::<u8>()?);
        self.set_evalinfo();
        Ok(())
    }
}
