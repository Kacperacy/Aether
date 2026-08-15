use crate::eval::material;
use crate::search::MAX_PLY;
use crate::search::see::see_value;
use aether_core::{Move, Piece, Square};
use board::Board;

const REPETITION_PENALTY: i32 = -5000;
const GOOD_CAPTURE_SCORE: i32 = 10_000;
const BAD_CAPTURE_SCORE: i32 = -2_000;

pub struct MoveOrderer {
    killers: [[Option<Move>; 2]; MAX_PLY],
    history: [[i32; 64]; 6],
    repetition_moves: [bool; 64 * 64],
}

impl MoveOrderer {
    pub fn new() -> Self {
        Self {
            killers: [[None; 2]; MAX_PLY],
            history: [[0; 64]; 6],
            repetition_moves: [false; 64 * 64],
        }
    }

    pub fn clear(&mut self) {
        self.killers = [[None; 2]; MAX_PLY];
        self.history = [[0; 64]; 6];
        self.repetition_moves = [false; 64 * 64];
    }

    pub fn clear_repetitions(&mut self) {
        self.repetition_moves = [false; 64 * 64];
    }

    #[inline]
    pub fn mark_repetition_move(&mut self, mv: &Move) {
        let idx = mv.from_sq().to_index() as usize * 64 + mv.to_sq().to_index() as usize;
        self.repetition_moves[idx] = true;
    }

    #[inline]
    fn is_repetition_move(&self, mv: &Move) -> bool {
        let idx = mv.from_sq().to_index() as usize * 64 + mv.to_sq().to_index() as usize;
        self.repetition_moves[idx]
    }

    pub fn update_history(&mut self, mv: Move, piece: Piece, depth: usize) {
        if mv.is_capture() {
            return;
        }

        let bonus = depth as i32 * depth as i32;
        let idx = mv.to_sq().to_index() as usize;
        self.history[piece as usize][idx] += bonus;

        if self.history[piece as usize][idx] > 8_000 {
            self.age_history();
        }
    }

    fn age_history(&mut self) {
        for piece in 0..Piece::NUM {
            for sq in 0..Square::NUM {
                self.history[piece][sq] /= 2;
            }
        }
    }

    #[inline(always)]
    fn history_score(&self, mv: &Move, piece: Piece) -> i32 {
        if mv.is_capture() {
            return 0;
        }
        self.history[piece as usize][mv.to_sq().to_index() as usize]
    }

    #[inline]
    pub fn store_killer(&mut self, mv: Move, ply: usize) {
        if ply >= MAX_PLY || mv.is_capture() || mv.is_promotion() {
            return;
        }

        if self.killers[ply][0] != Some(mv) {
            self.killers[ply][1] = self.killers[ply][0];
            self.killers[ply][0] = Some(mv);
        }
    }

    #[inline]
    fn killer_score(&self, mv: &Move, ply: usize) -> Option<i32> {
        if ply >= MAX_PLY {
            return None;
        }
        if self.killers[ply][0] == Some(*mv) {
            Some(8_500)
        } else if self.killers[ply][1] == Some(*mv) {
            Some(8_000)
        } else {
            None
        }
    }

    pub fn order_moves_with_see(
        &self,
        moves: &mut [Move],
        tt_move: Option<Move>,
        ply: usize,
        board: &Board,
    ) {
        moves.sort_by_cached_key(|mv| {
            std::cmp::Reverse(self.move_score_with_see(mv, tt_move, ply, board))
        });
    }

    #[inline(always)]
    fn move_score_with_see(
        &self,
        mv: &Move,
        tt_move: Option<Move>,
        ply: usize,
        board: &Board,
    ) -> i32 {
        let side = board.side_to_move();
        if Some(*mv) == tt_move {
            return 20_000;
        }

        let moving_piece = board.piece_at(mv.from_sq()).map(|(p, _)| p);

        if mv.is_capture() || mv.is_en_passant() {
            let captured_value = if mv.is_en_passant() {
                material::PAWN_VALUE
            } else {
                board
                    .piece_at(mv.to_sq())
                    .filter(|(_, c)| *c != side)
                    .map_or(0, |(p, _)| material::value(p))
            };
            let promo_bonus = mv.promotion_piece().map(material::value).unwrap_or(0);
            let attacker_value = moving_piece.map(material::value).unwrap_or(0);
            let mvv_lva = captured_value - attacker_value;

            if mvv_lva >= 0 {
                return GOOD_CAPTURE_SCORE + promo_bonus + 10 * captured_value - attacker_value;
            }

            let see = see_value(board, mv);
            return if see >= 0 {
                GOOD_CAPTURE_SCORE + promo_bonus + 10 * captured_value - attacker_value
            } else {
                BAD_CAPTURE_SCORE + promo_bonus + see
            };
        }

        if let Some(promo) = mv.promotion_piece() {
            return 9_000 + material::value(promo);
        }

        if let Some(killer_score) = self.killer_score(mv, ply) {
            return killer_score;
        }

        if self.is_repetition_move(mv) {
            return REPETITION_PENALTY;
        }

        moving_piece.map(|p| self.history_score(mv, p)).unwrap_or(0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use crate::search::test_support::pos;

    fn score(orderer: &MoveOrderer, board: &Board, mv: &Move) -> i32 {
        orderer.move_score_with_see(mv, None, 0, board)
    }

    #[test]
    fn test_captures_ordered_by_mvv_lva() {
        let orderer = MoveOrderer::new();
        // White pawn e4 can take the queen on d5; knight f3 can take the pawn e5.
        let board = pos("7k/8/8/3qp3/4P3/5N2/8/K7 w - - 0 1");

        let pawn_takes_queen = Move::new(Square::E4, Square::D5, Move::CAPTURE);
        let knight_takes_pawn = Move::new(Square::F3, Square::E5, Move::CAPTURE);

        assert!(
            score(&orderer, &board, &pawn_takes_queen)
                > score(&orderer, &board, &knight_takes_pawn)
        );
    }

    #[test]
    fn test_promotion_scores_above_quiet() {
        let orderer = MoveOrderer::new();
        let board = pos("7k/4P3/8/8/8/8/4P3/K7 w - - 0 1");

        let promotion = Move::new(Square::E7, Square::E8, Move::PROMO_Q);
        let quiet = Move::new(Square::E2, Square::E4, Move::DOUBLE_PUSH);

        assert!(score(&orderer, &board, &promotion) > score(&orderer, &board, &quiet));
    }

    #[test]
    fn test_tt_move_outranks_a_winning_capture() {
        let orderer = MoveOrderer::new();
        let board = pos("7k/8/8/3q4/4P3/8/8/K7 w - - 0 1");

        let capture = Move::new(Square::E4, Square::D5, Move::CAPTURE);

        let as_tt = orderer.move_score_with_see(&capture, Some(capture), 0, &board);
        let as_plain = orderer.move_score_with_see(&capture, None, 0, &board);

        assert!(as_tt > as_plain);
    }

    #[test]
    fn test_killer_beats_plain_quiet() {
        let mut orderer = MoveOrderer::new();
        let board = pos("7k/8/8/8/8/2N5/8/K7 w - - 0 1");

        let killer = Move::new(Square::C3, Square::E4, Move::QUIET);
        let other = Move::new(Square::C3, Square::D5, Move::QUIET);
        orderer.store_killer(killer, 0);

        assert!(score(&orderer, &board, &killer) > score(&orderer, &board, &other));
    }
}
