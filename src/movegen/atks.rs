use self::{
    in_betweens::IN_BTWN_ATKS,
    jumpers::{KING_ATTACKS, KNIGHT_ATTACKS},
    magics::{BISHOP_MAGICS, ROOK_MAGICS},
};

use super::{bitboard::Bitboard, types::Square};

// interface functions
pub fn bishop_attacks(square: Square, occupancy: Bitboard) -> Bitboard {
    BISHOP_MAGICS[square as usize][occupancy]
}

pub fn rook_attacks(square: Square, occupancy: Bitboard) -> Bitboard {
    ROOK_MAGICS[square as usize][occupancy]
}

pub fn knight_attacks(square: Square) -> Bitboard {
    KNIGHT_ATTACKS[square as usize]
}

pub fn king_attacks(square: Square) -> Bitboard {
    KING_ATTACKS[square as usize]
}

pub fn in_btwn_atks(square1: Square, square2: Square) -> Bitboard {
    IN_BTWN_ATKS[square1 as usize][square2 as usize]
}

mod magics {
    use std::{
        array,
        ops::{Index, IndexMut},
    };

    use static_init::dynamic;

    use super::super::{
        bitboard::{self, Bitboard, Direction},
        types::{square, Piece, Square},
    };

    const B_MAGIC_NUMBERS: [u64; 64] = [
        0x288284180820200,
        0x110040104002980,
        0x50050041000000,
        0x2004040482002000,
        0x40051040a0000000,
        0x91602100c201028,
        0x2824010802500004,
        0x4000108c14200401,
        0x401020982a480845,
        0x1012123c05020202,
        0x110100c00803100,
        0x20001220820000c0,
        0x1828084840023800,
        0x214886008020a,
        0x1b8088084100,
        0x41048482080a0200,
        0x10202002422804,
        0x20208842508602,
        0x104a008408031100,
        0xb40c800802004000,
        0x2405015820080040,
        0x880800510101100,
        0x168064041c0202,
        0x28808304008200,
        0x420ac0020380200,
        0x20a25006a0010200,
        0x2208080014024810,
        0x64010000490100,
        0x2888840002020204,
        0x8000902002021000,
        0x88a00008804a0,
        0xc10040010c00bc,
        0x281820100a160401,
        0xa48880440091040,
        0x1012010400404840,
        0xa000020180480081,
        0x158082400004100,
        0x2116088300020440,
        0x804810200024800,
        0x14850612802202d0,
        0x221888840000814,
        0x9604044402002404,
        0x71010a0092029000,
        0x880441420181c800,
        0x20202008800501,
        0x2101010102014500,
        0xc108016806804202,
        0x4010410109040420,
        0x5012802c00410,
        0x1c0c844100800,
        0x90110047100000,
        0x4040110840c0220,
        0x4001004045010004,
        0x100250010018,
        0x10200103020000,
        0x8004010204030011,
        0x4001002704024040,
        0x8000c9400880400,
        0x8002114021080800,
        0x4002040420200,
        0x800400004104400,
        0x401022004500080,
        0x8430540800a402,
        0x40184200424100,
    ];

    const R_MAGIC_NUMBERS: [u64; 64] = [
        0x80041040002280,
        0x140001000200840,
        0x180100084200008,
        0x4900090004205002,
        0x68800a8004000800,
        0x100040001000842,
        0x100020011000284,
        0x2a000048840d2201,
        0x1004801021814000,
        0x2008400020015000,
        0x400802000821008,
        0x8001000901201000,
        0x801000501100800,
        0x5002001002001458,
        0x3c00700608010c,
        0x8400800041002180,
        0x80044001c82000,
        0x10004002200040,
        0x41818050002004,
        0x10210009041001,
        0x1010008000410,
        0x4001010002080c00,
        0x84188c0001108a28,
        0x400020023044084,
        0x80014040002004,
        0x2121010040008c,
        0x84a0002080100080,
        0x80080100080,
        0x40080080080,
        0x1052000600104428,
        0x24401040010080a,
        0x29200040261,
        0x80104000808004a0,
        0xa1200840401000,
        0x40100180802000,
        0x2900180800804,
        0xa004000800800482,
        0x2000400810080,
        0xc000221084000801,
        0x1001a446000403,
        0x4340022480418000,
        0x500020014000,
        0x801208600420012,
        0x691001910010021,
        0x4041080045010030,
        0x420004008080,
        0x220020b10040008,
        0xc080018041020004,
        0x6040308201004200,
        0x4000802010400080,
        0x401100a00100,
        0x50011804004040,
        0x800d021004080100,
        0x20048004008a0080,
        0x1000080122502400,
        0x40340c0945248200,
        0x108002c058810021,
        0x8508200430022,
        0x40220151800842,
        0x400028b001000521,
        0x48000500061003,
        0x25001400860821,
        0x428502481004,
        0x1000209408027,
    ];

    #[dynamic]
    pub(super) static ROOK_MAGICS: [Magic; 64] =
        array::from_fn(|square| Magic::new(square as Square, Piece::R));

    #[dynamic]
    pub(super) static BISHOP_MAGICS: [Magic; 64] =
        array::from_fn(|square| Magic::new(square as Square, Piece::B));

    #[derive(Default, Clone)]
    pub(super) struct Magic {
        attacks: Vec<Bitboard>,
        mask: Bitboard,
        magic_number: u64,
        shift: u8,
    }

