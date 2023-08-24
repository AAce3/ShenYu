use std::fmt::{self, Write};

use super::{
    board::{Board, Castling},
    types::{square, Color, Piece},
};

impl fmt::Display for Board {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        for rank in (0..8).rev() {
            let displayed_rank = rank + 1;
            write!(f, "{displayed_rank} ")?;
            for file in 0..8 {
                let square = square::new_sq(rank, file);
                let piece_at = self.get_piece(square);
                let mut name = piece_at.name();
                if self.is_color(square, Color::W) {
                    name = name.to_ascii_uppercase();
                }
                if piece_at == Piece::None {
                    assert!(!self.is_color(square, Color::W) && !self.is_color(square, Color::B), "{square}")
                } else {
                    assert!(self.is_color(square, Color::W) || self.is_color(square, Color::B), "{square}")
                }
                write!(f, "{name} ")?;
            }
            writeln!(f)?;
        }
        write!(f, "  ")?;
        for file in b'a'..=b'h' {
            let file = file as char;
            write!(f, "{file} ")?;
        }
        writeln!(f)?;
        writeln!(f, "\n|[====---------------------====]|")?;
        writeln!(f, "|[    Active Color: {}", self.active_color())?;
        write!(f, "|[    Castle Rights: ")?;

        if *self.castling(Castling::WK) {
            f.write_char('K')?;
        }

        if *self.castling(Castling::WQ) {
            f.write_char('Q')?;
        }

        if *self.castling(Castling::BK) {
            f.write_char('k')?;
        }

        if *self.castling(Castling::BQ) {
            f.write_char('q')?;
        }

        if !(*self.castling(Castling::WK)
            | *self.castling(Castling::WQ)
            | *self.castling(Castling::BK)
            | *self.castling(Castling::BQ))
        {
            f.write_char('-')?;
        }

        writeln!(f)?;

        let ep_str = match self.passant_square() {
            Some(square) => square::name(square),
            None => "-",
        };

        writeln!(f, "|[    En Passant Square: {ep_str}")?;
        writeln!(f, "|[    Halfmove Clock: {}", self.halfmove_clock())?;
        f.write_str("|[====---------------------====]|")?;
        Ok(())
    }
}
