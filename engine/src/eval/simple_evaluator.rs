use crate::eval::Evaluator;
use aether_core::{ALL_PIECES, Color, Piece, Square};
use board::BoardQuery;

// --- Piece-Square Tables ---
// Values represent positional bonuses/penalties in centipawns.
// Tables are from white's perspective; black's values are mirrored.

/// Pawn middlegame piece-square table
/// Encourages central pawn control and advancement
#[rustfmt::skip]
const PAWN_PST_MG: [i32; 64] = [
      0,   0,   0,   0,   0,   0,   0,   0,
     50,  50,  50,  50,  50,  50,  50,  50,
     20,  20,  30,  40,  40,  30,  20,  20,
     10,  10,  20,  30,  30,  20,  10,  10,
      5,   5,  15,  25,  25,  15,   5,   5,
      0,   0,  10,  20,  20,  10,   0,   0,
      5,  10,   0,  -5,  -5,   0,  10,   5,
      0,   0,   0,   0,   0,   0,   0,   0,
];

/// Knight middlegame piece-square table
/// Knights are strongest in the center with maximum mobility
#[rustfmt::skip]
const KNIGHT_PST_MG: [i32; 64] = [
    -50, -40, -30, -30, -30, -30, -40, -50,
    -40, -20,   0,   5,   5,   0, -20, -40,
    -30,   5,  15,  20,  20,  15,   5, -30,
    -30,   0,  20,  25,  25,  20,   0, -30,
    -30,   5,  20,  25,  25,  20,   5, -30,
    -30,   0,  15,  20,  20,  15,   0, -30,
    -40, -20,   0,   5,   5,   0, -20, -40,
    -50, -40, -30, -30, -30, -30, -40, -50,
];

/// Bishop middlegame piece-square table
/// Bishops prefer long diagonals and active positions
#[rustfmt::skip]
const BISHOP_PST_MG: [i32; 64] = [
    -20, -10, -10, -10, -10, -10, -10, -20,
    -10,   5,   0,   0,   0,   0,   5, -10,
    -10,  10,  10,  10,  10,  10,  10, -10,
    -10,   0,  15,  15,  15,  15,   0, -10,
    -10,   5,  15,  15,  15,  15,   5, -10,
    -10,   0,  10,  15,  15,  10,   0, -10,
    -10,  10,   0,   5,   5,   0,  10, -10,
    -20, -10, -10, -10, -10, -10, -10, -20,
];

/// Rook middlegame piece-square table
/// Rooks gain value on open files and the seventh rank
#[rustfmt::skip]
const ROOK_PST_MG: [i32; 64] = [
      0,   0,   0,   5,   5,   0,   0,   0,
     10,  15,  15,  20,  20,  15,  15,  10,
      0,   0,   0,   5,   5,   0,   0,   0,
      0,   0,   0,   5,   5,   0,   0,   0,
      0,   0,   0,   5,   5,   0,   0,   0,
      0,   0,   0,   5,   5,   0,   0,   0,
      0,   0,   0,   5,   5,   0,   0,   0,
     -5,   0,   5,  10,  10,   5,   0,  -5,
];

/// Queen middlegame piece-square table
/// Queen should avoid early development but control center when active
#[rustfmt::skip]
const QUEEN_PST_MG: [i32; 64] = [
    -20, -10, -10,  -5,  -5, -10, -10, -20,
    -10,   0,   5,   5,   5,   5,   0, -10,
    -10,   5,  10,  10,  10,  10,   5, -10,
     -5,   0,  10,  10,  10,  10,   0,  -5,
     -5,   0,  10,  10,  10,  10,   0,  -5,
    -10,   0,   5,  10,  10,   5,   0, -10,
    -10,   0,   0,   0,   0,   0,   0, -10,
    -20, -10, -10,  -5,  -5, -10, -10, -20,
];

/// King middlegame piece-square table
/// King safety is paramount; stay castled and protected
#[rustfmt::skip]
const KING_PST_MG: [i32; 64] = [
    -50, -40, -30, -20, -20, -30, -40, -50,
    -30, -20, -10,   0,   0, -10, -20, -30,
    -30, -10,  20,  30,  30,  20, -10, -30,
    -30, -10,  30,  40,  40,  30, -10, -30,
    -30, -10,  30,  40,  40,  30, -10, -30,
    -30, -10,  20,  30,  30,  20, -10, -30,
    -30, -30, -10,   0,   0, -10, -30, -30,
     20,  30,  -5, -30, -10, -30,  30,  20,
];

/// Pawn endgame piece-square table
/// Passed pawns and advancement become critical
#[rustfmt::skip]
const PAWN_PST_EG: [i32; 64] = [
      0,   0,   0,   0,   0,   0,   0,   0,
    100, 100, 100, 100, 100, 100, 100, 100,
     60,  60,  60,  60,  60,  60,  60,  60,
     40,  40,  40,  40,  40,  40,  40,  40,
     20,  20,  20,  20,  20,  20,  20,  20,
     10,  10,  10,  10,  10,  10,  10,  10,
      5,   5,   5,   5,   5,   5,   5,   5,
      0,   0,   0,   0,   0,   0,   0,   0,
];

