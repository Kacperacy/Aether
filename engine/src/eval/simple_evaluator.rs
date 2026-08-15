use crate::eval::{EvalState, Evaluator, MAX_GAME_PHASE};
use aether_core::{BitBoard, Color, File, Piece, Square};
use board::Board;

const BISHOP_PAIR_MG: i32 = 30;
const BISHOP_PAIR_EG: i32 = 50;

const PINNED_PIECE_PENALTY_MG: i32 = 15;
const PINNED_PIECE_PENALTY_EG: i32 = 10;

const PINNER_BONUS_MG: i32 = 10;
const PINNER_BONUS_EG: i32 = 5;

const PASSED_PAWN_BONUS_MG: [i32; 6] = [5, 12, 25, 50, 100, 180];
const PASSED_PAWN_BONUS_EG: [i32; 6] = [15, 30, 55, 95, 160, 260];

const WHITE_PASSED_MASKS: [BitBoard; Square::NUM] = compute_white_passed_masks();
const BLACK_PASSED_MASKS: [BitBoard; Square::NUM] = compute_black_passed_masks();

const fn compute_white_passed_masks() -> [BitBoard; Square::NUM] {
    let mut masks = [BitBoard::EMPTY; Square::NUM];

    let mut sq = 0;
    while sq < Square::NUM as i8 {
        let file = sq % 8;
        let rank = sq / 8;

        if rank >= 1 && rank <= 6 {
            let mut blocking_files = File::from_index(file).bitboard().value();
            if file > 0 {
                blocking_files |= File::from_index(file - 1).bitboard().value();
            }
            if file < 7 {
                blocking_files |= File::from_index(file + 1).bitboard().value();
            }

            let ahead_mask = !((1u64 << (8 * (rank + 1))) - 1);

            masks[sq as usize] = BitBoard::new(blocking_files & ahead_mask);
        }
        sq += 1;
    }
    masks
}

const fn compute_black_passed_masks() -> [BitBoard; Square::NUM] {
    let mut masks = [BitBoard::EMPTY; Square::NUM];

    let mut sq = 0;
    while sq < Square::NUM as i8 {
        let file = sq % 8;
        let rank = sq / 8;

        if rank >= 1 && rank <= 6 {
            let mut blocking_files = File::from_index(file).bitboard().value();
            if file > 0 {
                blocking_files |= File::from_index(file - 1).bitboard().value();
            }
            if file < 7 {
                blocking_files |= File::from_index(file + 1).bitboard().value();
            }

            let ahead_mask = (1u64 << (8 * rank)) - 1;

            masks[sq as usize] = BitBoard::new(blocking_files & ahead_mask);
        }
        sq += 1;
    }
    masks
}

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

        for square in white_pawns.iter() {
            let sq_idx = square.to_index() as usize;
            let mask = WHITE_PASSED_MASKS[sq_idx];

            if (black_pawns & mask).is_empty() {
                let rank_idx = sq_idx / 8;
                if (1..=6).contains(&rank_idx) {
                    mg_score += PASSED_PAWN_BONUS_MG[rank_idx - 1];
                    eg_score += PASSED_PAWN_BONUS_EG[rank_idx - 1];
                }
            }
        }

        for square in black_pawns.iter() {
            let sq_idx = square.to_index() as usize;
            let mask = BLACK_PASSED_MASKS[sq_idx];

            if (white_pawns & mask).is_empty() {
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

        let white_pinned = (white_blockers & white_pieces).count() as i32;
        let black_pinned = (black_blockers & black_pieces).count() as i32;

        let white_pinning = board.pinners(Color::Black).count() as i32;
        let black_pinning = board.pinners(Color::White).count() as i32;

        let mg = (black_pinned - white_pinned) * PINNED_PIECE_PENALTY_MG
            + (white_pinning - black_pinning) * PINNER_BONUS_MG;
        let eg = (black_pinned - white_pinned) * PINNED_PIECE_PENALTY_EG
            + (white_pinning - black_pinning) * PINNER_BONUS_EG;

        (mg, eg)
    }

    #[inline(always)]
    fn evaluate_position(&self, board: &Board, eval_state: &EvalState) -> i32 {
        let (mut mg_score, mut eg_score) = eval_state.scores();

        let (mg_bonus, eg_bonus) = Self::bishop_pair_bonus(board);
        mg_score += mg_bonus;
        eg_score += eg_bonus;

        let (passed_mg, passed_eg) = Self::evaluate_passed_pawns(board);
        mg_score += passed_mg;
        eg_score += passed_eg;

        let (pinned_mg, pinned_eg) = Self::evaluate_pins(board);
        mg_score += pinned_mg;
        eg_score += pinned_eg;

        let phase = eval_state.game_phase();
        (mg_score * phase + eg_score * (MAX_GAME_PHASE - phase)) / MAX_GAME_PHASE
    }
}

impl Evaluator for SimpleEvaluator {
    type Acc = EvalState;

    #[inline(always)]
    fn evaluate(&self, board: &Board, eval_state: &EvalState) -> i32 {
        let score = self.evaluate_position(board, eval_state);

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

    fn white_mask(sq: Square) -> BitBoard {
        WHITE_PASSED_MASKS[sq.to_index() as usize]
    }

    fn black_mask(sq: Square) -> BitBoard {
        BLACK_PASSED_MASKS[sq.to_index() as usize]
    }

    #[test]
    fn test_white_passed_pawn_masks() {
        // A white pawn on e4 is passed unless a black pawn stands on the
        // d/e/f files somewhere ahead of it.
        let mask = white_mask(Square::E4);

        for sq in [Square::D5, Square::E5, Square::F5, Square::E8] {
            assert!(mask.contains(sq), "{sq} should be in the e4 mask");
        }
        for sq in [Square::E4, Square::E3] {
            assert!(!mask.contains(sq), "{sq} should not be in the e4 mask");
        }
    }

    #[test]
    fn test_black_passed_pawn_masks() {
        // Mirror image: "ahead" for Black means toward rank 1.
        let mask = black_mask(Square::E5);

        for sq in [Square::E4, Square::E3, Square::E1] {
            assert!(mask.contains(sq), "{sq} should be in the e5 mask");
        }
        for sq in [Square::E5, Square::E6] {
            assert!(!mask.contains(sq), "{sq} should not be in the e5 mask");
        }
    }

    #[test]
    fn test_edge_file_masks_do_not_wrap() {
        let a4 = white_mask(Square::A4);
        assert!(a4.contains(Square::B5), "b5 should be in the a4 mask");
        assert!(
            !a4.contains(Square::C5),
            "a4 mask must not reach the c-file"
        );

        let h4 = white_mask(Square::H4);
        assert!(h4.contains(Square::G5), "g5 should be in the h4 mask");
        assert!(
            !h4.contains(Square::F5),
            "h4 mask must not reach the f-file"
        );
    }
}
