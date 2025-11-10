//! Piece-square tables for positional evaluation.
//!
//! This module provides lookup tables that assign bonuses/penalties to pieces
//! based on their position on the board. Values are in centipawns (100 = 1 pawn).
//!
//! All tables are from White's perspective (rank 0 = 1st rank for White) and
//! are automatically flipped for Black pieces.

use aether_types::{Color, Piece, Square};

type PieceSquareTable = [i32; 64];

/// Pawn piece-square table
/// Encourages: pawn advancement, center control
const PAWN_TABLE: PieceSquareTable = [
     0,   0,   0,   0,   0,   0,   0,   0,  // Rank 1
     5,  10,  10, -20, -20,  10,  10,   5,  // Rank 2
     5,  -5, -10,   0,   0, -10,  -5,   5,  // Rank 3
     0,   0,   0,  20,  20,   0,   0,   0,  // Rank 4
     5,   5,  10,  25,  25,  10,   5,   5,  // Rank 5
    10,  10,  20,  30,  30,  20,  10,  10,  // Rank 6
    50,  50,  50,  50,  50,  50,  50,  50,  // Rank 7
     0,   0,   0,   0,   0,   0,   0,   0,  // Rank 8
];

/// Knight piece-square table
/// Encourages: centralization, avoid edges
const KNIGHT_TABLE: PieceSquareTable = [
    -50, -40, -30, -30, -30, -30, -40, -50,  // Rank 1
    -40, -20,   0,   0,   0,   0, -20, -40,  // Rank 2
    -30,   0,  10,  15,  15,  10,   0, -30,  // Rank 3
    -30,   5,  15,  20,  20,  15,   5, -30,  // Rank 4
    -30,   0,  15,  20,  20,  15,   0, -30,  // Rank 5
    -30,   5,  10,  15,  15,  10,   5, -30,  // Rank 6
    -40, -20,   0,   5,   5,   0, -20, -40,  // Rank 7
    -50, -40, -30, -30, -30, -30, -40, -50,  // Rank 8
];

/// Bishop piece-square table
/// Encourages: centralization, long diagonals
const BISHOP_TABLE: PieceSquareTable = [
    -20, -10, -10, -10, -10, -10, -10, -20,  // Rank 1
    -10,   0,   0,   0,   0,   0,   0, -10,  // Rank 2
    -10,   0,   5,  10,  10,   5,   0, -10,  // Rank 3
    -10,   5,   5,  10,  10,   5,   5, -10,  // Rank 4
    -10,   0,  10,  10,  10,  10,   0, -10,  // Rank 5
    -10,  10,  10,  10,  10,  10,  10, -10,  // Rank 6
    -10,   5,   0,   0,   0,   0,   5, -10,  // Rank 7
    -20, -10, -10, -10, -10, -10, -10, -20,  // Rank 8
];

/// Rook piece-square table
/// Encourages: 7th rank, open files
const ROOK_TABLE: PieceSquareTable = [
     0,   0,   0,   0,   0,   0,   0,   0,  // Rank 1
     5,  10,  10,  10,  10,  10,  10,   5,  // Rank 2
    -5,   0,   0,   0,   0,   0,   0,  -5,  // Rank 3
    -5,   0,   0,   0,   0,   0,   0,  -5,  // Rank 4
    -5,   0,   0,   0,   0,   0,   0,  -5,  // Rank 5
    -5,   0,   0,   0,   0,   0,   0,  -5,  // Rank 6
    -5,   0,   0,   0,   0,   0,   0,  -5,  // Rank 7
     0,   0,   0,   5,   5,   0,   0,   0,  // Rank 8
];

