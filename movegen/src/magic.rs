use aether_types::{BitBoard, File, Rank, Square};

#[rustfmt::skip]
const ROOK_RELEVANT_BITS: [u8; 64] = [
    12, 11, 11, 11, 11, 11, 11, 12,
    11, 10, 10, 10, 10, 10, 10, 11,
    11, 10, 10, 10, 10, 10, 10, 11,
    11, 10, 10, 10, 10, 10, 10, 11,
    11, 10, 10, 10, 10, 10, 10, 11,
    11, 10, 10, 10, 10, 10, 10, 11,
    11, 10, 10, 10, 10, 10, 10, 11,
    12, 11, 11, 11, 11, 11, 11, 12,
];

#[rustfmt::skip]
const BISHOP_RELEVANT_BITS: [u8; 64] = [
    6, 5, 5, 5, 5, 5, 5, 6,
    5, 5, 5, 5, 5, 5, 5, 5,
    5, 5, 7, 7, 7, 7, 5, 5,
    5, 5, 7, 9, 9, 7, 5, 5,
    5, 5, 7, 9, 9, 7, 5, 5,
    5, 5, 7, 7, 7, 7, 5, 5,
    5, 5, 5, 5, 5, 5, 5, 5,
    6, 5, 5, 5, 5, 5, 5, 6,
];

#[rustfmt::skip]
pub const BISHOP_MAGICS: [u64; 64] = [
    0x284001020881010e, 0x180401040c248082, 0x1004080095080002, 0x0204042381408042,
    0x0004042200800100, 0x0301100804040100, 0x0044480411083000, 0x0010140402021004,
    0x0001400404140462, 0x022004240082020c, 0x0020106082004000, 0x000034041c848240,
    0x1c00011040000a00, 0x0400050108c20040, 0x1100208801082108, 0x0021020100821008,
    0x2208802020c400a0, 0x0010000410420042, 0x0189002888010100, 0x04480100820040a0,
    0x0109005190401008, 0x2040200410080820, 0x0802010101012008, 0x6102000822020206,
    0x00282020c0130608, 0x0012221008109400, 0x1004280030004048, 0x0038080010220021,
    0x0022002012008042, 0x0100820807012000, 0x4210910002091010, 0x0104092001908400,
    0x8490103200142400, 0x8094903000091207, 0x6004040200040830, 0x0000020080480080,
    0x0040010010410041, 0x8002008202040a04, 0x2002080040c12426, 0x0042040110004040,
    0x8003091040585048, 0x0022091c20032300, 0x0088c0640d001004, 0x0020002011001800,
    0x0c10401812000040, 0x0040104040800040, 0x0004500089204604, 0x2004210200208202,
    0x0884040288440000, 0x0000411808030400, 0x1000011041104042, 0x0121848020880010,
    0x0108084420920005, 0x2190a0610c0b2058, 0x0088108420840a07, 0x92549c0802002400,
    0x8100820110010405, 0x4000003088080801, 0x0080104121082200, 0x2431000000420204,
    0x0804000010820200, 0x003001a021020080, 0x4140420828110040, 0x50a0200400408420,
];

#[rustfmt::skip]
pub const ROOK_MAGICS: [u64; 64] = [
    0x2180002a80104002, 0x2840004010002002, 0x010008c431006000, 0x1080080080100004,
    0x2080028108000400, 0x0e00100842000401, 0x0080010000800200, 0x0200144200810024,
    0x01a9800040023080, 0x8000804000802000, 0x0801002001001040, 0x0002000814220040,
    0x0001000801001004, 0x8001000400080300, 0x00d3008200041100, 0x040480018008c500,
    0x0080004000200042, 0x1122270040030080, 0xc008420013820320, 0x2420090010010020,
    0x4000808004000800, 0x8099010008020400, 0x8000040010010802, 0x0224020001008044,
    0x8010800080204006, 0x00115000c0002000, 0x0000804200102200, 0x8500220900100100,
    0x0005001100040801, 0x000a011e00040890, 0x1000040101000200, 0xc0540102001040ac,
    0x0300401028800080, 0x2040002000804080, 0x2100801000802000, 0x2002020812002040,
    0x0412820400800800, 0x0000040080800200, 0x0040099004000a18, 0x8439000061000082,
    0x4080008040008020, 0x4040904001010022, 0x1040200500110040, 0x0002000840220010,
    0x0004008040080800, 0x081a000410020008, 0x3200021008040001, 0x408000840046000b,
    0x00c0204010800080, 0x0450124000600a40, 0x609a100088200380, 0x0010040800809280,
    0x1001000410080100, 0x0404000200048080, 0x0600300201480400, 0x0011404081040200,
    0x0101004088122202, 0x2018802900400011, 0x2000120900c0a003, 0x1203601804100101,
    0x0003001002080005, 0x8041000400080201, 0x0200021000c80124, 0x0000044121840702,
];

