
use once_cell::sync::Lazy;
use rand::{rngs::StdRng, RngCore, SeedableRng};

use crate::{
    board_state::{bitboard::Bitboard, typedefs::Square},
    move_generation::masks::initialize_rook_atkmask,
};

use super::masks::{initialize_bishop_atkmask, rook_endpoints};

pub static ROOK_TABLES: Lazy<Table> = Lazy::new(fill_rook_table);
pub static BISHOP_TABLES: Lazy<Table> = Lazy::new(fill_bishop_table);

pub const ROOK_MAGIC_SIZE: usize = 102400;
pub const BISHOP_MAGIC_SIZE: usize = 5248;

// Magic Bitboards for sliding piece attacks. https://www.chessprogramming.org/Magic_Bitboards

pub fn rook_attacks(square: Square, occupancy: Bitboard) -> Bitboard {
    let magic = ROOK_TABLES.magics[square as usize];
    let idx = magic.generate_index(occupancy);
    ROOK_TABLES.atks[idx]
}

pub fn bishop_attacks(square: Square, occupancy: Bitboard) -> Bitboard {
    let magic = BISHOP_TABLES.magics[square as usize];
    let idx = magic.generate_index(occupancy);
    BISHOP_TABLES.atks[idx]
}

#[derive(Clone, Copy)]
pub struct Magic {
    pub mask: Bitboard,
    pub shift: u8,
    pub location: u64, // where the relevant values are within the table
    pub magic_number: u64,
}

impl Magic {
    #[inline(always)]
    pub fn generate_index(&self, occupancy: Bitboard) -> usize {
        cfg_if::cfg_if! {
            if #[cfg(target_feature = "bmi2")]{
                return _pext_u64(occupancy,self.mask) as usize;
            } else {
                let mut occ = occupancy;
                occ &= self.mask;
                occ *= self.magic_number;
                occ >>= self.shift;
                occ += self.location;
                occ as usize
            }
        }
    }
}

pub struct Table {
    atks: Vec<Bitboard>,
    magics: [Magic; 64],
}

fn fill_rook_table() -> Table {
    let mut atks = vec![0; ROOK_MAGIC_SIZE];
    let mut magics = [Magic {
        mask: 0,
        shift: 0,
        location: 0,
        magic_number: 0,
    }; 64];
    let mut offset = 0;
    for i in 0..64 {
        let mask = ROOK_MASKS[i];
        let bits = mask.count_ones();
        let newmagic = Magic {
            mask,
            shift: 64 - (bits as u8),
            location: offset,
            magic_number: ROOK_MAGICS[i],
        };
        magics[i] = newmagic;
        for submask in PermutationCounter::new(mask) {
            let idx = newmagic.generate_index(submask);
            atks[idx] = initialize_rook_atkmask(submask, i as u8);
            offset += 1;
        }
    }
    assert_eq!(offset as usize, ROOK_MAGIC_SIZE);
    Table { atks, magics }
}

fn fill_bishop_table() -> Table {
    let mut atks = vec![0; BISHOP_MAGIC_SIZE];
    let mut magics = [Magic {
        mask: 0,
        shift: 0,
        location: 0,
        magic_number: 0,
    }; 64];
    let mut offset = 0;
    for i in 0..64 {
        let mask = BISHOP_MASKS[i];
        let bits = mask.count_ones();
        let newmagic = Magic {
            mask,
            shift: 64 - (bits as u8),
            location: offset,
            magic_number: BISHOP_MAGICS[i],
        };
        magics[i] = newmagic;
        for submask in PermutationCounter::new(mask) {
            let idx = newmagic.generate_index(submask);
            atks[idx] = initialize_bishop_atkmask(submask, i as u8);
            offset += 1;
        }
    }
    assert_eq!(offset as usize, BISHOP_MAGIC_SIZE);
    Table { atks, magics }
}

