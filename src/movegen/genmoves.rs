use super::{
    action::{Action, MoveType},
    atks,
    bitboard::{self, Bitboard, Direction},
    board::{Board, Castling},
    movelist::MoveList,
    types::{square, Color, Piece, Square},
};

pub struct GenType;
impl GenType {
    pub const QUIETS: u8 = 0b01;
    pub const CAPTURES: u8 = 0b10;
    pub const ALL: u8 = 0b11;

    fn is_quiet(flags: u8) -> bool {
        flags & Self::QUIETS != 0
    }

    fn is_captures(flags: u8) -> bool {
        flags & Self::CAPTURES != 0
    }
}

macro_rules! select {
    ($condition:expr, $false_option:expr, $true_option:expr) => {
        [$false_option, $true_option][($condition as usize)]
    };
}

macro_rules! new_bbs {
    ( $($square:expr),* ) => {
        $(bitboard::new_bb($square)) | *
    };
}

impl Board {
    // used to check if TT move or killers is pseudolegal. Legality check will be done later
    pub fn is_pseudolegal(&mut self, action: Action) -> bool {
        let us = self.active_color();
        let them = !us;
        match action.move_type() {
            MoveType::Normal => {
                if action.pr_piece() != Piece::N {
                    return false;
                }
                // do a little more work for normal moves.
                if self.is_color(action.to(), us) || !self.is_color(action.from(), us) {
                    return false;
                }

                if self.get_piece(action.to()) == Piece::K {
                    return false;
                }

                let moving_piece = self.get_piece(action.from());

                match moving_piece {
                    Piece::P => {
                        if square::rank_of(action.to()) == 0 || square::rank_of(action.to()) == 7 {
                            return false;
                        }

                        let forward_bb = bitboard::forward(bitboard::new_bb(action.from()), us);

                        if self.is_color(action.to(), them) {
                            let capture_squares = bitboard::shift(forward_bb, Direction::E)
                                | bitboard::shift(forward_bb, Direction::W);
                            return bitboard::is_set(capture_squares, action.to());
                        } else {
                            // we know that action.to() is empty now. So, we can skip checking that

                            // if normal push
                            if bitboard::is_set(forward_bb, action.to()) {
                                return true;
                            }

                            // otherwise doublepush
                            let double_forward = bitboard::forward(forward_bb, us);
                            let fourth_rank = select!(us, 3, 4);

                            if square::rank_of(action.to()) == fourth_rank
                                && bitboard::is_set(double_forward, action.to())
                                && forward_bb & self.occupancy() == 0
                            {
                                return true;
                            }
                        }
                    }
                    Piece::N => {
                        return bitboard::is_set(atks::knight_attacks(action.from()), action.to())
                    }
                    Piece::B => {
                        return bitboard::is_set(
                            atks::bishop_attacks(action.from(), self.occupancy()),
                            action.to(),
                        )
                    }
                    Piece::R => {
                        return bitboard::is_set(
                            atks::rook_attacks(action.from(), self.occupancy()),
                            action.to(),
                        )
                    }
                    Piece::Q => {
                        return bitboard::is_set(
                            atks::bishop_attacks(action.from(), self.occupancy()),
                            action.to(),
                        ) || bitboard::is_set(
                            atks::rook_attacks(action.from(), self.occupancy()),
                            action.to(),
                        )
                    }
                    Piece::K => {
                        return bitboard::is_set(atks::king_attacks(action.from()), action.to())
                    }
                    Piece::None => return false,
                }
            }
            // castles, promotions and passants are all super rare. So, to test for legality, just generate the moves
            // as a cheap and dirty alternative to doing annoying piece checking
            MoveType::Castle => {
                let mut movelist = MoveList::new();

                if action.pr_piece() != Piece::N {
                    return false;
                }

                if self.get_piece(action.from()) == Piece::K {
                    let king_bb = self.piece_bb(Piece::K, us);
                    let atk_mask = self.generate_atk_mask(them, self.occupancy() ^ king_bb);

                    if atk_mask & king_bb != 0 {
                        return false;
                    }
                    self.generate_castles(atk_mask, &mut movelist);
                    return movelist.iter().any(|&a| *a == action);
                }
            }

            MoveType::Promotion => {
                let mut movelist = MoveList::new();

                let eighth_rank = select!(us, 7, 0);
                if self.get_piece(action.from()) == Piece::P
                    && square::rank_of(action.to()) == eighth_rank
                {
                    if self.is_color(action.to(), them) {
                        self.pawn_captures::<true>(
                            self.piece_bb(Piece::P, us),
                            self.color_bb(them),
                            &mut movelist,
                        )
                    } else {
                        self.pawn_forwards::<true>(
                            self.piece_bb(Piece::P, us),
                            self.occupancy(),
                            bitboard::FULL,
                            &mut movelist,
                        );
                    }
                    return movelist.iter().any(|&a| *a == action);
                }
            }
            MoveType::Passant => {
                let mut movelist = MoveList::new();

                if action.pr_piece() != Piece::N {
                    return false;
                }
                if self.get_piece(action.from()) == Piece::P {
                    self.generate_passant(&mut movelist);
                    return movelist.iter().any(|&a| *a == action);
                }
            }
        }

        false
    }

