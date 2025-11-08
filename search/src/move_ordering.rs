use aether_types::{Move, Piece};

/// Trait for move ordering strategies
///
/// Move ordering is critical for alpha-beta search efficiency.
/// Better move ordering leads to more cutoffs and faster search.
pub trait MoveOrderer {
    /// Score a move for ordering purposes
    /// Higher scores should be searched first
    fn score_move(&self, mv: &Move) -> i32;

    /// Order a list of moves in-place (best moves first)
    fn order_moves(&self, moves: &mut [Move]) {
        moves.sort_by_key(|mv| -self.score_move(mv));
    }
}

/// Simple move ordering based on MVV-LVA (Most Valuable Victim - Least Valuable Attacker)
/// and piece type
#[derive(Debug, Clone, Copy)]
pub struct SimpleMoveOrderer {
    /// Bonus for promotions
    pub promotion_bonus: i32,

    /// Bonus for captures (scaled by captured piece value)
    pub capture_bonus_multiplier: i32,
}

impl SimpleMoveOrderer {
    pub fn new() -> Self {
        Self {
            promotion_bonus: 10_000,
            capture_bonus_multiplier: 100,
        }
    }

    /// Get MVV-LVA score for a capture
    /// Higher value for capturing valuable pieces with less valuable attackers
    fn mvv_lva_score(&self, mv: &Move) -> i32 {
        if let Some(captured) = mv.capture {
            let victim_value = captured.value() as i32;
            let attacker_value = mv.piece.value() as i32;

            // MVV-LVA: value of victim * 100 - value of attacker
            // This prioritizes capturing valuable pieces with cheap pieces
            victim_value * self.capture_bonus_multiplier - attacker_value
        } else {
            0
        }
    }
}

impl Default for SimpleMoveOrderer {
    fn default() -> Self {
        Self::new()
    }
}

impl MoveOrderer for SimpleMoveOrderer {
    fn score_move(&self, mv: &Move) -> i32 {
        let mut score = 0;

        // Promotions are very good
        if let Some(promo_piece) = mv.promotion {
            score += self.promotion_bonus;
            // Queen promotions are best
            if promo_piece == Piece::Queen {
                score += 1000;
            }
        }

        // Captures scored by MVV-LVA
        if mv.capture.is_some() {
            score += self.mvv_lva_score(mv);
        }

        // Castling is generally good
        if mv.flags.is_castle {
            score += 50;
        }

        score
    }
}

/// Advanced move ordering with killer moves and history heuristic
#[derive(Debug, Clone)]
pub struct AdvancedMoveOrderer {
    base_orderer: SimpleMoveOrderer,

    /// Killer moves table: [ply][killer_slot]
    /// Stores non-capture moves that caused beta cutoffs
    killer_moves: Vec<Vec<Option<Move>>>,

    /// History heuristic: [piece][to_square]
    /// Tracks how often a move caused a cutoff
    history_table: [[i32; 64]; 6],

    /// Maximum number of killer moves per ply
    max_killers: usize,
}

impl AdvancedMoveOrderer {
    pub fn new() -> Self {
        Self {
            base_orderer: SimpleMoveOrderer::new(),
            killer_moves: vec![vec![None; 2]; 128], // 2 killer moves per ply, up to depth 128
            history_table: [[0; 64]; 6],
            max_killers: 2,
        }
    }

    /// Store a killer move at a given ply
    pub fn store_killer(&mut self, mv: Move, ply: usize) {
        if ply >= self.killer_moves.len() {
            return;
        }

        // Don't store captures as killer moves
        if mv.capture.is_some() {
            return;
        }

        // Check if already stored
        if self.killer_moves[ply].contains(&Some(mv)) {
            return;
        }

        // Shift killers and insert new one at front
        for i in (1..self.max_killers).rev() {
            self.killer_moves[ply][i] = self.killer_moves[ply][i - 1];
        }
        self.killer_moves[ply][0] = Some(mv);
    }

    /// Update history table for a move
    pub fn update_history(&mut self, mv: Move, depth: u8) {
        let piece_idx = mv.piece as usize;
        let to_idx = mv.to as usize;

        // Depth squared gives more weight to moves at higher depths
        let bonus = (depth as i32) * (depth as i32);
        self.history_table[piece_idx][to_idx] += bonus;

        // Cap to prevent overflow
        if self.history_table[piece_idx][to_idx] > 10_000_000 {
            self.age_history();
        }
    }

    /// Age history table (divide all values by 2)
    fn age_history(&mut self) {
        for piece_history in &mut self.history_table {
            for score in piece_history.iter_mut() {
                *score /= 2;
            }
        }
    }

    /// Clear killer moves and history
    pub fn clear(&mut self) {
        self.killer_moves = vec![vec![None; self.max_killers]; 128];
        self.history_table = [[0; 64]; 6];
    }

    /// Check if a move is a killer move at a given ply
    fn is_killer(&self, mv: &Move, ply: usize) -> bool {
        if ply >= self.killer_moves.len() {
            return false;
        }

        self.killer_moves[ply].contains(&Some(*mv))
    }

    /// Get history score for a move
    fn history_score(&self, mv: &Move) -> i32 {
        let piece_idx = mv.piece as usize;
        let to_idx = mv.to as usize;
        self.history_table[piece_idx][to_idx]
    }
}

impl Default for AdvancedMoveOrderer {
    fn default() -> Self {
        Self::new()
    }
}

impl MoveOrderer for AdvancedMoveOrderer {
    fn score_move(&self, mv: &Move) -> i32 {
        let mut score = self.base_orderer.score_move(mv);

        // Killer moves bonus (only for non-captures)
        if mv.capture.is_none() {
            // Check killer moves at ply 0 for now (will need ply parameter in real search)
            if self.is_killer(mv, 0) {
                score += 9000;
            }

            // Add history score
            score += self.history_score(mv) / 100; // Scale down history scores
        }

        score
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use aether_types::{MoveFlags, Square};

    fn make_test_move(from: i8, to: i8, piece: Piece, capture: Option<Piece>) -> Move {
        Move {
            from: Square::from_index(from),
            to: Square::from_index(to),
            piece,
            capture,
            promotion: None,
            flags: MoveFlags {
                is_castle: false,
                is_en_passant: false,
                is_double_pawn_push: false,
            },
        }
    }

    #[test]
    fn test_mvv_lva_ordering() {
        let orderer = SimpleMoveOrderer::new();

        // Pawn takes queen should score higher than queen takes pawn
        let pxq = make_test_move(0, 8, Piece::Pawn, Some(Piece::Queen));
        let qxp = make_test_move(0, 8, Piece::Queen, Some(Piece::Pawn));

        assert!(
            orderer.score_move(&pxq) > orderer.score_move(&qxp),
            "PxQ should score higher than QxP (MVV-LVA)"
        );
    }

    #[test]
    fn test_promotion_ordering() {
        let orderer = SimpleMoveOrderer::new();

        let promotion = make_test_move(0, 8, Piece::Pawn, None);
        let regular = make_test_move(0, 8, Piece::Knight, None);

        let mut promotion_move = promotion;
        promotion_move.promotion = Some(Piece::Queen);

        assert!(
            orderer.score_move(&promotion_move) > orderer.score_move(&regular),
            "Promotions should score higher than regular moves"
        );
    }
}
