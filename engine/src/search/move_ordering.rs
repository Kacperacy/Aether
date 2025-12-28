use crate::search::MAX_PLY;
use aether_core::{Move, Piece, Square};

/// Penalty applied to moves that would lead to a repeated position
const REPETITION_PENALTY: i32 = -5000;

pub struct MoveOrderer {
    killers: [[Option<Move>; 2]; MAX_PLY],
    history: [[i32; 64]; 6],
    /// Tracks moves that led to repetitions (indexed by from-to square pair)
    /// Uses a simple hash: from * 64 + to
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

    /// Clears only the repetition moves tracking (call at start of each search)
    pub fn clear_repetitions(&mut self) {
        self.repetition_moves = [false; 64 * 64];
    }

    /// Marks a move as leading to a repeated position
    #[inline]
    pub fn mark_repetition_move(&mut self, mv: &Move) {
        let idx = mv.from.to_index() as usize * 64 + mv.to.to_index() as usize;
        self.repetition_moves[idx] = true;
    }

    /// Checks if a move was previously found to lead to a repetition
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

        let idx = mv.to.to_index() as usize;
        self.history[mv.piece as usize][idx]
    }

    #[inline]
    pub fn store_killer(&mut self, mv: Move, ply: usize) {
        if ply >= MAX_PLY || mv.capture.is_some() {
            return;
        }

        if self.killers[ply][0] != Some(mv) {
            self.killers[ply][1] = self.killers[ply][0];
            self.killers[ply][0] = Some(mv);
        }
    }

    #[inline]
    pub fn is_killer(&self, mv: &Move, ply: usize) -> bool {
        if ply >= MAX_PLY {
            return false;
        }

        self.killers[ply][0] == Some(*mv) || self.killers[ply][1] == Some(*mv)
    }

    pub fn order_moves(&self, moves: &mut [Move]) {
        moves.sort_unstable_by(|a, b| {
            let a_score = self.move_score(a);
            let b_score = self.move_score(b);
            b_score.cmp(&a_score)
        });
    }

    pub fn order_moves_full(&self, moves: &mut [Move], tt_move: Option<Move>, ply: usize) {
        moves.sort_unstable_by(|a, b| {
            let a_score = self.move_score_full(a, tt_move, ply);
            let b_score = self.move_score_full(b, tt_move, ply);
            b_score.cmp(&a_score)
        });
    }

    #[inline(always)]
    fn move_score(&self, mv: &Move) -> i32 {
        let mut score = 0;

        // Captures: MVV-LVA (Most Valuable Victim - Least Valuable Attacker)
        if let Some(captured) = mv.capture {
            score += 10 * captured.value() as i32 - mv.piece.value() as i32;
        }

        // Promotions
        if let Some(promo) = mv.promotion {
            score += 100 + promo.value() as i32;
        }

        score
    }

    #[inline(always)]
    fn move_score_full(&self, mv: &Move, tt_move: Option<Move>, ply: usize) -> i32 {
        // Highest priority for TT move
        if Some(*mv) == tt_move {
            return 20_000;
        }

        // Captures: MVV-LVA
        if let Some(captured) = mv.capture {
            return 10_000 + 10 * captured.value() as i32 - mv.piece.value() as i32;
        }

        // Promotions
        if let Some(promo) = mv.promotion {
            return 9_000 + promo.value() as i32;
        }

        if self.is_killer(mv, ply) {
            return 8_000;
        }

        // Penalize moves that previously led to repetitions
        if self.is_repetition_move(mv) {
            return REPETITION_PENALTY;
        }

        self.history_score(mv)
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

        // Create moves: pawn takes queen vs knight takes pawn
        let pawn_takes_queen = Move {
            from: Square::E4,
            to: Square::D5,
            piece: Piece::Pawn,
            capture: Some(Piece::Queen),
            promotion: None,
            flags: MoveFlags {
                is_castle: false,
                is_en_passant: false,
                is_double_pawn_push: false,
            },
        };

        let knight_takes_pawn = Move {
            from: Square::F3,
            to: Square::E5,
            piece: Piece::Knight,
            capture: Some(Piece::Pawn),
            promotion: None,
            flags: MoveFlags {
                is_castle: false,
                is_en_passant: false,
                is_double_pawn_push: false,
            },
        };

        // Pawn takes queen should score higher
        let score1 = orderer.move_score(&pawn_takes_queen);
        let score2 = orderer.move_score(&knight_takes_pawn);

        assert!(
            score1 > score2,
            "Pawn takes queen should score higher than knight takes pawn"
        );
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
            flags: MoveFlags {
                is_castle: false,
                is_en_passant: false,
                is_double_pawn_push: false,
            },
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