/// Queen piece-square table
/// Encourages: centralization, mobility
const QUEEN_TABLE: PieceSquareTable = [
    -20, -10, -10,  -5,  -5, -10, -10, -20,  // Rank 1
    -10,   0,   0,   0,   0,   0,   0, -10,  // Rank 2
    -10,   0,   5,   5,   5,   5,   0, -10,  // Rank 3
     -5,   0,   5,   5,   5,   5,   0,  -5,  // Rank 4
      0,   0,   5,   5,   5,   5,   0,  -5,  // Rank 5
    -10,   5,   5,   5,   5,   5,   0, -10,  // Rank 6
    -10,   0,   5,   0,   0,   0,   0, -10,  // Rank 7
    -20, -10, -10,  -5,  -5, -10, -10, -20,  // Rank 8
];

/// King piece-square table (middlegame)
/// Encourages: castled position, safety
const KING_TABLE: PieceSquareTable = [
    -30, -40, -40, -50, -50, -40, -40, -30,  // Rank 1
    -30, -40, -40, -50, -50, -40, -40, -30,  // Rank 2
    -30, -40, -40, -50, -50, -40, -40, -30,  // Rank 3
    -30, -40, -40, -50, -50, -40, -40, -30,  // Rank 4
    -20, -30, -30, -40, -40, -30, -30, -20,  // Rank 5
    -10, -20, -20, -20, -20, -20, -20, -10,  // Rank 6
     20,  20,   0,   0,   0,   0,  20,  20,  // Rank 7
     20,  30,  10,   0,   0,  10,  30,  20,  // Rank 8
];

pub struct PieceSquareTables;

impl PieceSquareTables {
    /// Get piece-square table value for a piece at a square
    /// Automatically flips for Black pieces
    pub fn get_value(&self, piece: Piece, square: Square, color: Color) -> i32 {
        let table = match piece {
            Piece::Pawn => &PAWN_TABLE,
            Piece::Knight => &KNIGHT_TABLE,
            Piece::Bishop => &BISHOP_TABLE,
            Piece::Rook => &ROOK_TABLE,
            Piece::Queen => &QUEEN_TABLE,
            Piece::King => &KING_TABLE,
        };

        // For White, use square as-is
        // For Black, flip vertically (rank 0 <-> rank 7)
        let index = match color {
            Color::White => square as usize,
            Color::Black => {
                let rank = square.rank();
                let file = square.file();
                let flipped_rank = rank.flip();
                Square::new(file, flipped_rank) as usize
            }
        };

        table[index]
    }
}

/// Global piece-square tables instance
pub const PIECE_SQUARE_TABLES: PieceSquareTables = PieceSquareTables;

#[cfg(test)]
mod tests {
    use super::*;
    use aether_types::{File, Rank};

    #[test]
    fn test_pawn_advancement_bonus() {
        let pst = PieceSquareTables;

        // White pawn on e2 should have base value
        let e2 = Square::new(File::E, Rank::new(1));
        let e2_value = pst.get_value(Piece::Pawn, e2, Color::White);

        // White pawn on e4 should have higher value
        let e4 = Square::new(File::E, Rank::new(3));
        let e4_value = pst.get_value(Piece::Pawn, e4, Color::White);

        assert!(e4_value > e2_value, "Advanced pawns should be valued higher");
    }

    #[test]
    fn test_knight_centralization() {
        let pst = PieceSquareTables;

        // Knight on edge (a1)
        let a1 = Square::new(File::A, Rank::new(0));
        let edge_value = pst.get_value(Piece::Knight, a1, Color::White);

        // Knight in center (e4)
        let e4 = Square::new(File::E, Rank::new(3));
        let center_value = pst.get_value(Piece::Knight, e4, Color::White);

        assert!(center_value > edge_value, "Central knights should be valued higher");
    }

    #[test]
    fn test_color_symmetry() {
        let pst = PieceSquareTables;

        // White pawn on e2
        let e2 = Square::new(File::E, Rank::new(1));
        let white_value = pst.get_value(Piece::Pawn, e2, Color::White);

        // Black pawn on e7 (mirror position)
        let e7 = Square::new(File::E, Rank::new(6));
        let black_value = pst.get_value(Piece::Pawn, e7, Color::Black);

        assert_eq!(white_value, black_value, "Mirror positions should have equal values");
    }
}
