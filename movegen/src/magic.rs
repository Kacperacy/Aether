use crate::magic_constants::{BISHOP_MAGICS, BISHOP_MOVES, ROOK_MAGICS, ROOK_MOVES};
use aether_types::{BitBoard, Square};

pub struct MagicEntry {
    pub mask: u64,
    pub magic: u64,
    pub index_bits: u8,
}

#[inline(always)]
fn magic_index(mask: u64, magic: u64, index_bits: u8, blockers: u64) -> usize {
    let blockers = blockers & mask;
    let hash = blockers.wrapping_mul(magic);
    (hash >> (64 - index_bits)) as usize
}

#[inline(always)]
pub fn get_rook_attacks(square: Square, blockers: BitBoard) -> BitBoard {
    let magic = &ROOK_MAGICS[square as usize];
    let moves = &ROOK_MOVES[square as usize];
    let index = magic_index(magic.mask, magic.magic, magic.index_bits, blockers.value());
    BitBoard(moves[index])
}

#[inline(always)]
pub fn get_bishop_attacks(square: Square, blockers: BitBoard) -> BitBoard {
    let magic = &BISHOP_MAGICS[square as usize];
    let moves = &BISHOP_MOVES[square as usize];
    let index = magic_index(magic.mask, magic.magic, magic.index_bits, blockers.value());
    BitBoard(moves[index])
}

#[inline(always)]
pub fn get_queen_attacks(square: Square, blockers: BitBoard) -> BitBoard {
    get_rook_attacks(square, blockers) | get_bishop_attacks(square, blockers)
}

// Optimized batch operations for multiple squares
pub fn get_rook_attacks_batch(squares: &[Square], blockers: BitBoard) -> Vec<BitBoard> {
    squares
        .iter()
        .map(|&sq| get_rook_attacks(sq, blockers))
        .collect()
}

pub fn get_bishop_attacks_batch(squares: &[Square], blockers: BitBoard) -> Vec<BitBoard> {
    squares
        .iter()
        .map(|&sq| get_bishop_attacks(sq, blockers))
        .collect()
}

// Cache-friendly attack generation
pub struct AttackCache {
    rook_cache: [[BitBoard; 4096]; 64],
    bishop_cache: [[BitBoard; 512]; 64],
}

impl Default for AttackCache {
    fn default() -> Self {
        Self::new()
    }
}

impl AttackCache {
    pub fn new() -> Self {
        Self {
            rook_cache: [[BitBoard::EMPTY; 4096]; 64],
            bishop_cache: [[BitBoard::EMPTY; 512]; 64],
        }
    }

    pub fn get_rook_attacks(&self, square: Square, blockers: BitBoard) -> BitBoard {
        let magic = &ROOK_MAGICS[square as usize];
        let index = magic_index(magic.mask, magic.magic, magic.index_bits, blockers.value());
        self.rook_cache[square as usize][index]
    }

    pub fn get_bishop_attacks(&self, square: Square, blockers: BitBoard) -> BitBoard {
        let magic = &BISHOP_MAGICS[square as usize];
        let index = magic_index(magic.mask, magic.magic, magic.index_bits, blockers.value());
        self.bishop_cache[square as usize][index]
    }
}