#[derive(Copy, Clone)]
pub struct Magic {
    pub mask: BitBoard,
    pub magic: u64,
    pub shift: u8,
    pub offset: usize,
}

pub struct MagicBitboards {
    pub rook_magics: [Magic; 64],
    pub bishop_magics: [Magic; 64],
    pub rook_attacks: Vec<BitBoard>,
    pub bishop_attacks: Vec<BitBoard>,
}

impl MagicBitboards {
    pub fn new() -> Self {
        let mut rook_magics = [Magic {
            mask: BitBoard::EMPTY,
            magic: 0,
            shift: 0,
            offset: 0,
        }; 64];

        let mut bishop_magics = [Magic {
            mask: BitBoard::EMPTY,
            magic: 0,
            shift: 0,
            offset: 0,
        }; 64];

        let mut rook_attacks = Vec::new();
        let mut bishop_attacks = Vec::new();

        let mut rook_offset = 0;
        let mut bishop_offset = 0;

        // Initialize rook magics with hardcoded values
        for square_idx in 0..64 {
            let square = Square::from_index(square_idx as i8);
            let mask = Self::rook_mask(square);
            let bits = ROOK_RELEVANT_BITS[square_idx];
            let shift = 64 - bits;

            let size = 1 << bits;
            // Use hardcoded magic
            let magic = ROOK_MAGICS[square_idx];

            rook_magics[square_idx] = Magic {
                mask,
                magic,
                shift,
                offset: rook_offset,
            };

            rook_offset += size;
            rook_attacks.resize(rook_offset, BitBoard::EMPTY);

            Self::initialize_attacks(
                &mut rook_attacks,
                square,
                mask,
                magic,
                shift,
                rook_magics[square_idx].offset,
                Self::rook_attacks,
            );
        }

        // Initialize bishop magics with hardcoded values
        for square_idx in 0..64 {
            let square = Square::from_index(square_idx as i8);
            let mask = Self::bishop_mask(square);
            let bits = BISHOP_RELEVANT_BITS[square_idx];
            let shift = 64 - bits;

            let size = 1 << bits;
            // Use hardcoded magic
            let magic = BISHOP_MAGICS[square_idx];

            bishop_magics[square_idx] = Magic {
                mask,
                magic,
                shift,
                offset: bishop_offset,
            };

            bishop_offset += size;
            bishop_attacks.resize(bishop_offset, BitBoard::EMPTY);

            Self::initialize_attacks(
                &mut bishop_attacks,
                square,
                mask,
                magic,
                shift,
                bishop_magics[square_idx].offset,
                Self::bishop_attacks,
            );
        }

        Self {
            rook_magics,
            bishop_magics,
            rook_attacks,
            bishop_attacks,
        }
    }

    pub fn get_rook_attacks(&self, square: Square, occupancy: BitBoard) -> BitBoard {
        let square_idx = square.to_index() as usize;
        let magic = &self.rook_magics[square_idx];
        let index = self.magic_index(occupancy, magic);
        self.rook_attacks[magic.offset + index]
    }

    pub fn get_bishop_attacks(&self, square: Square, occupancy: BitBoard) -> BitBoard {
        let square_idx = square.to_index() as usize;
        let magic = &self.bishop_magics[square_idx];
        let index = self.magic_index(occupancy, magic);
        self.bishop_attacks[magic.offset + index]
    }

    pub fn get_queen_attacks(&self, square: Square, occupancy: BitBoard) -> BitBoard {
        self.get_rook_attacks(square, occupancy) | self.get_bishop_attacks(square, occupancy)
    }

    fn magic_index(&self, occupancy: BitBoard, magic: &Magic) -> usize {
        let relevant = occupancy & magic.mask;
        ((relevant.value().wrapping_mul(magic.magic)) >> magic.shift) as usize
    }

