use super::{
    board::Board,
    typedefs::{Sq, BLACK, WHITE, SQUARE_NAMES},
};
use std::fmt;
use std::fmt::Write;

const SYMBOLS: [[char; 2]; 7] = [
    [' ', ' '],
    ['♟', '♙'],
    ['♞', '♘'],
    ['♝', '♗'],
    ['♜', '♖'],
    ['♛', '♕'],
    ['♚', '♔'],
];
pub const CASTLE_RIGHTS: [&str; 16] = [
    "-", "q", "k", "kq", "Q", "Qq", "Qk", "Qkq", "K", "Kq", "Kk", "Kkq", "KQ", "KQq", "KQk", "KQkq",
];
// prints a board to stdout for debugging
impl fmt::Display for Board {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut base_string = String::new();
        let mut char_indices = [' '; 64]; // this is in big-endian, so to get indeces we have to use XOR with 56
        for square in 0..64 {
            let idx = square.flip();
            let piece_at_square = self.get_at_square(square);
            let symbols = SYMBOLS[piece_at_square as usize];

            if piece_at_square != 0 {
                if self.is_color(square, WHITE) {
                    let valid_character = symbols[0];
                    char_indices[idx as usize] = valid_character;
                } else if self.is_color(square, BLACK) {
                    let valid_character = symbols[1];
                    char_indices[idx as usize] = valid_character;
                } else {
                    panic!("Something went wrong when displaying board")
                }
            }
        }
        for rank in 0..8 {
            base_string += "\n---+----+----+----+----+----+----+----+----+\n";
            let num = 8 - rank;
            write!(&mut base_string, " {}", num).unwrap();
            base_string += " | ";

            for file in 0..8 {
                let index = rank * 8 + file;
                let char_at_idx = char_indices[index];
                base_string.push(char_at_idx);
                base_string += "  | ";
            }
            //base_string += "|"
        }
        base_string += "\n---+----+----+----+----+----+----+----+----+";
        base_string += "\n   | a  | b  | c  | d  | e  | f  | g  | h  |";
        base_string += "\n\n+------------------------------------------+";
        write!(
            &mut base_string,
            "\n|   Active Color: {}",
            if self.tomove == WHITE {
                "White"
            } else {
                "Black"
            }
        )
        .unwrap();

        let passantstring = match self.passant_square{
            None => "None",
            Some(val) => SQUARE_NAMES[val as usize]
        };

        write!(&mut base_string, "\n|   Valid En Passant Square: {}", passantstring).unwrap();
        write!(&mut base_string, "\n|   Castling Rights: {}", CASTLE_RIGHTS[self.castling_rights as usize]).unwrap();
        write!(&mut base_string, "\n|   Halfmove Clock: {}", self.halfmove_clock).unwrap();
        write!(&mut base_string, "\n|   Zobrist Key: {}", self.zobrist_key).unwrap();
        base_string += "\n+------------------------------------------+";
        write!(f, "{}", base_string)
    }
}
