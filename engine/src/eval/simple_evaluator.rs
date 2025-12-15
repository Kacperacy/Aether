use crate::eval::Evaluator;
use aether_core::{ALL_SQUARES, Color, Piece, Square};
use board::BoardQuery;

/// Piece-square tables for positional evaluation
/// Values are from White's perspective, mirrored for Black
#[rustfmt::skip]
const PAWN_PST: [i32; 64] = [
     0,  0,  0,  0,  0,  0,  0,  0,
    50, 50, 50, 50, 50, 50, 50, 50,
    10, 10, 20, 30, 30, 20, 10, 10,
     5,  5, 10, 25, 25, 10,  5,  5,
     0,  0,  0, 20, 20,  0,  0,  0,
     5, -5,-10,  0,  0,-10, -5,  5,
     5, 10, 10,-20,-20, 10, 10,  5,
     0,  0,  0,  0,  0,  0,  0,  0,
];

#[rustfmt::skip]
const KNIGHT_PST: [i32; 64] = [
    -50,-40,-30,-30,-30,-30,-40,-50,
    -40,-20,  0,  0,  0,  0,-20,-40,
    -30,  0, 10, 15, 15, 10,  0,-30,
    -30,  5, 15, 20, 20, 15,  5,-30,
    -30,  0, 15, 20, 20, 15,  0,-30,
    -30,  5, 10, 15, 15, 10,  5,-30,
    -40,-20,  0,  5,  5,  0,-20,-40,
    -50,-40,-30,-30,-30,-30,-40,-50,
];

#[rustfmt::skip]
const BISHOP_PST: [i32; 64] = [
    -20,-10,-10,-10,-10,-10,-10,-20,
    -10,  0,  0,  0,  0,  0,  0,-10,
    -10,  0,  5, 10, 10,  5,  0,-10,
    -10,  5,  5, 10, 10,  5,  5,-10,
    -10,  0, 10, 10, 10, 10,  0,-10,
    -10, 10, 10, 10, 10, 10, 10,-10,
    -10,  5,  0,  0,  0,  0,  5,-10,
    -20,-10,-10,-10,-10,-10,-10,-20,
];

#[rustfmt::skip]
const ROOK_PST: [i32; 64] = [
     0,  0,  0,  0,  0,  0,  0,  0,
     5, 10, 10, 10, 10, 10, 10,  5,
    -5,  0,  0,  0,  0,  0,  0, -5,
    -5,  0,  0,  0,  0,  0,  0, -5,
    -5,  0,  0,  0,  0,  0,  0, -5,
    -5,  0,  0,  0,  0,  0,  0, -5,
    -5,  0,  0,  0,  0,  0,  0, -5,
     0,  0,  0,  5,  5,  0,  0,  0,
];

#[rustfmt::skip]
const QUEEN_PST: [i32; 64] = [
    -20,-10,-10, -5, -5,-10,-10,-20,
    -10,  0,  0,  0,  0,  0,  0,-10,
    -10,  0,  5,  5,  5,  5,  0,-10,
     -5,  0,  5,  5,  5,  5,  0, -5,
      0,  0,  5,  5,  5,  5,  0, -5,
    -10,  5,  5,  5,  5,  5,  0,-10,
    -10,  0,  5,  0,  0,  0,  0,-10,
    -20,-10,-10, -5, -5,-10,-10,-20,
];

#[rustfmt::skip]
const KING_MIDDLEGAME_PST: [i32; 64] = [
    -30,-40,-40,-50,-50,-40,-40,-30,
    -30,-40,-40,-50,-50,-40,-40,-30,
    -30,-40,-40,-50,-50,-40,-40,-30,
    -30,-40,-40,-50,-50,-40,-40,-30,
    -20,-30,-30,-40,-40,-30,-30,-20,
    -10,-20,-20,-20,-20,-20,-20,-10,
     20, 20,  0,  0,  0,  0, 20, 20,
     20, 30, 10,  0,  0, 10, 30, 20,
];

#[rustfmt::skip]
const KING_ENDGAME_PST: [i32; 64] = [
    -50,-40,-30,-20,-20,-30,-40,-50,
    -30,-20,-10,  0,  0,-10,-20,-30,
    -30,-10, 20, 30, 30, 20,-10,-30,
    -30,-10, 30, 40, 40, 30,-10,-30,
    -30,-10, 30, 40, 40, 30,-10,-30,
    -30,-10, 20, 30, 30, 20,-10,-30,
    -30,-30,  0,  0,  0,  0,-30,-30,
    -50,-30,-30,-30,-30,-30,-30,-50,
];