/// Knight endgame piece-square table
/// Central knights maintain good mobility
#[rustfmt::skip]
const KNIGHT_PST_EG: [i32; 64] = [
    -50, -40, -30, -30, -30, -30, -40, -50,
    -40, -20,   0,   0,   0,   0, -20, -40,
    -30,   0,  10,  15,  15,  10,   0, -30,
    -30,   5,  15,  20,  20,  15,   5, -30,
    -30,   0,  15,  20,  20,  15,   0, -30,
    -30,   5,  10,  15,  15,  10,   5, -30,
    -40, -20,   0,   5,   5,   0, -20, -40,
    -50, -40, -30, -30, -30, -30, -40, -50,
];

/// Bishop endgame piece-square table
/// Diagonal control remains important for restricting enemy king
#[rustfmt::skip]
const BISHOP_PST_EG: [i32; 64] = [
    -20, -10, -10, -10, -10, -10, -10, -20,
    -10,   0,   0,   0,   0,   0,   0, -10,
    -10,   0,  10,  10,  10,  10,   0, -10,
    -10,   5,  10,  15,  15,  10,   5, -10,
    -10,   0,  10,  15,  15,  10,   0, -10,
    -10,  10,  10,  10,  10,  10,  10, -10,
    -10,   5,   0,   0,   0,   0,   5, -10,
    -20, -10, -10, -10, -10, -10, -10, -20,
];

/// Rook endgame piece-square table
/// Seventh rank remains strong for attacking pawns and restricting king
#[rustfmt::skip]
const ROOK_PST_EG: [i32; 64] = [
      0,   0,   0,   0,   0,   0,   0,   0,
     15,  15,  15,  15,  15,  15,  15,  15,
      0,   0,   0,   0,   0,   0,   0,   0,
      0,   0,   0,   0,   0,   0,   0,   0,
      0,   0,   0,   0,   0,   0,   0,   0,
      0,   0,   0,   0,   0,   0,   0,   0,
      0,   0,   0,   0,   0,   0,   0,   0,
      0,   0,   0,   0,   0,   0,   0,   0,
];

/// Queen endgame piece-square table
/// Centralization maximizes queen activity in open endgames
#[rustfmt::skip]
const QUEEN_PST_EG: [i32; 64] = [
    -20, -10, -10,  -5,  -5, -10, -10, -20,
    -10,   0,   5,   5,   5,   5,   0, -10,
    -10,   5,  10,  15,  15,  10,   5, -10,
     -5,   0,  15,  20,  20,  15,   0,  -5,
     -5,   0,  15,  20,  20,  15,   0,  -5,
    -10,   5,  10,  15,  15,  10,   5, -10,
    -10,   0,   0,   5,   5,   0,   0, -10,
    -20, -10, -10,  -5,  -5, -10, -10, -20,
];

/// King endgame piece-square table
/// Active centralized king is essential for both offense and defense
#[rustfmt::skip]
const KING_PST_EG: [i32; 64] = [
    -50, -30, -20, -10, -10, -20, -30, -50,
    -30, -10,   0,  10,  10,   0, -10, -30,
    -20,   0,  20,  30,  30,  20,   0, -20,
    -10,  10,  30,  40,  40,  30,  10, -10,
    -10,  10,  30,  40,  40,  30,  10, -10,
    -20,   0,  20,  30,  30,  20,   0, -20,
    -30, -10,   0,  10,  10,   0, -10, -30,
    -50, -30, -20, -10, -10, -20, -30, -50,
];

/// Phase weight for knights (minor pieces)
const KNIGHT_PHASE: i32 = 1;
/// Phase weight for bishops (minor pieces)
const BISHOP_PHASE: i32 = 1;
/// Phase weight for rooks (major pieces)
const ROOK_PHASE: i32 = 2;
/// Phase weight for queens (strongest piece)
const QUEEN_PHASE: i32 = 4;
/// Total phase at game start (4 knights + 4 bishops + 4 rooks + 2 queens)
const TOTAL_PHASE: i32 = KNIGHT_PHASE * 4 + BISHOP_PHASE * 4 + ROOK_PHASE * 4 + QUEEN_PHASE * 2;

/// Bishop pair bonus in middlegame (centipawns)
const BISHOP_PAIR_MG: i32 = 23;
/// Bishop pair bonus in endgame (centipawns)
const BISHOP_PAIR_EG: i32 = 62;

/// Simple positional evaluator using piece-square tables
#[derive(Debug, Clone, Copy, Default)]
pub struct SimpleEvaluator;

impl SimpleEvaluator {
    /// Creates a new SimpleEvaluator instance
    #[must_use]
    pub fn new() -> Self {
        Self
    }

