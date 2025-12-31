use crate::{BISHOP_MAGICS, BISHOP_MOVES, BitBoard, ROOK_MAGICS, ROOK_MOVES, Square};

#[derive(Debug, Clone, Copy)]
pub struct MagicEntry {
    /// Mask of relevant occupancy bits for the piece on the square
    pub mask: u64,

    /// Magic multiplier for hashing blocker configurations
    pub magic: u64,

    /// Number of bits used for indexing into the attack table
    pub index_bits: u8,
}

impl MagicEntry {
    /// Computes the index into the attack table based on the occupied squares.
    /// Uses PEXT instruction when BMI2 is available (Intel Haswell+, AMD Excavator+),
    /// falls back to magic multiplication otherwise.
    #[cfg(all(target_arch = "x86_64", target_feature = "bmi2"))]
    #[inline(always)]
    fn index(&self, occupied: u64) -> u64 {
        // PEXT directly extracts the relevant bits into a compact index
        // This is ~2x faster than magic multiplication (~3 cycles vs ~6 cycles)
        unsafe { std::arch::x86_64::_pext_u64(occupied, self.mask) }
    }

    /// Fallback implementation using magic bitboard multiplication
    #[cfg(not(all(target_arch = "x86_64", target_feature = "bmi2")))]
    #[inline(always)]
    const fn index(&self, occupied: u64) -> u64 {
        let relevant = occupied & self.mask;
        let hash = relevant.wrapping_mul(self.magic);
        hash >> (64 - self.index_bits)
    }
}

/// Computes rook attacks for a given square and occupied bitboard using magic bitboards
#[inline(always)]
pub fn rook_attacks(square: Square, occupied: BitBoard) -> BitBoard {
    let sq_idx = square.to_index() as usize;

    let magic = &ROOK_MAGICS[sq_idx];
    let moves = &ROOK_MOVES[sq_idx];
    let index = magic.index(occupied.value()) as usize;

    BitBoard(moves[index])
}

/// Computes bishop attacks for a given square and occupied bitboard using magic bitboards
#[inline(always)]
pub fn bishop_attacks(square: Square, occupied: BitBoard) -> BitBoard {
    let sq_idx = square.to_index() as usize;

    let magic = &BISHOP_MAGICS[sq_idx];
    let moves = &BISHOP_MOVES[sq_idx];
    let index = magic.index(occupied.value()) as usize;

    BitBoard(moves[index])
}

/// Computes queen attacks for a given square and occupied bitboard using magic bitboards
#[inline(always)]
pub fn queen_attacks(square: Square, occupied: BitBoard) -> BitBoard {
    rook_attacks(square, occupied) | bishop_attacks(square, occupied)
}

/// Batch computes rook attacks for multiple squares given an occupied bitboard
pub fn rook_attacks_batch(
    squares: &[Square],
    occupied: BitBoard,
) -> impl Iterator<Item = BitBoard> + '_ {
    squares.iter().map(move |&sq| rook_attacks(sq, occupied))
}

/// Batch computes bishop attacks for multiple squares given an occupied bitboard
pub fn bishop_attacks_batch(
    squares: &[Square],
    occupied: BitBoard,
) -> impl Iterator<Item = BitBoard> + '_ {
    squares.iter().map(move |&sq| bishop_attacks(sq, occupied))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_magic_entry_index() {
        let entry = MagicEntry {
            mask: 0x007E7E7E7E7E7E00, // typical rook mask
            magic: 0x0080001020400080,
            index_bits: 12,
        };

        // Index should always be < 2^12 = 4096
        for occ in [0u64, u64::MAX, 0x0F0F0F0F0F0F0F0F] {
            let idx = entry.index(occ);
            assert!(idx < (1 << 12));
        }
    }

    #[test]
    fn test_rook_attacks_symmetry() {
        let empty = BitBoard::EMPTY;

        // Rook on a1 should attack same as h8 (180° rotation)
        let a1_attacks = rook_attacks(Square::A1, empty);
        let h8_attacks = rook_attacks(Square::H8, empty);

        assert_eq!(a1_attacks.count(), h8_attacks.count());
        assert_eq!(a1_attacks.count(), 14); // 7 horizontal + 7 vertical
    }

    #[test]
    fn test_bishop_attacks_center() {
        let empty = BitBoard::EMPTY;
        let attacks = bishop_attacks(Square::D4, empty);

        // From d4, bishop should attack 13 squares on diagonals
        assert_eq!(attacks.count(), 13);

        // Check some specific squares
        assert!(attacks.has(Square::A1));
        assert!(attacks.has(Square::G7));
        assert!(attacks.has(Square::A7));
        assert!(attacks.has(Square::G1));
    }

    #[test]
    fn test_queen_attacks_combines() {
        let empty = BitBoard::EMPTY;
        let queen = queen_attacks(Square::E4, empty);
        let rook = rook_attacks(Square::E4, empty);
        let bishop = bishop_attacks(Square::E4, empty);

        // Queen should be exactly rook | bishop
        assert_eq!(queen, rook | bishop);
        assert_eq!(queen.count(), 27); // 14 rook + 13 bishop
    }

    #[test]
    fn test_blocker_stops_attacks() {
        let mut occupied = BitBoard::EMPTY;
        occupied |= BitBoard::from_square(Square::E6);

        let attacks = rook_attacks(Square::E1, occupied);

        // Should attack up to and including e6
        assert!(attacks.has(Square::E2));
        assert!(attacks.has(Square::E6));
        // But not beyond
        assert!(!attacks.has(Square::E7));
        assert!(!attacks.has(Square::E8));
    }

    #[test]
    fn test_multiple_blockers() {
        let mut occupied = BitBoard::EMPTY;
        occupied |= BitBoard::from_square(Square::E6);
        occupied |= BitBoard::from_square(Square::C4);
        occupied |= BitBoard::from_square(Square::G4);

        let attacks = rook_attacks(Square::E4, occupied);

        // Should be blocked in all directions
        assert!(attacks.has(Square::E5)); // can move up once
        assert!(attacks.has(Square::E6)); // blocker included
        assert!(!attacks.has(Square::E7)); // blocked

        assert!(attacks.has(Square::D4)); // can move left once
        assert!(attacks.has(Square::C4)); // blocker included
        assert!(!attacks.has(Square::B4)); // blocked

        assert!(attacks.has(Square::F4)); // can move right once
        assert!(attacks.has(Square::G4)); // blocker included
        assert!(!attacks.has(Square::H4)); // blocked
    }
}