pub fn find_rook_magics() {
    let mut random = StdRng::from_entropy();
    let mut rook_table = vec![0; ROOK_MAGIC_SIZE];
    let mut magics = [Magic {
        mask: 0,
        shift: 0,
        location: 0,
        magic_number: 0,
    }; 64];
    let mut offset = 0;
    for i in 0..64 {
        let mask = ROOK_MASKS[i];
        let bits = mask.count_ones();
        let end = offset + (1 << bits) - 1;
        let mut found = false;

        let mut try_magic = Magic {
            mask,
            shift: 64 - bits as u8,
            location: offset,
            magic_number: 0,
        };

        while !found {
            found = true;
            try_magic.magic_number = random.next_u64() & random.next_u64() & random.next_u64();
            for submask in PermutationCounter::new(mask) {
                let idx = try_magic.generate_index(submask);

                let movemask = initialize_rook_atkmask(submask, i as u8);
                if rook_table[idx] == 0 || rook_table[idx] == movemask {
                    rook_table[idx] = movemask;
                } else {
                    // failed
                    for index in offset..=end {
                        rook_table[index as usize] = 0;
                    }
                    found = false;
                    break;
                }
            }
        }
        offset += 1 << bits;
        magics[i] = try_magic;
    }

    for (idx, magic) in magics.iter().enumerate() {
        if idx % 4 == 0 {
            println!("  ");
        }
        let num = magic.magic_number;
        print!("{num:#x}, ");
    }
    assert_eq!(offset as usize, ROOK_MAGIC_SIZE);
}

pub fn find_bishop_magics() {
    let mut random = StdRng::from_entropy();
    let mut bishop_table = vec![0; BISHOP_MAGIC_SIZE];
    let mut magics = [Magic {
        mask: 0,
        shift: 0,
        location: 0,
        magic_number: 0,
    }; 64];
    let mut offset = 0;
    for i in 0..64 {
        let mask = BISHOP_MASKS[i];
        let bits = mask.count_ones();
        let end = offset + (1 << bits) - 1;
        let mut found = false;
        let mut try_magic = Magic {
            mask,
            shift: 64 - bits as u8,
            location: offset,
            magic_number: 0,
        };

        while !found {
            found = true;
            try_magic.magic_number = random.next_u64() & random.next_u64() & random.next_u64();
            for submask in PermutationCounter::new(mask) {
                let idx = try_magic.generate_index(submask);
                let movemask = initialize_bishop_atkmask(submask, i as u8);
                if bishop_table[idx] == 0 || bishop_table[idx] == movemask {
                    bishop_table[idx] = movemask;
                } else {
                    // failed
                    for index in offset..=end {
                        bishop_table[index as usize] = 0;
                    }
                    found = false;
                    break;
                }
            }
        }
        magics[i] = try_magic;
        offset += 1 << bits;
    }
    for (idx, magic) in magics.iter().enumerate() {
        if idx % 4 == 0 {
            println!("  ");
        }
        let num = magic.magic_number;
        print!("{num:#x}, ");
    }
    assert_eq!(offset as usize, BISHOP_MAGIC_SIZE);
}

#[rustfmt::skip]
static ROOK_MAGICS: [u64; 64] = [
    0x80041040002280, 0x140001000200840, 0x180100084200008, 0x4900090004205002, 
    0x68800a8004000800, 0x100040001000842, 0x100020011000284, 0x2a000048840d2201,
    0x1004801021814000, 0x2008400020015000, 0x400802000821008, 0x8001000901201000,
    0x801000501100800, 0x5002001002001458, 0x3c00700608010c, 0x8400800041002180,
    0x80044001c82000, 0x10004002200040, 0x41818050002004, 0x10210009041001,
    0x1010008000410, 0x4001010002080c00, 0x84188c0001108a28, 0x400020023044084,
    0x80014040002004, 0x2121010040008c, 0x84a0002080100080, 0x80080100080,
    0x40080080080, 0x1052000600104428, 0x24401040010080a, 0x29200040261,
    0x80104000808004a0, 0xa1200840401000, 0x40100180802000, 0x2900180800804,
    0xa004000800800482, 0x2000400810080, 0xc000221084000801, 0x1001a446000403,
    0x4340022480418000, 0x500020014000, 0x801208600420012, 0x691001910010021,
    0x4041080045010030, 0x420004008080, 0x220020b10040008, 0xc080018041020004,
    0x6040308201004200, 0x4000802010400080, 0x401100a00100, 0x50011804004040,
    0x800d021004080100, 0x20048004008a0080, 0x1000080122502400, 0x40340c0945248200,
    0x108002c058810021, 0x8508200430022, 0x40220151800842, 0x400028b001000521,
    0x48000500061003, 0x25001400860821, 0x428502481004, 0x1000209408027,
];