    /// Converts a square index for PST lookup based on color
    /// White pieces use flipped index (from white's perspective)
    #[inline]
    fn pst_index(square: Square, color: Color) -> usize {
        let idx = square.to_index() as usize;
        if color == Color::White { idx ^ 56 } else { idx }
    }

    /// Calculates game phase based on remaining material
    /// Returns value from 0 (endgame) to 256 (opening/middlegame)
    fn calculate_phase<T: BoardQuery>(board: &T) -> i32 {
        let mut phase = TOTAL_PHASE;

        phase -= board.piece_count(Piece::Knight, Color::White) as i32 * KNIGHT_PHASE;
        phase -= board.piece_count(Piece::Knight, Color::Black) as i32 * KNIGHT_PHASE;
        phase -= board.piece_count(Piece::Bishop, Color::White) as i32 * BISHOP_PHASE;
        phase -= board.piece_count(Piece::Bishop, Color::Black) as i32 * BISHOP_PHASE;
        phase -= board.piece_count(Piece::Rook, Color::White) as i32 * ROOK_PHASE;
        phase -= board.piece_count(Piece::Rook, Color::Black) as i32 * ROOK_PHASE;
        phase -= board.piece_count(Piece::Queen, Color::White) as i32 * QUEEN_PHASE;
        phase -= board.piece_count(Piece::Queen, Color::Black) as i32 * QUEEN_PHASE;

        ((TOTAL_PHASE - phase) * 256 / TOTAL_PHASE).max(0)
    }

    /// Returns middlegame and endgame PST values for a piece at given index
    #[inline]
    fn pst_values(piece: Piece, idx: usize) -> (i32, i32) {
        match piece {
            Piece::Pawn => (PAWN_PST_MG[idx], PAWN_PST_EG[idx]),
            Piece::Knight => (KNIGHT_PST_MG[idx], KNIGHT_PST_EG[idx]),
            Piece::Bishop => (BISHOP_PST_MG[idx], BISHOP_PST_EG[idx]),
            Piece::Rook => (ROOK_PST_MG[idx], ROOK_PST_EG[idx]),
            Piece::Queen => (QUEEN_PST_MG[idx], QUEEN_PST_EG[idx]),
            Piece::King => (KING_PST_MG[idx], KING_PST_EG[idx]),
        }
    }

    /// Calculates bishop pair bonus for both sides
    fn bishop_pair_bonus<T: BoardQuery>(board: &T) -> (i32, i32) {
        let white_pair = board.piece_count(Piece::Bishop, Color::White) >= 2;
        let black_pair = board.piece_count(Piece::Bishop, Color::Black) >= 2;

        let mg = if white_pair { BISHOP_PAIR_MG } else { 0 }
            - if black_pair { BISHOP_PAIR_MG } else { 0 };
        let eg = if white_pair { BISHOP_PAIR_EG } else { 0 }
            - if black_pair { BISHOP_PAIR_EG } else { 0 };

        (mg, eg)
    }

    /// Evaluates position from white's perspective
    fn evaluate_position<T: BoardQuery>(&self, board: &T) -> i32 {
        let mut mg_score = 0i32;
        let mut eg_score = 0i32;

        // Iterate directly through piece bitboards - avoids piece_at() lookup per square
        for &piece in &ALL_PIECES {
            let material = piece.value() as i32;

            // White pieces
            for square in board.piece_bb(piece, Color::White).iter() {
                let idx = Self::pst_index(square, Color::White);
                let (pst_mg, pst_eg) = Self::pst_values(piece, idx);
                mg_score += material + pst_mg;
                eg_score += material + pst_eg;
            }

            // Black pieces
            for square in board.piece_bb(piece, Color::Black).iter() {
                let idx = Self::pst_index(square, Color::Black);
                let (pst_mg, pst_eg) = Self::pst_values(piece, idx);
                mg_score -= material + pst_mg;
                eg_score -= material + pst_eg;
            }
        }

        let (mg_bonus, eg_bonus) = Self::bishop_pair_bonus(board);
        mg_score += mg_bonus;
        eg_score += eg_bonus;

        let phase = Self::calculate_phase(board);
        (mg_score * phase + eg_score * (256 - phase)) / 256
    }
}

impl Evaluator for SimpleEvaluator {
    /// Evaluates position from the side to move's perspective
    fn evaluate<T: BoardQuery>(&self, board: &T) -> i32 {
        let score = self.evaluate_position(board);

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
        let a1 = Square::A1;
        assert_eq!(SimpleEvaluator::pst_index(a1, Color::White), 56);
        assert_eq!(SimpleEvaluator::pst_index(a1, Color::Black), 0);

        let e4 = Square::E4;
        let white_idx = SimpleEvaluator::pst_index(e4, Color::White);
        let black_idx = SimpleEvaluator::pst_index(e4, Color::Black);
        assert_eq!(white_idx ^ 56, black_idx);
    }
}
