use crate::search::MAX_PLY;
use crate::search::see::see_value;
use aether_core::{BitBoard, Color, Move, Piece, Square};

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
        let idx = mv.from.to_index() as usize * 64 + mv.to.to_index() as usize;
        self.repetition_moves[idx] = true;
    }

    #[inline]
    fn is_repetition_move(&self, mv: &Move) -> bool {
        let idx = mv.from.to_index() as usize * 64 + mv.to.to_index() as usize;
        self.repetition_moves[idx]
    }

    pub fn update_history(&mut self, mv: Move, depth: usize) {
        if mv.capture.is_some() {
            return;
        }

        let bonus = depth as i32 * depth as i32;
        let idx = mv.to.to_index() as usize;
        self.history[mv.piece as usize][idx] += bonus;

        if self.history[mv.piece as usize][idx] > 8_000 {
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
    fn history_score(&self, mv: &Move) -> i32 {
        if mv.capture.is_some() {
            return 0;
        }
        self.history[mv.piece as usize][mv.to.to_index() as usize]
    }

    #[inline]
    pub fn store_killer(&mut self, mv: Move, ply: usize) {
        if ply >= MAX_PLY || mv.capture.is_some() || mv.promotion.is_some() {
            return;
        }

        if self.killers[ply][0] != Some(mv) {
            self.killers[ply][1] = self.killers[ply][0];
            self.killers[ply][0] = Some(mv);
        }
    }

    #[allow(dead_code)]
    #[inline]
    pub fn is_killer(&self, mv: &Move, ply: usize) -> bool {
        if ply >= MAX_PLY {
            return false;
        }
        self.killers[ply][0] == Some(*mv) || self.killers[ply][1] == Some(*mv)
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

    #[allow(dead_code)]
    pub fn order_moves(&self, moves: &mut [Move]) {
        moves.sort_unstable_by(|a, b| {
            let a_score = self.move_score(a);
            let b_score = self.move_score(b);
            b_score.cmp(&a_score)
        });
    }

    pub fn order_moves_with_see(
        &self,
        moves: &mut [Move],
        tt_move: Option<Move>,
        ply: usize,
        side: Color,
        occupied: BitBoard,
        pieces: &[[BitBoard; 6]; 2],
    ) {
        moves.sort_by_cached_key(|mv| {
            std::cmp::Reverse(self.move_score_with_see(mv, tt_move, ply, side, occupied, pieces))
        });
    }

    #[inline(always)]
    fn move_score_with_see(
        &self,
        mv: &Move,
        tt_move: Option<Move>,
        ply: usize,
        side: Color,
        occupied: BitBoard,
        pieces: &[[BitBoard; 6]; 2],
    ) -> i32 {
        if Some(*mv) == tt_move {
            return 20_000;
        }

        if let Some(captured) = mv.capture {
            let promo_bonus = mv.promotion.map(|p| p.value()).unwrap_or(0);
            let mvv_lva = captured.value() - mv.piece.value();

            if mvv_lva >= 0 {
                return GOOD_CAPTURE_SCORE + promo_bonus + 10 * captured.value() - mv.piece.value();
            }

            let see = see_value(mv, side, occupied, pieces);
            return if see >= 0 {
                GOOD_CAPTURE_SCORE + promo_bonus + 10 * captured.value() - mv.piece.value()
            } else {
                BAD_CAPTURE_SCORE + promo_bonus + see
            };
        }

        if let Some(promo) = mv.promotion {
            return 9_000 + promo.value();
        }

        if let Some(killer_score) = self.killer_score(mv, ply) {
            return killer_score;
        }

        if self.is_repetition_move(mv) {
            return REPETITION_PENALTY;
        }

        self.history_score(mv)
    }

    #[allow(dead_code)]
    #[inline(always)]
    fn move_score(&self, mv: &Move) -> i32 {
        let mut score = 0;

        if let Some(captured) = mv.capture {
            score += 10 * captured.value() - mv.piece.value();
        }

        if let Some(promo) = mv.promotion {
            score += 100 + promo.value();
        }

        score
    }
}

impl Default for MoveOrderer {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use aether_core::{Move, MoveFlags, Piece, Square};

    #[test]
    fn test_move_ordering_captures() {
        let orderer = MoveOrderer::new();

        let pawn_takes_queen = Move {
            from: Square::E4,
            to: Square::D5,
            piece: Piece::Pawn,
            capture: Some(Piece::Queen),
            promotion: None,
            flags: MoveFlags::default(),
        };

        let knight_takes_pawn = Move {
            from: Square::F3,
            to: Square::E5,
            piece: Piece::Knight,
            capture: Some(Piece::Pawn),
            promotion: None,
            flags: MoveFlags::default(),
        };

        let score1 = orderer.move_score(&pawn_takes_queen);
        let score2 = orderer.move_score(&knight_takes_pawn);

        assert!(score1 > score2);
    }

    #[test]
    fn test_promotion_scores_high() {
        let orderer = MoveOrderer::new();

        let promotion = Move {
            from: Square::E7,
            to: Square::E8,
            piece: Piece::Pawn,
            capture: None,
            promotion: Some(Piece::Queen),
            flags: MoveFlags::default(),
        };

        let normal_move = Move {
            from: Square::E2,
            to: Square::E4,
            piece: Piece::Pawn,
            capture: None,
            promotion: None,
            flags: MoveFlags {
                is_castle: false,
                is_en_passant: false,
                is_double_pawn_push: true,
            },
        };

        assert!(orderer.move_score(&promotion) > orderer.move_score(&normal_move));
    }
}