#[rustfmt::skip]
static BISHOP_MAGICS: [u64; 64] = [
    0x288284180820200, 0x110040104002980, 0x50050041000000, 0x2004040482002000, 
    0x40051040a0000000, 0x91602100c201028, 0x2824010802500004, 0x4000108c14200401,
    0x401020982a480845, 0x1012123c05020202, 0x110100c00803100, 0x20001220820000c0,
    0x1828084840023800, 0x214886008020a, 0x1b8088084100, 0x41048482080a0200,
    0x10202002422804, 0x20208842508602, 0x104a008408031100, 0xb40c800802004000,
    0x2405015820080040, 0x880800510101100, 0x168064041c0202, 0x28808304008200,
    0x420ac0020380200, 0x20a25006a0010200, 0x2208080014024810, 0x64010000490100,
    0x2888840002020204, 0x8000902002021000, 0x88a00008804a0, 0xc10040010c00bc,
    0x281820100a160401, 0xa48880440091040, 0x1012010400404840, 0xa000020180480081,
    0x158082400004100, 0x2116088300020440, 0x804810200024800, 0x14850612802202d0,
    0x221888840000814, 0x9604044402002404, 0x71010a0092029000, 0x880441420181c800,
    0x20202008800501, 0x2101010102014500, 0xc108016806804202, 0x4010410109040420,
    0x5012802c00410, 0x1c0c844100800, 0x90110047100000, 0x4040110840c0220,
    0x4001004045010004, 0x100250010018, 0x10200103020000, 0x8004010204030011,
    0x4001002704024040, 0x8000c9400880400, 0x8002114021080800, 0x4002040420200,
    0x800400004104400, 0x401022004500080, 0x8430540800a402, 0x40184200424100,
];

static ROOK_MASKS: Lazy<[Bitboard; 64]> = Lazy::new(generate_rookmasks);
static BISHOP_MASKS: Lazy<[Bitboard; 64]> = Lazy::new(generate_bishopmasks);

pub fn generate_rookmasks() -> [Bitboard; 64] {
    let mut arr = [0; 64];
    for (i, mask) in arr.iter_mut().enumerate() {
        *mask = initialize_rook_atkmask(0, i as u8) & !rook_endpoints(i as u8);
    }
    arr
}

pub fn generate_bishopmasks() -> [Bitboard; 64] {
    let mut arr = [0; 64];
    for (i, mask) in arr.iter_mut().enumerate() {
        *mask = (initialize_bishop_atkmask(0, i as u8)) & 0x7e7e7e7e7e7e00;
    }
    arr
}

// permutationcounter is an iterator that goes through all sub-masks of a masks.
// A submask of bitboard A is a bitboard B such that B & A = B. So for example,
/* (Using 4x4 16-bit "bitboards" for simplicity)
Let's suppose this is the bitboard A
    . . . .
    . . 1 .
    . 1 1 .
    . . . .

    potential submasks of A are:
    . . . .
    . . 1 .
    . 1 0 .
    . . . .

    . . . .
    . . 0 .
    . 0 1 .
    . . . .

    etc. etc.
*/
// using basic combinatorics we can derive that there are 2^n submasks for a mask with n bits.
pub struct PermutationCounter {
    pub starting_bb: u64,
    pub current_bb: u64,
}

impl PermutationCounter {
    pub fn new(bb: u64) -> PermutationCounter {
        PermutationCounter {
            starting_bb: bb,
            current_bb: bb,
        }
    }
}

impl Iterator for PermutationCounter {
    type Item = u64;
    fn next(&mut self) -> Option<Self::Item> {
        if self.current_bb == 0 {
            self.current_bb = u64::MAX;
            Some(0)
        } else if self.current_bb == u64::MAX {
            None
        } else {
            // carry rippler
            let prev_bb = self.current_bb;
            self.current_bb -= 1;
            self.current_bb &= self.starting_bb;
            Some(prev_bb)
        }
    }
}
