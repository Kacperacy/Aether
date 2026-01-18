use crate::eval::Evaluator;
use aether_core::{Color, Piece};
use board::Board;

/// Bishop pair bonus in middlegame (centipawns)
const BISHOP_PAIR_MG: i32 = 30;
/// Bishop pair bonus in endgame (centipawns)
const BISHOP_PAIR_EG: i32 = 50;

/// Penalty for pinned pieces (centipawns per pinned piece)
const PINNED_PIECE_PENALTY_MG: i32 = 15;
const PINNED_PIECE_PENALTY_EG: i32 = 10;

/// Bonus for pinning enemy pieces (centipawns per pinner)
const PINNER_BONUS_MG: i32 = 10;
const PINNER_BONUS_EG: i32 = 5;

/// Passed pawn bonus by rank (from pawn's perspective, index 0 = rank 2, index 5 = rank 7)
const PASSED_PAWN_BONUS_MG: [i32; 6] = [5, 12, 25, 50, 100, 180];
const PASSED_PAWN_BONUS_EG: [i32; 6] = [15, 30, 55, 95, 160, 260];

/// Precomputed masks for passed pawn detection
/// For white pawn on square S, WHITE_PASSED_MASKS[S] contains all squares
/// that must be empty of black pawns for it to be passed
const WHITE_PASSED_MASKS: [u64; 64] = compute_white_passed_masks();
const BLACK_PASSED_MASKS: [u64; 64] = compute_black_passed_masks();

/// Computes passed pawn masks for white pawns at compile time
const fn compute_white_passed_masks() -> [u64; 64] {
    let mut masks = [0u64; 64];
    let file_masks: [u64; 8] = [
        0x0101010101010101, // A
        0x0202020202020202, // B
        0x0404040404040404, // C
        0x0808080808080808, // D
        0x1010101010101010, // E
        0x2020202020202020, // F
        0x4040404040404040, // G
        0x8080808080808080, // H
    ];

    let mut sq = 0;
    while sq < 64 {
        let file = sq % 8;
        let rank = sq / 8;

        // Only ranks 1-6 can have passed pawns (pawns don't exist on 0 or 7)
        if rank >= 1 && rank <= 6 {
            // Files that block: same file + adjacent files
            let mut blocking_files = file_masks[file as usize];
            if file > 0 {
                blocking_files |= file_masks[(file - 1) as usize];
            }
            if file < 7 {
                blocking_files |= file_masks[(file + 1) as usize];
            }

            // Ranks ahead (for white, higher ranks)
            let ahead_mask = !((1u64 << (8 * (rank + 1))) - 1);

            masks[sq as usize] = blocking_files & ahead_mask;
        }
        sq += 1;
    }
    masks
}

/// Computes passed pawn masks for black pawns at compile time
const fn compute_black_passed_masks() -> [u64; 64] {
    let mut masks = [0u64; 64];
    let file_masks: [u64; 8] = [
        0x0101010101010101,
        0x0202020202020202,
        0x0404040404040404,
        0x0808080808080808,
        0x1010101010101010,
        0x2020202020202020,
        0x4040404040404040,
        0x8080808080808080,
    ];

    let mut sq = 0;
    while sq < 64 {
        let file = sq % 8;
        let rank = sq / 8;

        // Only ranks 1-6 can have passed pawns
        if rank >= 1 && rank <= 6 {
            let mut blocking_files = file_masks[file as usize];
            if file > 0 {
                blocking_files |= file_masks[(file - 1) as usize];
            }
            if file < 7 {
                blocking_files |= file_masks[(file + 1) as usize];
            }

            // Ranks ahead (for black, lower ranks)
            let ahead_mask = (1u64 << (8 * rank)) - 1;

            masks[sq as usize] = blocking_files & ahead_mask;
        }
        sq += 1;
    }
    masks
}

/// Simple positional evaluator using piece-square tables
#[derive(Debug, Clone, Copy, Default)]
pub struct SimpleEvaluator;

impl SimpleEvaluator {
    #[must_use]
    pub fn new() -> Self {
        Self
    }

    fn bishop_pair_bonus(board: &Board) -> (i32, i32) {
        let white_pair = board.piece_count(Piece::Bishop, Color::White) >= 2;
        let black_pair = board.piece_count(Piece::Bishop, Color::Black) >= 2;

        let mg = if white_pair { BISHOP_PAIR_MG } else { 0 }
            - if black_pair { BISHOP_PAIR_MG } else { 0 };
        let eg = if white_pair { BISHOP_PAIR_EG } else { 0 }
            - if black_pair { BISHOP_PAIR_EG } else { 0 };

        (mg, eg)
    }

    #[inline]
    fn evaluate_passed_pawns(board: &Board) -> (i32, i32) {
        let mut mg_score = 0;
        let mut eg_score = 0;

        let white_pawns = board.piece_bb(Piece::Pawn, Color::White);
        let black_pawns = board.piece_bb(Piece::Pawn, Color::Black);
        let black_pawns_raw = black_pawns.value();
        let white_pawns_raw = white_pawns.value();

        // Check each white pawn using precomputed masks
        for square in white_pawns.iter() {
            let sq_idx = square.to_index() as usize;
            let mask = WHITE_PASSED_MASKS[sq_idx];

            // Single AND + comparison instead of multiple bitboard operations
            if (black_pawns_raw & mask) == 0 {
                // Bonus index: rank 2 = index 0, rank 7 = index 5
                let rank_idx = sq_idx / 8;
                if (1..=6).contains(&rank_idx) {
                    mg_score += PASSED_PAWN_BONUS_MG[rank_idx - 1];
                    eg_score += PASSED_PAWN_BONUS_EG[rank_idx - 1];
                }
            }
        }

        // Check each black pawn using precomputed masks
        for square in black_pawns.iter() {
            let sq_idx = square.to_index() as usize;
            let mask = BLACK_PASSED_MASKS[sq_idx];

            if (white_pawns_raw & mask) == 0 {
                // Bonus index from black's perspective: rank 7 = index 0, rank 2 = index 5
                let rank_idx = sq_idx / 8;
                if (1..=6).contains(&rank_idx) {
                    mg_score -= PASSED_PAWN_BONUS_MG[6 - rank_idx];
                    eg_score -= PASSED_PAWN_BONUS_EG[6 - rank_idx];
                }
            }
        }

        (mg_score, eg_score)
    }