    pub fn genmoves<const STAGE: u8>(&mut self, movelist: &mut MoveList) {
        assert!(STAGE != 0);
        let us = self.active_color();
        let them = !us;
        let our_pieces = self.color_bb(us);
        let their_pieces = self.color_bb(them);
        let occ = our_pieces | their_pieces;

        let mut valid_squares = !our_pieces; // initially set to prevent self captures

        if !GenType::is_captures(STAGE) {
            // stage must be quiets
            valid_squares = !occ;
        }

        if !GenType::is_quiet(STAGE) {
            // stage must be captures
            valid_squares &= their_pieces
        }

        let king_bb = self.piece_bb(Piece::K, us);
        let atk_mask = self.generate_atk_mask(them, occ ^ king_bb); // x-ray atks through king

        let legals = if atk_mask & king_bb != 0 {
            self.check_for_legals()
        } else {
            bitboard::FULL
        };

        // generate king moves
        {
            let from = bitboard::lsb(king_bb);
            let mut mask = atks::king_attacks(from);
            mask &= !atk_mask;
            mask &= valid_squares;

            while mask != 0 {
                let to = bitboard::pop_lsb(&mut mask);
                movelist.push(Action::new(from, to).into())
            }

            if legals == 0 {
                // if we are in double check, there are no legal moves and we return only king moves
                return;
            }

            if legals == bitboard::FULL && GenType::is_quiet(STAGE) {
                // we are not in check, so generate castling
                self.generate_castles(atk_mask, movelist)
            }
        }

        valid_squares &= legals;
        let bishop_pinmask = self.generate_bishop_pins();
        let rook_pinmask = self.generate_rook_pins();
        {
            // generate pawn moves

            let pawns = self.piece_bb(Piece::P, us);
            let b_pinned_pawns = pawns & bishop_pinmask;
            let r_pinned_pawns = pawns & rook_pinmask;
            let free_pawns = pawns ^ (b_pinned_pawns | r_pinned_pawns);
            let safe_capture = valid_squares & their_pieces;

            if GenType::is_captures(STAGE) {
                // promotions are cooler. generate them first.
                self.pawn_forwards::<true>(free_pawns, occ, legals, movelist);
                self.pawn_forwards::<true>(r_pinned_pawns, occ, legals & rook_pinmask, movelist);
                self.pawn_captures::<true>(free_pawns, safe_capture, movelist);
                self.pawn_captures::<true>(b_pinned_pawns, safe_capture & bishop_pinmask, movelist);
                self.pawn_captures::<false>(free_pawns, safe_capture, movelist);
                // likewise, bishop pinned pawns can capture if they are on the right squares
                self.pawn_captures::<false>(
                    b_pinned_pawns,
                    safe_capture & bishop_pinmask,
                    movelist,
                );
            }
            if GenType::is_quiet(STAGE) {
                self.pawn_forwards::<false>(free_pawns, occ, legals, movelist);
                // pawns pinned by a rook can move forward if it's still blocking
                self.pawn_forwards::<false>(r_pinned_pawns, occ, legals & rook_pinmask, movelist);
            }
        }

        {
            // generate knight moves
            let mut knights = self.piece_bb(Piece::N, us) & !rook_pinmask & !bishop_pinmask;
            // pinned knights can't move
            while knights != 0 {
                let from = bitboard::pop_lsb(&mut knights);
                let mut atks = atks::knight_attacks(from) & valid_squares;
                while atks != 0 {
                    let to = bitboard::pop_lsb(&mut atks);
                    movelist.push(Action::new(from, to).into())
                }
            }
        }

        {
            // generate bishop moves
            let mut bishops = self.diagonal_sliders(us) & !rook_pinmask; // bishops pinned by rooks can't move
            let mut pinned_bishops = bishops & bishop_pinmask;
            bishops ^= pinned_bishops;
            while bishops != 0 {
                let from = bitboard::pop_lsb(&mut bishops);
                let mut atks = atks::bishop_attacks(from, occ) & valid_squares;
                while atks != 0 {
                    let to = bitboard::pop_lsb(&mut atks);
                    movelist.push(Action::new(from, to).into())
                }
            }
            // pinned bishops can only travel along the pinned diagonal
            while pinned_bishops != 0 {
                let from = bitboard::pop_lsb(&mut pinned_bishops);
                let mut atks = atks::bishop_attacks(from, occ) & valid_squares & bishop_pinmask;

                while atks != 0 {
                    let to = bitboard::pop_lsb(&mut atks);
                    movelist.push(Action::new(from, to).into())
                }
            }
        }

        {
            // generate rook moves
            let mut rooks = self.orthogonal_sliders(us) & !bishop_pinmask; // rooks pinned by bishops can't move
            let mut pinned_rooks = rooks & rook_pinmask;
            rooks ^= pinned_rooks;

            while rooks != 0 {
                let from = bitboard::pop_lsb(&mut rooks);
                let mut atks = atks::rook_attacks(from, occ) & valid_squares;
                while atks != 0 {
                    let to = bitboard::pop_lsb(&mut atks);
                    movelist.push(Action::new(from, to).into())
                }
            }
            // pinned rooks can only travel along pinned rank or file
            while pinned_rooks != 0 {
                let from = bitboard::pop_lsb(&mut pinned_rooks);
                let mut atks = atks::rook_attacks(from, occ) & valid_squares & rook_pinmask;
                while atks != 0 {
                    let to = bitboard::pop_lsb(&mut atks);
                    movelist.push(Action::new(from, to).into())
                }
            }
        }

        if GenType::is_captures(STAGE) {
            self.generate_passant(movelist)
        }
    }

