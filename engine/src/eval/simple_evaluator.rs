use crate::eval::Evaluator;
use aether_core::{ALL_SQUARES, Color, Piece, Square};
use board::BoardQuery;

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

// Knights EG: still like center
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

// Bishops EG: similar to MG
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

// Rooks EG: 7th rank still great
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

// Queen EG: centralization more important
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

// King EG: CENTRALIZE! Active king is crucial
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

const KNIGHT_PHASE: i32 = 1;
const BISHOP_PHASE: i32 = 1;
const ROOK_PHASE: i32 = 2;
const QUEEN_PHASE: i32 = 4;
const TOTAL_PHASE: i32 = KNIGHT_PHASE * 4 + BISHOP_PHASE * 4 + ROOK_PHASE * 4 + QUEEN_PHASE * 2;

#[derive(Debug, Clone, Copy, Default)]
pub struct SimpleEvaluator;

impl SimpleEvaluator {
    pub fn new() -> Self {
        Self
    }

    #[inline]
    fn pst_index(square: Square, color: Color) -> usize {
        let idx = square.to_index() as usize;
        if color == Color::White { idx ^ 56 } else { idx }
    }

    /// Calculate game phase (0 = endgame, 256 = opening/middlegame)
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

        // phase = TOTAL_PHASE means no pieces captured (opening)
        // phase = 0 means all pieces captured (pure endgame)
        // Normalize so 256 = opening, 0 = endgame
        ((TOTAL_PHASE - phase) * 256 / TOTAL_PHASE).max(0)
    }

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

    fn evaluate_position<T: BoardQuery>(&self, board: &T) -> i32 {
        let mut mg_score = 0i32;
        let mut eg_score = 0i32;

        for &square in &ALL_SQUARES {
            if let Some((piece, color)) = board.piece_at(square) {
                let material = piece.value() as i32;
                let idx = Self::pst_index(square, color);
                let (pst_mg, pst_eg) = Self::pst_values(piece, idx);

                let mg_piece = material + pst_mg;
                let eg_piece = material + pst_eg;

                if color == Color::White {
                    mg_score += mg_piece;
                    eg_score += eg_piece;
                } else {
                    mg_score -= mg_piece;
                    eg_score -= eg_piece;
                }
            }
        }

        // Bishop pair bonus
        if board.piece_count(Piece::Bishop, Color::White) >= 2 {
            mg_score += 23;
            eg_score += 62;
        }
        if board.piece_count(Piece::Bishop, Color::Black) >= 2 {
            mg_score -= 23;
            eg_score -= 62;
        }

        // Interpolate between middlegame and endgame scores
        let phase = Self::calculate_phase(board);
        (mg_score * phase + eg_score * (256 - phase)) / 256
    }
}

impl Evaluator for SimpleEvaluator {
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