    #[inline]
    fn evaluate_pins(board: &Board) -> (i32, i32) {
        let white_blockers = board.blockers_for_king(Color::White);
        let black_blockers = board.blockers_for_king(Color::Black);

        let white_pieces = board.occupied_by(Color::White);
        let black_pieces = board.occupied_by(Color::Black);

        // Count pinned pieces (our pieces blocking attacks on our king)
        let white_pinned = (white_blockers & white_pieces).count() as i32;
        let black_pinned = (black_blockers & black_pieces).count() as i32;

        // Count pinners (our pieces pinning enemy pieces to enemy king)
        // pinners[White] = black sliders pinning white pieces, so white_pinners counts black's pinning power
        // pinners[Black] = white sliders pinning black pieces, so black_pinners counts white's pinning power
        let white_pinning = board.pinners(Color::Black).count() as i32; // white sliders pinning black
        let black_pinning = board.pinners(Color::White).count() as i32; // black sliders pinning white

        let mg = (black_pinned - white_pinned) * PINNED_PIECE_PENALTY_MG
            + (white_pinning - black_pinning) * PINNER_BONUS_MG;
        let eg = (black_pinned - white_pinned) * PINNED_PIECE_PENALTY_EG
            + (white_pinning - black_pinning) * PINNER_BONUS_EG;

        (mg, eg)
    }

    #[inline(always)]
    fn evaluate_position(&self, board: &Board) -> i32 {
        // Use incrementally updated PST scores - O(1) instead of O(pieces)
        let (mut mg_score, mut eg_score) = board.pst_scores();

        let (mg_bonus, eg_bonus) = Self::bishop_pair_bonus(board);
        mg_score += mg_bonus;
        eg_score += eg_bonus;

        let (passed_mg, passed_eg) = Self::evaluate_passed_pawns(board);
        mg_score += passed_mg;
        eg_score += passed_eg;

        let (pinned_mg, pinned_eg) = Self::evaluate_pins(board);
        mg_score += pinned_mg;
        eg_score += pinned_eg;

        let phase = board.game_phase();
        (mg_score * phase + eg_score * (256 - phase)) / 256
    }
}

impl Evaluator for SimpleEvaluator {
    #[inline(always)]
    fn evaluate(&self, board: &Board) -> i32 {
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
    fn test_white_passed_pawn_masks() {
        // White pawn on e4 (square index 28) should check d5-d8, e5-e8, f5-f8
        let e4_mask = WHITE_PASSED_MASKS[28];
        // d5=35, d6=43, d7=51, d8=59
        // e5=36, e6=44, e7=52, e8=60
        // f5=37, f6=45, f7=53, f8=61
        assert!(e4_mask & (1u64 << 35) != 0, "d5 should be in mask");
        assert!(e4_mask & (1u64 << 36) != 0, "e5 should be in mask");
        assert!(e4_mask & (1u64 << 37) != 0, "f5 should be in mask");
        assert!(e4_mask & (1u64 << 60) != 0, "e8 should be in mask");
        // e4 itself should NOT be in mask
        assert!(e4_mask & (1u64 << 28) == 0, "e4 should NOT be in mask");
        // e3 (below) should NOT be in mask
        assert!(e4_mask & (1u64 << 20) == 0, "e3 should NOT be in mask");
    }

    #[test]
    fn test_black_passed_pawn_masks() {
        // Black pawn on e5 (square index 36) should check d1-d4, e1-e4, f1-f4
        let e5_mask = BLACK_PASSED_MASKS[36];
        // e4=28, e3=20, e2=12, e1=4
        assert!(e5_mask & (1u64 << 28) != 0, "e4 should be in mask");
        assert!(e5_mask & (1u64 << 20) != 0, "e3 should be in mask");
        assert!(e5_mask & (1u64 << 4) != 0, "e1 should be in mask");
        // e5 itself should NOT be in mask
        assert!(e5_mask & (1u64 << 36) == 0, "e5 should NOT be in mask");
        // e6 (above) should NOT be in mask
        assert!(e5_mask & (1u64 << 44) == 0, "e6 should NOT be in mask");
    }

    #[test]
    fn test_edge_file_masks() {
        // White pawn on a4 (square 24) should only check a5-a8 and b5-b8 (no file to the left)
        let a4_mask = WHITE_PASSED_MASKS[24];
        // c5 should NOT be in mask (too far right)
        assert!(a4_mask & (1u64 << 34) == 0, "c5 should NOT be in a4 mask");
        // b5 should be in mask
        assert!(a4_mask & (1u64 << 33) != 0, "b5 should be in a4 mask");

        // White pawn on h4 (square 31) should only check g5-g8 and h5-h8
        let h4_mask = WHITE_PASSED_MASKS[31];
        // f5 should NOT be in mask
        assert!(h4_mask & (1u64 << 37) == 0, "f5 should NOT be in h4 mask");
        // g5 should be in mask
        assert!(h4_mask & (1u64 << 38) != 0, "g5 should be in h4 mask");
    }
}
