use crate::{BitBoard, Color, Square};

/// Pre-computed pawn attacks for all squares and colors
static PAWN_ATTACKS: [[BitBoard; 64]; 2] = [
    init_pawn_attacks(Color::White),
    init_pawn_attacks(Color::Black),
];

/// Pre-computed knight attacks for all squares
static KNIGHT_ATTACKS: [BitBoard; 64] = init_knight_attacks();

/// Pre-computed king attacks for all squares
static KING_ATTACKS: [BitBoard; 64] = init_king_attacks();

/// Get squares attacked by a pawn of `color` on `square`
#[inline(always)]
pub fn pawn_attacks(square: Square, color: Color) -> BitBoard {
    PAWN_ATTACKS[color as usize][square.to_index() as usize]
}

/// Get squares from which a pawn of `color` can attack `square`
#[inline(always)]
pub fn pawn_attacks_from(square: Square, color: Color) -> BitBoard {
    let rank_offset = -color.forward_direction();
    let mut attacks = BitBoard::EMPTY;
    if let Some(sq) = square.offset(-1, rank_offset) {
        attacks |= sq.bitboard();
    }
    if let Some(sq) = square.offset(1, rank_offset) {
        attacks |= sq.bitboard();
    }
    attacks
}

/// Get pawn move targets (non-capturing) for a pawn of `color` on `square`, given `occupied` squares
#[inline(always)]
pub fn pawn_moves(square: Square, color: Color, occupied: BitBoard) -> BitBoard {
    let mut moves = BitBoard::EMPTY;

    let push_dir = color.forward_direction();

    if let Some(single) = square.offset(0, push_dir) {
        if !occupied.has(single) {
            moves |= single.bitboard();

            // Double push from starting rank
            let starting_rank = color.pawn_start_rank();

            if square.rank() == starting_rank {
                if let Some(double) = square.offset(0, 2 * push_dir) {
                    if !occupied.has(double) {
                        moves |= double.bitboard();
                    }
                }
            }
        }
    }

    moves
}

/// Get squares attacked by a knight on `square`
#[inline(always)]
pub fn knight_attacks(square: Square) -> BitBoard {
    KNIGHT_ATTACKS[square.to_index() as usize]
}

/// Get squares attacked by a king on `square`
#[inline(always)]
pub fn king_attacks(square: Square) -> BitBoard {
    KING_ATTACKS[square.to_index() as usize]
}

/// Check if a square is a promotion rank for the given color
#[inline(always)]
pub const fn is_promotion_rank(square: Square, color: Color) -> bool {
    square.rank() as u8 == color.pawn_promotion_rank() as u8
}

/// Initialize pawn attacks table for a given color
const fn init_pawn_attacks(color: Color) -> [BitBoard; 64] {
    let mut attacks = [BitBoard::EMPTY; 64];
    let mut i: usize = 0;

    while i < 64 {
        let square = Square::from_index(i as i8);
        attacks[i] = compute_pawn_attacks(square, color);
        i += 1;
    }

    attacks
}

/// Compute pawn attacks for a given square and color
const fn compute_pawn_attacks(square: Square, color: Color) -> BitBoard {
    let rank_offset = color.forward_direction();
    let mut result = BitBoard::EMPTY;

    if let Some(target) = square.offset(-1, rank_offset) {
        result = BitBoard(result.0 | target.bitboard().0);
    }
    if let Some(target) = square.offset(1, rank_offset) {
        result = BitBoard(result.0 | target.bitboard().0);
    }

    result
}

/// Initialize king attacks table
const fn init_knight_attacks() -> [BitBoard; 64] {
    const KNIGHT_OFFSETS: [(i8, i8); 8] = [
        (1, 2),
        (2, 1),
        (2, -1),
        (1, -2),
        (-1, -2),
        (-2, -1),
        (-2, 1),
        (-1, 2),
    ];

    let mut attacks = [BitBoard::EMPTY; 64];
    let mut i: usize = 0;

    while i < 64 {
        let square = Square::from_index(i as i8);
        attacks[i] = compute_offsets(square, &KNIGHT_OFFSETS);
        i += 1;
    }

    attacks
}

/// Initialize king attacks table
const fn init_king_attacks() -> [BitBoard; 64] {
    const KING_OFFSETS: [(i8, i8); 8] = [
        (1, 1),
        (1, 0),
        (1, -1),
        (0, -1),
        (-1, -1),
        (-1, 0),
        (-1, 1),
        (0, 1),
    ];

    let mut attacks = [BitBoard::EMPTY; 64];
    let mut i: usize = 0;

    while i < 64 {
        let square = Square::from_index(i as i8);
        attacks[i] = compute_offsets(square, &KING_OFFSETS);
        i += 1;
    }

    attacks
}

