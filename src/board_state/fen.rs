use crate::board_state::{display, typedefs::SQUARE_NAMES};

use super::{
    board::Board,
    typedefs::{Sq, Square, BISHOP, BLACK, KING, KNIGHT, PAWN, QUEEN, ROOK, WHITE},
};
const PIECES: u8 = 0;
const ACTIVE_COLOR: u8 = 1;
const CASTLE_RIGHTS: u8 = 2;
const EP_SQUARE: u8 = 3;
const HALFMOVE_CLOCK: u8 = 4;
const PIECECHARS: [char; 7] = [' ', 'p', 'n', 'b', 'r', 'q', 'k'];
impl Board {
    pub fn generate_fen(&self) -> String {
        let mut base = String::new();
        // parse board
        let mut curr_skip = 0;

        for sqr in 0..64 {
            if sqr % 8 == 0 && sqr != 0 {
                if curr_skip != 0 {
                    let strval = char::from_digit(curr_skip, 10).unwrap();
                    base.push(strval);
                }
                base.push('/');

                curr_skip = 0;
            }
            let actual_sqr = sqr.flip();
            let piece_at = self.get_at_square(actual_sqr);
            if piece_at != 0 {
                if curr_skip != 0 {
                    let strval = char::from_digit(curr_skip, 10).unwrap();
                    base.push(strval);
                }
                curr_skip = 0;
                let mut relevant_char = PIECECHARS[piece_at as usize];
                if self.is_color(actual_sqr, WHITE) {
                    relevant_char = relevant_char.to_ascii_uppercase();
                }
                base.push(relevant_char);
            } else {
                curr_skip += 1;
            }
        }
        const ACTIVE_COLORS: [char; 2] = ['w', 'b'];
        base += " ";
        base.push(ACTIVE_COLORS[self.tomove as usize]);
        base += " ";
        base += display::CASTLE_RIGHTS[self.castling_rights as usize];
        let passantstring = match self.passant_square {
            None => "-",
            Some(val) => SQUARE_NAMES[val as usize],
        };
        base += " ";
        base += passantstring;
        base
    }
    // Parse forsyth-edwards notation, returning a result of whether it was successful or not
    pub fn parse_fen(fen: &str) -> Result<Board, u8> {
        let split_fen = fen.split_whitespace();
        let mut starting_board = Board::new();

        for (id, part) in split_fen.enumerate() {
            match id as u8 {
                PIECES => starting_board.set_pieces(part)?,
                ACTIVE_COLOR => starting_board.set_tomove(part)?,
                CASTLE_RIGHTS => starting_board.set_castling(part)?,
                EP_SQUARE => starting_board.set_ep(part)?,
                HALFMOVE_CLOCK => starting_board.set_halfmove_clock(part)?,
                5 => (),
                _ => return Err(6),
            }
        }
        starting_board.zobrist_key = starting_board.generate_zobrist();
        starting_board.evaluator = starting_board.generate_eval();
        Ok(starting_board)
    }

    fn set_pieces(&mut self, pieces: &str) -> Result<(), u8> {
        let mut file = 0;
        let mut rank = 7;
        for symbol in pieces.chars() {
            if symbol == '/' {
                file = 0;
                rank -= 1;
            } else if symbol.is_numeric() {
                let val = char::to_digit(symbol, 10).unwrap() as u8;
                file += val;
                if file > 8 {
                    return Err(PIECES);
                }
            } else {
                let sqr = (rank * 8) + file;
                let pchar = symbol.to_ascii_lowercase();
                let ptype = match pchar {
                    'k' => KING,
                    'q' => QUEEN,
                    'b' => BISHOP,
                    'n' => KNIGHT,
                    'r' => ROOK,
                    'p' => PAWN,
                    _ => return Err(PIECES),
                };
                let color = if symbol.is_lowercase() { BLACK } else { WHITE };
                self.set_piece(sqr, ptype, color);
                file += 1
            }
        }
        Ok(())
    }

    fn set_tomove(&mut self, part: &str) -> Result<(), u8> {
        if part == "w" {
            self.tomove = WHITE;
            Ok(())
        } else if part == "b" {
            self.tomove = BLACK;
            Ok(())
        } else {
            Err(CASTLE_RIGHTS)
        }
    }

    fn set_castling(&mut self, part: &str) -> Result<(), u8> {
        let wk = part.contains('K') as u8;
        let wq = part.contains('Q') as u8;
        let bk = part.contains('k') as u8;
        let bq = part.contains('q') as u8;

        self.castling_rights = (wk << 3) | (wq << 2) | (bk << 1) | bq;
        if self.castling_rights != 0 || part.contains('-') {
            Ok(())
        } else {
            Err(CASTLE_RIGHTS)
        }
    }

    fn set_ep(&mut self, part: &str) -> Result<(), u8> {
        if part == "-" {
            self.passant_square = None;
            Ok(())
        } else {
            let sqr = Square::algebraic_to_sqr(part);
            match sqr {
                None => Err(EP_SQUARE),
                Some(val) => {
                    self.passant_square = Some(val);
                    Ok(())
                }
            }
        }
    }

    fn set_halfmove_clock(&mut self, part: &str) -> Result<(), u8> {
        let val = part.parse::<u8>();
        match val {
            Ok(num) => {
                self.halfmove_clock = num;
                Ok(())
            }
            Err(_) => Err(0),
        }
    }
}