    fn generate_castles(&self, atk_mask: Bitboard, movelist: &mut MoveList) {
        let us = self.active_color();
        let occ = self.occupancy();
        // we are not in check, so generate castling
        let kingside_rights = select!(us, self.castling(Castling::WK), self.castling(Castling::BK));
        let queenside_rights =
            select!(us, self.castling(Castling::WQ), self.castling(Castling::BQ));
        // TODO
        if *kingside_rights {
            const BLOCKED_KINGSIDE_W: Bitboard = new_bbs!(square::G1, square::F1);
            const BLOCKED_KINGSIDE_B: Bitboard = new_bbs!(square::G8, square::F8);
            let block_bb = select!(us, BLOCKED_KINGSIDE_W, BLOCKED_KINGSIDE_B);

            // checks if castling is obstructed by pieces or check
            let is_safe = block_bb & occ == 0 && block_bb & atk_mask == 0;
            if is_safe {
                let king_from = select!(us, square::E1, square::E8);
                let king_to = select!(us, square::G1, square::G8);
                movelist.push(Action::new_type(king_from, king_to, MoveType::Castle).into())
            }
        }

        if *queenside_rights {
            const CHECKED_QUEENSIDE_W: Bitboard = new_bbs!(square::D1, square::C1);
            const CHECKED_QUEENSIDE_B: Bitboard = new_bbs!(square::D8, square::C8);
            const BLOCKED_QUEENSIDE_W: Bitboard = new_bbs!(square::D1, square::C1, square::B1);
            const BLOCKED_QUEENSIDE_B: Bitboard = new_bbs!(square::D8, square::C8, square::B8);

            let block_bb = select!(us, BLOCKED_QUEENSIDE_W, BLOCKED_QUEENSIDE_B);
            let check_bb = select!(us, CHECKED_QUEENSIDE_W, CHECKED_QUEENSIDE_B);
            // checks if castling is obstructed by pieces or check
            let is_safe = block_bb & occ == 0 && check_bb & atk_mask == 0;
            if is_safe {
                let king_from = select!(us, square::E1, square::E8);
                let king_to = select!(us, square::C1, square::C8);
                movelist.push(Action::new_type(king_from, king_to, MoveType::Castle).into())
            }
        }
    }
    fn generate_passant(&mut self, movelist: &mut MoveList) {
        let us = self.active_color();
        let them = !us;
        let passant_square = self.passant_square();
        if let Some(passant_square) = passant_square {
            let passant_bb = bitboard::new_bb(passant_square);
            let shifted = bitboard::forward(passant_bb, them);
            let mut passantables =
                bitboard::shift(shifted, Direction::E) | bitboard::shift(shifted, Direction::W); // squares of pawns that can potentially capture passant
            passantables &= self.piece_bb(Piece::P, us);
            while passantables != 0 {
                let from = bitboard::pop_lsb(&mut passantables);
                let passant_move = Action::new_type(from, passant_square, MoveType::Passant);
                // verify that the passant move is legal
                self.make_move(passant_move);

                if !self.in_check(us) {
                    movelist.push(passant_move.into());
                }
                self.unmake_move(passant_move)
            }
        }
    }
    fn pawn_captures<const PROMOTIONS: bool>(
        &self,
        pawns: Bitboard,
        safe_squares: Bitboard,
        movelist: &mut MoveList,
    ) {
        let us = self.active_color();
        let eighth_rank = select!(us, bitboard::rank_bb(7), bitboard::rank_bb(0));

        let forward = bitboard::forward(pawns, us)
            & if PROMOTIONS {
                eighth_rank
            } else {
                !eighth_rank
            }; // promotions are generated seperately
        let mut east_capture = bitboard::shift(forward, Direction::E) & safe_squares;
        let mut west_capture = bitboard::shift(forward, Direction::W) & safe_squares;

        while east_capture != 0 {
            let to = bitboard::pop_lsb(&mut east_capture);
            let from = select!(
                us,
                square::shift(to, Direction::SW),
                square::shift(to, Direction::NW)
            );
            if PROMOTIONS {
                movelist.push(Action::new_pr(from, to, Piece::Q).into());
                movelist.push(Action::new_pr(from, to, Piece::N).into());
                movelist.push(Action::new_pr(from, to, Piece::R).into());
                movelist.push(Action::new_pr(from, to, Piece::B).into());
            } else {
                movelist.push(Action::new(from, to).into())
            }
        }

        while west_capture != 0 {
            let to = bitboard::pop_lsb(&mut west_capture);
            let from = select!(
                us,
                square::shift(to, Direction::SE),
                square::shift(to, Direction::NE)
            );
            if PROMOTIONS {
                movelist.push(Action::new_pr(from, to, Piece::Q).into());
                movelist.push(Action::new_pr(from, to, Piece::N).into());
                movelist.push(Action::new_pr(from, to, Piece::R).into());
                movelist.push(Action::new_pr(from, to, Piece::B).into());
            } else {
                movelist.push(Action::new(from, to).into())
            }
        }
    }

