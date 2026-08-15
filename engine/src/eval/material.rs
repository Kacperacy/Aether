//! Material weights.
//!
//! These are evaluation tuning parameters, not chess rules, so they live here
//! rather than on `Piece` in the core vocabulary crate.

use super::Score;
use aether_core::Piece;

pub const PAWN_VALUE: Score = 100;
pub const KNIGHT_VALUE: Score = 320;
pub const BISHOP_VALUE: Score = 330;
pub const ROOK_VALUE: Score = 500;
pub const QUEEN_VALUE: Score = 900;
pub const KING_VALUE: Score = 20000;

pub const VALUES: [Score; Piece::NUM] = [
    PAWN_VALUE,
    KNIGHT_VALUE,
    BISHOP_VALUE,
    ROOK_VALUE,
    QUEEN_VALUE,
    KING_VALUE,
];

/// Static material value of `piece`.
#[inline(always)]
pub const fn value(piece: Piece) -> Score {
    VALUES[piece as usize]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_piece_values() {
        assert_eq!(value(Piece::Pawn), 100);
        assert_eq!(value(Piece::Queen), 900);
    }
}
