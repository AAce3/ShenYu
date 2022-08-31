use crate::{
    board_state::{
        board::Board,
        typedefs::{Square, BISHOP, KNIGHT, QUEEN, ROOK},
    },
    move_generation::{
        action::Action,
        makemove::PROMOTION,
    },
};

impl Board {
    pub fn do_input_move(&mut self, movestring: String) -> Result<(), u8> {
        let moves = self.generate_moves::<true, true>();
        let from = &movestring[..2];
        let to = &movestring[2..4];
        let fromsqr = match str_to_sqr(from) {
            Some(sqr) => sqr,
            None => return Err(0),
        };
        let tosqr = match str_to_sqr(to) {
            Some(sqr) => sqr,
            None => return Err(0),
        };

        if movestring.len() == 5 {
            // promotion
            let piece_str = &movestring[4..];
            let pr_piece = match piece_str {
                "q" => QUEEN,
                "n" => KNIGHT,
                "b" => BISHOP,
                "r" => ROOK,
                _ => return Err(0),
            };

            for i in 0..moves.length {
                let action = moves[i as usize];
                if action.move_type() == PROMOTION
                    && action.promote_to() == pr_piece
                    && action.move_from() == fromsqr
                    && action.move_to() == tosqr
                {
                    *self = self.do_move(action);
                    return Ok(());
                }
            }
            Err(0)
        } else {
            for i in 0..moves.length {
                let action = moves[i as usize];
                if action.move_from() == fromsqr && action.move_to() == tosqr {
                    *self = self.do_move(action);
                    return Ok(());
                }
            }
            Err(0)
        }
    }
}

fn str_to_sqr(squarename: &str) -> Option<Square> {
    let squarename = squarename.chars().collect::<Vec<char>>();
    if squarename.len() != 2 {
        None
    } else {
        let file = match squarename[0] {
            'a' => 0,
            'b' => 1,
            'c' => 2,
            'd' => 3,
            'e' => 4,
            'f' => 5,
            'g' => 6,
            'h' => 7,
            _ => return None,
        };

        let rank = match char::to_digit(squarename[1], 10) {
            Some(sqr) => sqr as u8 - 1,
            None => return None,
        };
        Some(rank * 8 + file)
    }
}