    fn pawn_forwards<const PROMOTIONS: bool>(
        &self,
        pawns: Bitboard,
        occupancy: Bitboard,
        legals: Bitboard,
        movelist: &mut MoveList,
    ) {
        let us = self.active_color();
        let eighth_rank = select!(us, bitboard::rank_bb(7), bitboard::rank_bb(0));
        let fourth_rank = select!(us, bitboard::rank_bb(3), bitboard::rank_bb(4));
        let mut forward = bitboard::forward(pawns, us)
            & !occupancy
            & if PROMOTIONS {
                eighth_rank
            } else {
                !eighth_rank
            }; // promotions are generated seperately
        let mut push_two = bitboard::forward(forward, us) & !occupancy & fourth_rank;

        forward &= legals;
        push_two &= legals;
        while forward != 0 {
            let to = bitboard::pop_lsb(&mut forward);
            let from = select!(
                us,
                square::shift(to, Direction::S),
                square::shift(to, Direction::N)
            );

            if PROMOTIONS {
                movelist.push(Action::new_pr(from, to, Piece::Q).into());
                movelist.push(Action::new_pr(from, to, Piece::N).into());
                movelist.push(Action::new_pr(from, to, Piece::R).into());
                movelist.push(Action::new_pr(from, to, Piece::B).into());
            } else {
                movelist.push(Action::new(from, to).into())
            }
        }

        // dont waste time on this when generating promotions
        if !PROMOTIONS {
            while push_two != 0 {
                let to = bitboard::pop_lsb(&mut push_two);
                let from = select!(
                    us,
                    square::shift(square::shift(to, Direction::S), Direction::S),
                    square::shift(square::shift(to, Direction::N), Direction::N)
                );

                movelist.push(Action::new(from, to).into())
            }
        }
    }

    // generate squares where pinned pieces can move to
    fn generate_rook_pins(&self) -> Bitboard {
        let us = self.active_color();
        let them = !us;
        let king_square = bitboard::lsb(self.piece_bb(Piece::K, us));
        let mut our_pieces = self.color_bb(us);
        let their_pieces = self.color_bb(them);
        let occ = our_pieces | their_pieces;

        our_pieces &= !atks::rook_attacks(king_square, occ); // clear out any potential pinned pieces
        let their_attackers = self.orthogonal_sliders(them); // then, x-ray attack to see if there are any pinners
        let mut new_attackers =
            atks::rook_attacks(king_square, our_pieces | their_pieces) & their_attackers;

        let mut pinmask = new_attackers;
        while new_attackers != 0 {
            pinmask |= atks::in_btwn_atks(king_square, bitboard::pop_lsb(&mut new_attackers))
        }
        pinmask
    }