/// Helper: compute attacks based on given offsets
const fn compute_offsets<const N: usize>(square: Square, offsets: &[(i8, i8); N]) -> BitBoard {
    let mut result = BitBoard::EMPTY;
    let mut i: usize = 0;

    while i < N {
        let (file_off, rank_off) = offsets[i];
        if let Some(target) = square.offset(file_off, rank_off) {
            result = BitBoard(result.0 | target.bitboard().0);
        }
        i += 1;
    }

    result
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_pawn_attacks_white() {
        let attacks = pawn_attacks(Square::E4, Color::White);
        // White pawn on e4 attacks d5 and f5
        assert!(attacks.has(Square::D5));
        assert!(attacks.has(Square::F5));
        assert_eq!(attacks.count(), 2);
    }

    #[test]
    fn test_pawn_attacks_black() {
        let attacks = pawn_attacks(Square::E4, Color::Black);
        // Black pawn on e4 attacks d3 and f3
        assert!(attacks.has(Square::D3));
        assert!(attacks.has(Square::F3));
        assert_eq!(attacks.count(), 2);
    }

    #[test]
    fn test_pawn_attacks_edge() {
        let attacks = pawn_attacks(Square::A4, Color::White);
        // Pawn on a-file can only attack to the right
        assert!(attacks.has(Square::B5));
        assert_eq!(attacks.count(), 1);
    }

    #[test]
    fn test_pawn_attacks_from_inverse() {
        let square = Square::E4;
        let attackers = pawn_attacks_from(square, Color::White);

        // White pawns that attack e4 are on d3 and f3
        assert!(attackers.has(Square::D3));
        assert!(attackers.has(Square::F3));
        assert_eq!(attackers.count(), 2);

        // Verify inverse relationship
        assert!(pawn_attacks(Square::D3, Color::White).has(square));
        assert!(pawn_attacks(Square::F3, Color::White).has(square));
    }

    #[test]
    fn test_pawn_moves_single_push() {
        let moves = pawn_moves(Square::E2, Color::White, BitBoard::EMPTY);
        // From e2, white pawn can move to e3 and e4
        assert!(moves.has(Square::E3));
        assert!(moves.has(Square::E4));
        assert_eq!(moves.count(), 2);
    }

    #[test]
    fn test_pawn_moves_blocked() {
        let mut occupied = BitBoard::EMPTY;
        occupied |= Square::E3.bitboard();

        let moves = pawn_moves(Square::E2, Color::White, occupied);
        // Blocked by piece on e3
        assert!(moves.is_empty());
    }

    #[test]
    fn test_pawn_moves_double_blocked() {
        let mut occupied = BitBoard::EMPTY;
        occupied |= Square::E4.bitboard();

        let moves = pawn_moves(Square::E2, Color::White, occupied);
        // Can push once but not twice
        assert!(moves.has(Square::E3));
        assert!(!moves.has(Square::E4));
        assert_eq!(moves.count(), 1);
    }

    #[test]
    fn test_knight_attacks_center() {
        let attacks = knight_attacks(Square::E4);
        // Knight has 8 moves from center
        assert_eq!(attacks.count(), 8);

        // Check specific squares
        assert!(attacks.has(Square::D6));
        assert!(attacks.has(Square::F6));
        assert!(attacks.has(Square::G5));
        assert!(attacks.has(Square::G3));
        assert!(attacks.has(Square::F2));
        assert!(attacks.has(Square::D2));
        assert!(attacks.has(Square::C3));
        assert!(attacks.has(Square::C5));
    }

    #[test]
    fn test_knight_attacks_corner() {
        let attacks = knight_attacks(Square::A1);
        // Knight in corner has only 2 moves
        assert_eq!(attacks.count(), 2);
        assert!(attacks.has(Square::B3));
        assert!(attacks.has(Square::C2));
    }

    #[test]
    fn test_king_attacks_center() {
        let attacks = king_attacks(Square::E4);
        // King has 8 moves from center
        assert_eq!(attacks.count(), 8);
    }

    #[test]
    fn test_king_attacks_corner() {
        let attacks = king_attacks(Square::A1);
        // King in corner has 3 moves
        assert_eq!(attacks.count(), 3);
        assert!(attacks.has(Square::A2));
        assert!(attacks.has(Square::B1));
        assert!(attacks.has(Square::B2));
    }

    #[test]
    fn test_is_promotion_rank() {
        assert!(is_promotion_rank(Square::E8, Color::White));
        assert!(is_promotion_rank(Square::A8, Color::White));
        assert!(!is_promotion_rank(Square::E7, Color::White));

        assert!(is_promotion_rank(Square::E1, Color::Black));
        assert!(is_promotion_rank(Square::H1, Color::Black));
        assert!(!is_promotion_rank(Square::E2, Color::Black));
    }
}