    fn initialize_attacks<F>(
        attacks: &mut Vec<BitBoard>,
        square: Square,
        mask: BitBoard,
        magic: u64,
        shift: u8,
        offset: usize,
        attack_fn: F,
    ) where
        F: Fn(Square, BitBoard) -> BitBoard,
    {
        let mut occupancy = BitBoard::EMPTY;
        let bits = mask.len();
        let size = 1 << bits;

        for i in 0..size {
            let index = (occupancy.value().wrapping_mul(magic) >> shift) as usize;
            attacks[offset + index] = attack_fn(square, occupancy);

            if i < size - 1 {
                // Carry-Rippler trick to enumerate all subsets of mask
                occupancy.0 = (occupancy.0.wrapping_sub(mask.0)) & mask.0;
            }
        }
    }

    fn rook_mask(square: Square) -> BitBoard {
        let mut mask = BitBoard::EMPTY;
        let rank = square.rank() as i8;
        let file = square.file() as i8;

        // North
        for r in rank + 1..7 {
            mask |= BitBoard::from_square(Square::new(File::from_index(file), Rank::new(r)));
        }
        // South
        for r in 1..rank {
            mask |= BitBoard::from_square(Square::new(File::from_index(file), Rank::new(r)));
        }
        // East
        for f in file + 1..7 {
            mask |= BitBoard::from_square(Square::new(File::from_index(f), Rank::new(rank)));
        }
        // West
        for f in 1..file {
            mask |= BitBoard::from_square(Square::new(File::from_index(f), Rank::new(rank)));
        }

        mask
    }

    fn bishop_mask(square: Square) -> BitBoard {
        let mut mask = BitBoard::EMPTY;
        let rank = square.rank() as i8;
        let file = square.file() as i8;

        // North-East
        for (r, f) in (rank + 1..7).zip(file + 1..7) {
            mask |= BitBoard::from_square(Square::new(File::from_index(f), Rank::new(r)));
        }
        // North-West
        for (r, f) in (rank + 1..7).zip((1..file).rev()) {
            mask |= BitBoard::from_square(Square::new(File::from_index(f), Rank::new(r)));
        }
        // South-East
        for (r, f) in (1..rank).rev().zip(file + 1..7) {
            mask |= BitBoard::from_square(Square::new(File::from_index(f), Rank::new(r)));
        }
        // South-West
        for (r, f) in (1..rank).rev().zip((1..file).rev()) {
            mask |= BitBoard::from_square(Square::new(File::from_index(f), Rank::new(r)));
        }

        mask
    }

    fn rook_attacks(square: Square, blockers: BitBoard) -> BitBoard {
        let mut attacks = BitBoard::EMPTY;
        let rank = square.rank() as i8;
        let file = square.file() as i8;

        // North
        for r in rank + 1..=7 {
            let target = Square::new(File::from_index(file), Rank::new(r));
            attacks |= BitBoard::from_square(target);
            if blockers.has(target) {
                break;
            }
        }
        // South
        for r in (0..rank).rev() {
            let target = Square::new(File::from_index(file), Rank::new(r));
            attacks |= BitBoard::from_square(target);
            if blockers.has(target) {
                break;
            }
        }
        // East
        for f in file + 1..=7 {
            let target = Square::new(File::from_index(f), Rank::new(rank));
            attacks |= BitBoard::from_square(target);
            if blockers.has(target) {
                break;
            }
        }
        // West
        for f in (0..file).rev() {
            let target = Square::new(File::from_index(f), Rank::new(rank));
            attacks |= BitBoard::from_square(target);
            if blockers.has(target) {
                break;
            }
        }

        attacks
    }

    fn bishop_attacks(square: Square, blockers: BitBoard) -> BitBoard {
        let mut attacks = BitBoard::EMPTY;
        let rank = square.rank() as i8;
        let file = square.file() as i8;

        // North-East
        for (r, f) in (rank + 1..=7).zip(file + 1..=7) {
            let target = Square::new(File::from_index(f), Rank::new(r));
            attacks |= BitBoard::from_square(target);
            if blockers.has(target) {
                break;
            }
        }
        // North-West
        for (r, f) in (rank + 1..=7).zip((0..file).rev()) {
            let target = Square::new(File::from_index(f), Rank::new(r));
            attacks |= BitBoard::from_square(target);
            if blockers.has(target) {
                break;
            }
        }
        // South-East
        for (r, f) in ((0..rank).rev()).zip(file + 1..=7) {
            let target = Square::new(File::from_index(f), Rank::new(r));
            attacks |= BitBoard::from_square(target);
            if blockers.has(target) {
                break;
            }
        }
        // South-West
        for (r, f) in ((0..rank).rev()).zip((0..file).rev()) {
            let target = Square::new(File::from_index(f), Rank::new(r));
            attacks |= BitBoard::from_square(target);
            if blockers.has(target) {
                break;
            }
        }

        attacks
    }
}
