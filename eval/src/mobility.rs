//! Mobility Evaluation
//!
//! Evaluates piece mobility - the number of squares a piece can move to.
//! Higher mobility generally correlates with better position and control.

use aether_types::{BoardQuery, Color, Piece, Square};
use crate::Score;

/// Bonus per legal move for each piece type
const KNIGHT_MOBILITY_BONUS: Score = 4;
const BISHOP_MOBILITY_BONUS: Score = 3;
const ROOK_MOBILITY_BONUS: Score = 2;
const QUEEN_MOBILITY_BONUS: Score = 1;

/// Evaluate mobility for a given color
///
/// Returns a score where positive = better mobility
pub fn evaluate_mobility<T: BoardQuery>(board: &T, color: Color) -> Score {
    let mut mobility_score = 0;

    // Iterate through all pieces of the given color
    for square_idx in 0..64 {
        let square = Square::from_index(square_idx);

        if let Some((piece, piece_color)) = board.piece_at(square) {
            if piece_color != color {
                continue;
            }

            // Count legal moves for this piece
            let move_count = count_piece_moves(board, square, piece, color);

            // Apply mobility bonus based on piece type
            let bonus = match piece {
                Piece::Knight => move_count * KNIGHT_MOBILITY_BONUS,
                Piece::Bishop => move_count * BISHOP_MOBILITY_BONUS,
                Piece::Rook => move_count * ROOK_MOBILITY_BONUS,
                Piece::Queen => move_count * QUEEN_MOBILITY_BONUS,
                Piece::Pawn | Piece::King => 0, // Don't count pawn/king mobility separately
            };

            mobility_score += bonus;
        }
    }

    mobility_score
}

/// Count pseudo-legal moves for a piece at a given square
fn count_piece_moves<T: BoardQuery>(board: &T, square: Square, piece: Piece, color: Color) -> Score {
    let mut count = 0;

    match piece {
        Piece::Knight => {
            // Knight moves: 8 possible L-shaped moves
            let offsets = [
                (-2, -1), (-2, 1), (-1, -2), (-1, 2),
                (1, -2), (1, 2), (2, -1), (2, 1),
            ];

            for (df, dr) in offsets {
                if let Some(target) = offset_square(square, df, dr)
                    && can_move_to(board, target, color) {
                        count += 1;
                    }
            }
        }
        Piece::Bishop => {
            // Bishop moves: diagonals
            count += count_sliding_moves(board, square, color, &[(1, 1), (1, -1), (-1, 1), (-1, -1)]);
        }
        Piece::Rook => {
            // Rook moves: files and ranks
            count += count_sliding_moves(board, square, color, &[(1, 0), (-1, 0), (0, 1), (0, -1)]);
        }
        Piece::Queen => {
            // Queen moves: combination of rook and bishop
            count += count_sliding_moves(board, square, color, &[
                (1, 0), (-1, 0), (0, 1), (0, -1),
                (1, 1), (1, -1), (-1, 1), (-1, -1),
            ]);
        }
        _ => {}
    }

    count
}

/// Count moves in sliding directions (for rook, bishop, queen)
fn count_sliding_moves<T: BoardQuery>(
    board: &T,
    square: Square,
    color: Color,
    directions: &[(i8, i8)],
) -> Score {
    let mut count = 0;

    for &(df, dr) in directions {
        let mut current_square = square;

        // Slide in this direction until blocked
        while let Some(next_square) = offset_square(current_square, df, dr) {
            if can_move_to(board, next_square, color) {
                count += 1;

                // If there's an enemy piece, we can capture but can't go further
                if board.piece_at(next_square).is_some() {
                    break;
                }

                current_square = next_square;
            } else {
                break; // Blocked by own piece
            }
        }
    }

    count
}

/// Check if we can move to a square (empty or enemy piece)
fn can_move_to<T: BoardQuery>(board: &T, square: Square, our_color: Color) -> bool {
    if let Some((_, piece_color)) = board.piece_at(square) {
        // Can capture enemy piece
        piece_color != our_color
    } else {
        // Empty square
        true
    }
}

/// Offset a square by file and rank deltas
fn offset_square(square: Square, df: i8, dr: i8) -> Option<Square> {
    let file = square.file() as i8;
    let rank = square.rank() as i8;

    let new_file = file + df;
    let new_rank = rank + dr;

    if (0..8).contains(&new_file) && (0..8).contains(&new_rank) {
        Some(Square::from_index(new_rank * 8 + new_file))
    } else {
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_offset_square() {
        let e4 = Square::from_index(28); // e4

        // Test valid offsets
        assert!(offset_square(e4, 1, 0).is_some()); // f4
        assert!(offset_square(e4, 0, 1).is_some()); // e5

        // Test boundary cases
        let a1 = Square::from_index(0);
        assert!(offset_square(a1, -1, 0).is_none()); // Off left edge
        assert!(offset_square(a1, 0, -1).is_none()); // Off bottom edge

        let h8 = Square::from_index(63);
        assert!(offset_square(h8, 1, 0).is_none()); // Off right edge
        assert!(offset_square(h8, 0, 1).is_none()); // Off top edge
    }
}
