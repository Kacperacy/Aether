use crate::search::MAX_PLY;
use crate::search::see::{piece_on, see_value};
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
    pub fn order_moves(&self, moves: &mut [Move], side: Color, pieces: &[[BitBoard; 6]; 2]) {
        moves.sort_unstable_by(|a, b| {
            let a_score = self.move_score(a, side, pieces);
            let b_score = self.move_score(b, side, pieces);
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

        let moving_piece = piece_on(mv.from_sq(), &pieces[side as usize]);

        if mv.is_capture() || mv.is_en_passant() {
            let captured_value = if mv.is_en_passant() {
                Piece::PAWN_VALUE
            } else {
                piece_on(mv.to_sq(), &pieces[side.opponent() as usize])
                    .map(Piece::value)
                    .unwrap_or(0)
            };
            let promo_bonus = mv.promotion_piece().map(|p| p.value()).unwrap_or(0);
            let attacker_value = moving_piece.map(Piece::value).unwrap_or(0);
            let mvv_lva = captured_value - attacker_value;

            if mvv_lva >= 0 {
                return GOOD_CAPTURE_SCORE + promo_bonus + 10 * captured_value - attacker_value;
            }

            let see = see_value(mv, side, occupied, pieces);
            return if see >= 0 {
                GOOD_CAPTURE_SCORE + promo_bonus + 10 * captured_value - attacker_value
            } else {
                BAD_CAPTURE_SCORE + promo_bonus + see
            };
        }

        if let Some(promo) = mv.promotion_piece() {
            return 9_000 + promo.value();
        }

        if let Some(killer_score) = self.killer_score(mv, ply) {
            return killer_score;
        }

        if self.is_repetition_move(mv) {
            return REPETITION_PENALTY;
        }

        moving_piece.map(|p| self.history_score(mv, p)).unwrap_or(0)
    }

    #[allow(dead_code)]
    #[inline(always)]
    fn move_score(&self, mv: &Move, side: Color, pieces: &[[BitBoard; 6]; 2]) -> i32 {
        let mut score = 0;

        let moving_value = piece_on(mv.from_sq(), &pieces[side as usize])
            .map(Piece::value)
            .unwrap_or(0);

        if mv.is_capture() || mv.is_en_passant() {
            let captured_value = if mv.is_en_passant() {
                Piece::PAWN_VALUE
            } else {
                piece_on(mv.to_sq(), &pieces[side.opponent() as usize])
                    .map(Piece::value)
                    .unwrap_or(0)
            };
            score += 10 * captured_value - moving_value;
        }

        if let Some(promo) = mv.promotion_piece() {
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
    use aether_core::{Move, Square};

    #[test]
    fn test_move_ordering_captures() {
        let orderer = MoveOrderer::new();

        let mut pieces = [[BitBoard::EMPTY; 6]; 2];
        pieces[Color::White as usize][Piece::Pawn as usize] = Square::E4.bitboard();
        pieces[Color::Black as usize][Piece::Queen as usize] = Square::D5.bitboard();
        pieces[Color::White as usize][Piece::Knight as usize] = Square::F3.bitboard();
        pieces[Color::Black as usize][Piece::Pawn as usize] = Square::E5.bitboard();

        let pawn_takes_queen = Move::new(Square::E4, Square::D5, Move::CAPTURE);
        let knight_takes_pawn = Move::new(Square::F3, Square::E5, Move::CAPTURE);

        let score1 = orderer.move_score(&pawn_takes_queen, Color::White, &pieces);
        let score2 = orderer.move_score(&knight_takes_pawn, Color::White, &pieces);

        assert!(score1 > score2);
    }

    #[test]
    fn test_promotion_scores_high() {
        let orderer = MoveOrderer::new();

        let mut pieces = [[BitBoard::EMPTY; 6]; 2];
        pieces[Color::White as usize][Piece::Pawn as usize] =
            Square::E7.bitboard() | Square::E2.bitboard();

        let promotion = Move::new(Square::E7, Square::E8, Move::PROMO_Q);
        let normal_move = Move::new(Square::E2, Square::E4, Move::DOUBLE_PUSH);

        assert!(
            orderer.move_score(&promotion, Color::White, &pieces)
                > orderer.move_score(&normal_move, Color::White, &pieces)
        );
    }
}