#[derive(Debug, Clone, Copy, Default)]
pub struct SimpleEvaluator;

impl SimpleEvaluator {
    /// Creates a new SimpleEvaluator
    pub fn new() -> Self {
        Self
    }

    /// Get PST index for a square from a given color's perspective
    #[inline]
    fn pst_index(square: Square, color: Color) -> usize {
        let idx = square.to_index() as usize;
        if color == Color::White {
            // Mirror vertically for white (rank 0 -> rank 7)
            idx ^ 56
        } else {
            idx
        }
    }

    /// Check if we're in the endgame (for king PST selection)
    fn is_endgame<T: BoardQuery>(&self, board: &T) -> bool {
        let white_queens = board.piece_count(Piece::Queen, Color::White);
        let black_queens = board.piece_count(Piece::Queen, Color::Black);

        // Endgame if both sides have no queens, or
        // if a side has at most 1 minor piece besides pawns and king
        if white_queens == 0 && black_queens == 0 {
            return true;
        }

        let white_minors = board.piece_count(Piece::Knight, Color::White)
            + board.piece_count(Piece::Bishop, Color::White);
        let black_minors = board.piece_count(Piece::Knight, Color::Black)
            + board.piece_count(Piece::Bishop, Color::Black);
        let white_rooks = board.piece_count(Piece::Rook, Color::White);
        let black_rooks = board.piece_count(Piece::Rook, Color::Black);

        // Simple heuristic: endgame if limited material
        white_queens + black_queens <= 1
            && white_rooks + black_rooks <= 2
            && white_minors + black_minors <= 2
    }

    /// Get piece-square table value for a piece
    fn piece_square_value(
        &self,
        piece: Piece,
        square: Square,
        color: Color,
        is_endgame: bool,
    ) -> i32 {
        let idx = Self::pst_index(square, color);

        match piece {
            Piece::Pawn => PAWN_PST[idx],
            Piece::Knight => KNIGHT_PST[idx],
            Piece::Bishop => BISHOP_PST[idx],
            Piece::Rook => ROOK_PST[idx],
            Piece::Queen => QUEEN_PST[idx],
            Piece::King => {
                if is_endgame {
                    KING_ENDGAME_PST[idx]
                } else {
                    KING_MIDDLEGAME_PST[idx]
                }
            }
        }
    }

    /// Complete evaluation including material and positional factors
    fn evaluate_position<T: BoardQuery>(&self, board: &T) -> i32 {
        let mut score = 0i32;
        let is_endgame = self.is_endgame(board);

        for &square in &ALL_SQUARES {
            if let Some((piece, color)) = board.piece_at(square) {
                let material = piece.value() as i32;
                let positional = self.piece_square_value(piece, square, color, is_endgame);

                let piece_score = material + positional;

                if color == Color::White {
                    score += piece_score;
                } else {
                    score -= piece_score;
                }
            }
        }

        // Bishop pair bonus
        if board.piece_count(Piece::Bishop, Color::White) >= 2 {
            score += 30;
        }
        if board.piece_count(Piece::Bishop, Color::Black) >= 2 {
            score -= 30;
        }

        score
    }
}

impl Evaluator for SimpleEvaluator {
    fn evaluate<T: BoardQuery>(&self, board: &T) -> i32 {
        let score = self.evaluate_position(board);

        // Return from side to move's perspective
        if board.side_to_move() == Color::White {
            score
        } else {
            -score
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_pst_index_symmetry() {
        // a1 for white should map to a8 position (index 56)
        let a1 = Square::A1;
        assert_eq!(SimpleEvaluator::pst_index(a1, Color::White), 56);

        // a1 for black stays at a1 (index 0)
        assert_eq!(SimpleEvaluator::pst_index(a1, Color::Black), 0);

        // e4 for white
        let e4 = Square::E4;
        let white_idx = SimpleEvaluator::pst_index(e4, Color::White);
        let black_idx = SimpleEvaluator::pst_index(e4, Color::Black);

        // They should be vertically mirrored
        assert_eq!(white_idx ^ 56, black_idx);
    }
}