    fn generate_bishop_pins(&self) -> Bitboard {
        let us = self.active_color();
        let them = !us;
        let king_square = bitboard::lsb(self.piece_bb(Piece::K, us));
        let mut our_pieces = self.color_bb(us);
        let their_pieces = self.color_bb(them);
        let occ = our_pieces | their_pieces;

        our_pieces &= !atks::bishop_attacks(king_square, occ);
        let their_attackers = self.diagonal_sliders(them);
        let mut new_attackers =
            atks::bishop_attacks(king_square, our_pieces | their_pieces) & their_attackers;

        let mut pinmask = new_attackers;
        while new_attackers != 0 {
            pinmask |= atks::in_btwn_atks(king_square, bitboard::pop_lsb(&mut new_attackers))
        }
        pinmask
    }

    // if we are in check, generate a set of moves that we can move out of check with
    fn check_for_legals(&self) -> Bitboard {
        let us = self.active_color();
        let them = !us;

        let occ = self.occupancy();
        let king_square = bitboard::lsb(self.piece_bb(Piece::K, us));
        let knight_attackers = atks::knight_attacks(king_square) & self.piece_bb(Piece::N, them); // determine which knights are checking king
        let forward_pawns = bitboard::forward(self.piece_bb(Piece::K, us), us);
        let pawn_attackers = (bitboard::shift(forward_pawns, Direction::E) // determine which pawns are checking king
            | bitboard::shift(forward_pawns, Direction::W))
            & self.piece_bb(Piece::P, them);

        let jumpers = pawn_attackers | knight_attackers; // since jumpers cannot be blocked, these need to be considered

        let orthogonal_attackers =
            atks::rook_attacks(king_square, occ) & self.orthogonal_sliders(them);
        let diagonal_attackers =
            atks::bishop_attacks(king_square, occ) & self.diagonal_sliders(them);

        let sliders = orthogonal_attackers | diagonal_attackers; // these can be blocked

        let num_checkers = bitboard::popcount(sliders | jumpers);

        match num_checkers.cmp(&1) {
            std::cmp::Ordering::Greater => {
                // more than 1 checker, only moves are king moves
                0
            }
            std::cmp::Ordering::Equal => {
                // there is exactly one checker
                if sliders != 0 {
                    // if there are sliders, find pieces that can block
                    let in_between = atks::in_btwn_atks(bitboard::lsb(sliders), king_square);
                    in_between | sliders // Block or capture moves
                } else {
                    // if no sliders, consider jumpers that can only be captured
                    jumpers
                }
            }
            std::cmp::Ordering::Less => {
                // no checkers, all moves are possible
                bitboard::FULL
            }
        }
    }

    fn generate_atk_mask(&self, color: Color, occupancy: Bitboard) -> Bitboard {
        let mut base = 0;
        // pawns attacks can be calculated just by shifting the pawns diagonally
        let pawns = self.piece_bb(Piece::P, color);
        let forward_pawns = bitboard::forward(pawns, color);
        base |= bitboard::shift(forward_pawns, Direction::E)
            | bitboard::shift(forward_pawns, Direction::W);

        // knights
        let mut knights = self.piece_bb(Piece::N, color);
        while knights != 0 {
            let square = bitboard::pop_lsb(&mut knights);
            base |= atks::knight_attacks(square)
        }

        // orthogonal sliders
        let mut orth_sliders = self.orthogonal_sliders(color);
        while orth_sliders != 0 {
            let square = bitboard::pop_lsb(&mut orth_sliders);
            base |= atks::rook_attacks(square, occupancy)
        }

        // diagonal sliders
        let mut diag_sliders = self.diagonal_sliders(color);
        while diag_sliders != 0 {
            let square = bitboard::pop_lsb(&mut diag_sliders);
            base |= atks::bishop_attacks(square, occupancy)
        }

        // kings
        let king_square = bitboard::lsb(self.piece_bb(Piece::K, color));
        base |= atks::king_attacks(king_square);

        base
    }

    pub fn in_check(&self, color: Color) -> bool {
        let king_square = bitboard::lsb(self.piece_bb(Piece::K, color));
        self.is_attacked(king_square, !color)
    }
    // check if square is attacked by color
    fn is_attacked(&self, square: Square, color: Color) -> bool {
        let occupancy = self.occupancy();

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

        (kings & king_atks)
            | (pawns & pawn_atks)
            | (knights & knight_atks)
            | (orthogonals & rook_atks)
            | (diagonals & bishop_atks)
            != 0
    }
}
