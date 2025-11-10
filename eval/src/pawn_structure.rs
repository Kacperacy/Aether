//! Pawn Structure Evaluation
//!
//! Evaluates pawn structure features:
//! - Doubled pawns (penalty)
//! - Isolated pawns (penalty)
//! - Passed pawns (bonus)
//! - Pawn chains (bonus)

use aether_types::{BoardQuery, Color, File, Piece, Rank, Square};
use crate::Score;

/// Penalty for doubled pawns (two pawns on same file)
const DOUBLED_PAWN_PENALTY: Score = 20;

/// Penalty for isolated pawns (no friendly pawns on adjacent files)
const ISOLATED_PAWN_PENALTY: Score = 15;

/// Bonus for passed pawns (no enemy pawns blocking or attacking)
const PASSED_PAWN_BONUS: Score = 30;

/// Bonus per rank advanced for passed pawns (increases closer to promotion)
const PASSED_PAWN_RANK_BONUS: Score = 10;

/// Bonus for pawn chains (pawns protecting each other diagonally)
const PAWN_CHAIN_BONUS: Score = 5;

/// Evaluate pawn structure for a given color
///
/// Returns a score where positive = better pawn structure
pub fn evaluate_pawn_structure<T: BoardQuery>(board: &T, color: Color) -> Score {
    let mut score = 0;

    // Collect all pawns for this color
    let mut pawns = Vec::new();
    for square_idx in 0..64 {
        let square = Square::from_index(square_idx);
        if let Some((piece, piece_color)) = board.piece_at(square)
            && piece == Piece::Pawn && piece_color == color {
                pawns.push(square);
            }
    }

    // Evaluate each pawn
    for &pawn_square in &pawns {
        let file = pawn_square.file();
        let rank = pawn_square.rank();

        // Check for doubled pawns (another pawn on same file)
        if is_doubled_pawn(board, pawn_square, file, color) {
            score -= DOUBLED_PAWN_PENALTY;
        }

        // Check for isolated pawns (no friendly pawns on adjacent files)
        if is_isolated_pawn(board, file, color) {
            score -= ISOLATED_PAWN_PENALTY;
        }

        // Check for passed pawns (no enemy pawns blocking or attacking)
        if is_passed_pawn(board, pawn_square, file, rank, color) {
            score += PASSED_PAWN_BONUS;

            // Add bonus based on how far advanced the pawn is
            let advancement = match color {
                Color::White => rank as i32,
                Color::Black => 7 - rank as i32,
            };
            score += advancement * PASSED_PAWN_RANK_BONUS;
        }

        // Check for pawn chains (protected by another pawn)
        if is_protected_by_pawn(board, pawn_square, color) {
            score += PAWN_CHAIN_BONUS;
        }
    }

    score
}

/// Check if a pawn is doubled (another friendly pawn on same file)
fn is_doubled_pawn<T: BoardQuery>(board: &T, pawn_square: Square, file: File, color: Color) -> bool {
    for rank_idx in 0..8 {
        let rank = Rank::new(rank_idx);
        let square = Square::new(file, rank);

        // Skip the pawn itself
        if square == pawn_square {
            continue;
        }

        if let Some((piece, piece_color)) = board.piece_at(square)
            && piece == Piece::Pawn && piece_color == color {
                return true; // Found another friendly pawn on same file
            }
    }

    false
}

/// Check if a pawn is isolated (no friendly pawns on adjacent files)
fn is_isolated_pawn<T: BoardQuery>(board: &T, file: File, color: Color) -> bool {
    let left_file = offset_file(file, -1);
    let right_file = offset_file(file, 1);

    // Check left file
    if let Some(left) = left_file
        && has_pawn_on_file(board, left, color) {
            return false;
        }

    // Check right file
    if let Some(right) = right_file
        && has_pawn_on_file(board, right, color) {
            return false;
        }

    true // No friendly pawns on adjacent files
}

/// Check if a pawn is passed (no enemy pawns blocking or attacking)
fn is_passed_pawn<T: BoardQuery>(
    board: &T,
    _pawn_square: Square,
    file: File,
    rank: Rank,
    color: Color,
) -> bool {
    let opponent = color.opponent();

    // Check if any enemy pawns can stop this pawn
    let (start_rank, end_rank) = match color {
        Color::White => (rank as i8 + 1, 8), // Check ranks ahead
        Color::Black => (0, rank as i8),      // Check ranks ahead
    };

    // Check same file and adjacent files
    for file_offset in -1..=1 {
        if let Some(check_file) = offset_file(file, file_offset) {
            for rank_idx in start_rank..end_rank {
                if !(0..8).contains(&rank_idx) {
                    continue;
                }

                let check_rank = Rank::new(rank_idx);
                let check_square = Square::new(check_file, check_rank);

                if let Some((piece, piece_color)) = board.piece_at(check_square)
                    && piece == Piece::Pawn && piece_color == opponent {
                        return false; // Enemy pawn blocks or can capture
                    }
            }
        }
    }

    true // No enemy pawns can stop this pawn
}

/// Check if a pawn is protected by another friendly pawn
fn is_protected_by_pawn<T: BoardQuery>(board: &T, pawn_square: Square, color: Color) -> bool {
    let file = pawn_square.file();
    let rank = pawn_square.rank();

    // Check diagonally behind (where protecting pawns would be)
    let back_rank = match color {
        Color::White => rank.offset(-1),
        Color::Black => rank.offset(1),
    };

    if let Some(back_rank) = back_rank {
        // Check left diagonal
        if let Some(left_file) = offset_file(file, -1) {
            let check_square = Square::new(left_file, back_rank);
            if let Some((piece, piece_color)) = board.piece_at(check_square)
                && piece == Piece::Pawn && piece_color == color {
                    return true;
                }
        }

        // Check right diagonal
        if let Some(right_file) = offset_file(file, 1) {
            let check_square = Square::new(right_file, back_rank);
            if let Some((piece, piece_color)) = board.piece_at(check_square)
                && piece == Piece::Pawn && piece_color == color {
                    return true;
                }
        }
    }

    false
}

/// Check if there's a pawn of the given color on a file
fn has_pawn_on_file<T: BoardQuery>(board: &T, file: File, color: Color) -> bool {
    for rank_idx in 0..8 {
        let rank = Rank::new(rank_idx);
        let square = Square::new(file, rank);

        if let Some((piece, piece_color)) = board.piece_at(square)
            && piece == Piece::Pawn && piece_color == color {
                return true;
            }
    }

    false
}

/// Offset a file by delta (-1, 0, 1)
fn offset_file(file: File, delta: i8) -> Option<File> {
    let file_idx = file as i8;
    let new_idx = file_idx + delta;

    if (0..8).contains(&new_idx) {
        Some(File::from_index(new_idx))
    } else {
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_offset_file() {
        assert_eq!(offset_file(File::E, 0), Some(File::E));
        assert_eq!(offset_file(File::E, 1), Some(File::F));
        assert_eq!(offset_file(File::E, -1), Some(File::D));
        assert_eq!(offset_file(File::A, -1), None);
        assert_eq!(offset_file(File::H, 1), None);
    }
}
