use crate::Evaluator;
use aether_types::{ALL_COLORS, ALL_PIECES, BoardQuery, Color};

#[derive(Debug, Clone, Copy, Default)]
pub struct SimpleEvaluator;

impl SimpleEvaluator {
    /// Creates a new SimpleEvaluator
    pub fn new() -> Self {
        Self
    }

    /// Simple material balance evaluation
    fn material_balance<T: BoardQuery>(&self, board: &T) -> i32 {
        let mut score = 0;

        for color in ALL_COLORS {
            for piece in ALL_PIECES {
                let piece_count = board.piece_count(piece, color);

                if color == Color::White {
                    score += piece.value() as i32 * piece_count as i32;
                } else {
                    score -= piece.value() as i32 * piece_count as i32;
                }
            }
        }

        score
    }
}

impl Evaluator for SimpleEvaluator {
    fn evaluate<T: BoardQuery>(&self, board: &T) -> i32 {
        if board.side_to_move() == Color::White {
            self.material_balance(board)
        } else {
            -self.material_balance(board)
        }
    }
}