    fn rook_mask(square: Square) -> Bitboard {
        let file = square::file_of(square);
        let rank = square::rank_of(square);
        let endpoints = bitboard::new_bb(file)
            | bitboard::new_bb(square::flip_v(file))
            | bitboard::new_bb(rank * 8)
            | bitboard::new_bb(square::flip_h(rank * 8));
        ((bitboard::file_bb_of(square) | bitboard::rank_bb_of(square)) & !endpoints)
            & !bitboard::new_bb(square)
    }

    fn bishop_mask(square: Square) -> Bitboard {
        const ENDPOINTS: Bitboard = 0xff818181818181ff;
        ((bitboard::antidiagonal_bb(square) | bitboard::diagonal_bb(square)) & !ENDPOINTS)
            & !bitboard::new_bb(square)
    }

    fn dumb_attacks(square: Square, occupancy: Bitboard, piece: Piece) -> Bitboard {
        let mut moves = 0;
        let free = !occupancy;

        let mut direction_bitboards = [bitboard::new_bb(square); 4];

        let shift_directions = if piece == Piece::R {
            [Direction::N, Direction::S, Direction::E, Direction::W]
        } else if piece == Piece::B {
            [Direction::NE, Direction::SE, Direction::NW, Direction::SW]
        } else {
            panic!("This shouldn't happen")
        };

        while direction_bitboards.iter().any(|&a| a != 0) {
            direction_bitboards
                .iter_mut()
                .enumerate()
                .for_each(|(index, bb)| *bb = bitboard::shift(*bb, shift_directions[index]));

            moves |= direction_bitboards.iter().fold(0, |a, b| a | b);
            direction_bitboards.iter_mut().for_each(|a| *a &= free);
        }
        moves
    }

    impl Index<Bitboard> for Magic {
        type Output = Bitboard;

        fn index(&self, occupancy: Bitboard) -> &Self::Output {
            &self.attacks[self.compute_index(occupancy)]
        }
    }

    impl IndexMut<Bitboard> for Magic {
        fn index_mut(&mut self, occupancy: Bitboard) -> &mut Self::Output {
            let index = self.compute_index(occupancy);
            &mut self.attacks[index]
        }
    }

    impl Magic {
        fn compute_index(&self, mut occupancy: Bitboard) -> usize {
            occupancy &= self.mask;
            occupancy = occupancy.wrapping_mul(self.magic_number);
            occupancy >>= self.shift;
            occupancy as usize
        }

        fn new(square: Square, piece: Piece) -> Magic {
            let (magic_number, mask) = if piece == Piece::R {
                (R_MAGIC_NUMBERS[square as usize], rook_mask(square))
            } else if piece == Piece::B {
                (B_MAGIC_NUMBERS[square as usize], bishop_mask(square))
            } else {
                panic!("This shouldn't happen")
            };

            let mask_pop = bitboard::popcount(mask);
            let shift = 64 - mask_pop;
            let permutations = 1 << mask_pop;
            let attacks = vec![0; permutations];

            let mut magic = Magic {
                attacks,
                mask,
                magic_number,
                shift,
            };
            let mut occupancy = 0;
            loop {
                let atk_bb = dumb_attacks(square, occupancy, piece);
                magic[occupancy] = atk_bb;
                occupancy = (occupancy.wrapping_sub(mask)) & mask;
                if occupancy == 0 {
                    break;
                }
            }
            magic
        }
    }
}

mod jumpers {
    use std::array;

    use super::{
        super::bitboard::{self, Bitboard, Direction},
        Square,
    };

    use bitboard::shift;
    use static_init::dynamic;

    #[dynamic]
    pub(super) static KING_ATTACKS: [Bitboard; 64] =
        array::from_fn(|square| dumb_king_attacks(square as Square));

    #[dynamic]
    pub(super) static KNIGHT_ATTACKS: [Bitboard; 64] =
        array::from_fn(|square| dumb_knight_attacks(square as Square));

    fn dumb_king_attacks(square: Square) -> Bitboard {
        let square_bb = bitboard::new_bb(square);
        shift(square_bb, Direction::N)
            | shift(square_bb, Direction::S)
            | shift(square_bb, Direction::E)
            | shift(square_bb, Direction::W)
            | shift(square_bb, Direction::NE)
            | shift(square_bb, Direction::NW)
            | shift(square_bb, Direction::SE)
            | shift(square_bb, Direction::SW)
    }

    fn dumb_knight_attacks(square: Square) -> Bitboard {
        let square_bb = bitboard::new_bb(square);
        shift(shift(square_bb, Direction::N), Direction::NE)
            | shift(shift(square_bb, Direction::N), Direction::NW)
            | shift(shift(square_bb, Direction::S), Direction::SE)
            | shift(shift(square_bb, Direction::S), Direction::SW)
            | shift(shift(square_bb, Direction::E), Direction::NE)
            | shift(shift(square_bb, Direction::E), Direction::SE)
            | shift(shift(square_bb, Direction::W), Direction::NW)
            | shift(shift(square_bb, Direction::W), Direction::SW)
    }
}

mod in_betweens {
    use static_init::dynamic;

    use crate::movegen::{bitboard::Bitboard, types::Square};
    #[dynamic]
    pub(super) static IN_BTWN_ATKS: [[Bitboard; 64]; 64] = generate_array();

    fn generate_array() -> [[Bitboard; 64]; 64] {
        let mut value = [[0; 64]; 64];
        for sq1 in 0..64 {
            for sq2 in 0..64 {
                value[sq1 as usize][sq2 as usize] = in_between_rays(sq1, sq2)
            }
        }
        value
    }

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
}
