use aether_core::Move;

pub struct MoveOrderer;

impl MoveOrderer {
    pub fn new() -> Self {
        Self
    }

    pub fn order_moves(&self, moves: &mut [Move]) {
        moves.sort_by(|a, b| {
            let a_score = self.move_score(a);
            let b_score = self.move_score(b);
            b_score.cmp(&a_score)
        });
    }

    fn move_score(&self, mv: &Move) -> i32 {
        let mut score = 0;

        // Captures: MVV-LVA (Most Valuable Victim - Least Valuable Attacker)
        if let Some(captured) = mv.capture {
            score += 10 * captured.value() as i32;
            score -= mv.piece.value() as i32;
        }

        // Promotions
        if let Some(promo) = mv.promotion {
            score += 100 + promo.value() as i32;
        }

        // TODO:
        // - Killer moves
        // - History heuristic
        // - Counter moves
        // - PV moves

        score
    }

    pub fn order_moves_with_tt(&self, moves: &mut [Move], tt_move: Option<Move>) {
        self.order_moves(moves);

        if let Some(tt_mv) = tt_move {
            if let Some(pos) = moves.iter().position(|&m| m == tt_mv) {
                moves.swap(0, pos);
            }
        }
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
