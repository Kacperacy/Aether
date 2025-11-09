//! Evaluation crate
//!
//! Responsibilities:
//! - Provide position evaluation heuristics and feature extraction used by the search.
//! - Remain decoupled from policy/state management; operate on data exposed via `aether_types`.
//! - Avoid heavy dependencies to keep compile times reasonable and layering clean.
//!
//! Typical consumers: `search`, `engine`, benchmarking tools.

mod piece_square_tables;
mod king_safety;

use aether_types::{BoardQuery, Color, Square};
pub use piece_square_tables::PIECE_SQUARE_TABLES;
pub use king_safety::evaluate_king_safety;

/// Evaluation score in centipawns (100 = 1 pawn)
pub type Score = i32;

/// Maximum evaluation score (mate or near-mate)
pub const MATE_SCORE: Score = 100_000;

/// Minimum evaluation score
pub const NEG_MATE_SCORE: Score = -100_000;

/// Checkmate score at depth N
pub const fn mate_in(ply: i32) -> Score {
    MATE_SCORE - ply
}

/// Score for being checkmated at depth N
pub const fn mated_in(ply: i32) -> Score {
    -MATE_SCORE + ply
}

/// Trait for position evaluation
pub trait Evaluator {
    /// Evaluate the position from the perspective of the side to move.
    /// Positive scores favor the side to move.
    fn evaluate<T: BoardQuery>(&self, board: &T) -> Score;
}

/// Simple material + piece-square table evaluator
#[derive(Debug, Clone, Copy)]
pub struct SimpleEvaluator;

impl SimpleEvaluator {
    pub fn new() -> Self {
        Self
    }

    /// Evaluate material balance
    fn material_balance<T: BoardQuery>(&self, board: &T) -> Score {
        let mut score = 0;

        for square_idx in 0..64 {
            let square = Square::from_index(square_idx);
            if let Some((piece, color)) = board.piece_at(square) {
                let piece_value = piece.value() as i32;
                let value = match color {
                    Color::White => piece_value,
                    Color::Black => -piece_value,
                };
                score += value;
            }
        }

        score
    }

    /// Evaluate piece positioning using piece-square tables
    fn positional_value<T: BoardQuery>(&self, board: &T) -> Score {
        let mut score = 0;

        for square_idx in 0..64 {
            let square = Square::from_index(square_idx);
            if let Some((piece, color)) = board.piece_at(square) {
                let pst_value = PIECE_SQUARE_TABLES.get_value(piece, square, color);
                let value = match color {
                    Color::White => pst_value,
                    Color::Black => -pst_value,
                };
                score += value;
            }
        }

        score
    }
}

impl Default for SimpleEvaluator {
    fn default() -> Self {
        Self::new()
    }
}

impl Evaluator for SimpleEvaluator {
    fn evaluate<T: BoardQuery>(&self, board: &T) -> Score {
        // Calculate material and positional scores
        let material = self.material_balance(board);
        let positional = self.positional_value(board);

        // Calculate king safety for both sides
        let white_king_safety = king_safety::evaluate_king_safety(board, Color::White);
        let black_king_safety = king_safety::evaluate_king_safety(board, Color::Black);
        let king_safety_diff = white_king_safety - black_king_safety;

        let total = material + positional + king_safety_diff;

        // Return score from perspective of side to move
        match board.side_to_move() {
            Color::White => total,
            Color::Black => -total,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_mate_scores() {
        assert_eq!(mate_in(0), MATE_SCORE);
        assert_eq!(mate_in(10), MATE_SCORE - 10);
        assert_eq!(mated_in(0), -MATE_SCORE);
        assert_eq!(mated_in(10), -MATE_SCORE + 10);
    }
}
