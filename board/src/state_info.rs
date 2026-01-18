use aether_core::{BitBoard, CastlingRights, Color, Piece, Square};

/// Stores all irreversible game state that must be preserved for unmake_move.
/// This follows the Stockfish pattern where StateInfo contains both the current
/// game state and serves as history entries.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct StateInfo {
    /// Castling availability for both sides
    pub castling_rights: [CastlingRights; 2],
    /// Valid en passant target square (if any)
    pub en_passant_square: Option<Square>,
    /// Halfmove clock for 50-move rule (resets on pawn move or capture)
    pub halfmove_clock: u16,
    /// Piece captured on this move (for unmake)
    pub captured_piece: Option<(Piece, Color)>,

    /// Zobrist hash of the position
    pub zobrist_hash: u64,
    /// Game phase for tapered evaluation (0 = endgame, 256 = opening)
    pub game_phase: i16,
    /// Piece-square table middlegame score
    pub pst_mg: i32,
    /// Piece-square table endgame score
    pub pst_eg: i32,

    /// Cached king squares for both sides [White, Black]
    pub king_square: [Square; 2],
    /// Pieces giving check to the side to move (computed after each move)
    pub checkers: BitBoard,
    /// Pieces blocking slider attacks on our king [White, Black]
    pub blockers_for_king: [BitBoard; 2],
    /// Enemy pieces pinning our pieces to our king [White, Black]
    pub pinners: [BitBoard; 2],
}

impl StateInfo {
    pub const fn new() -> Self {
        Self {
            castling_rights: [CastlingRights::EMPTY; 2],
            en_passant_square: None,
            halfmove_clock: 0,
            captured_piece: None,
            zobrist_hash: 0,
            game_phase: 0,
            pst_mg: 0,
            pst_eg: 0,
            king_square: [Square::E1, Square::E8],
            checkers: BitBoard::EMPTY,
            blockers_for_king: [BitBoard::EMPTY; 2],
            pinners: [BitBoard::EMPTY; 2],
        }
    }
}

impl Default for StateInfo {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_state_info() {
        let state = StateInfo::new();
        assert!(state.en_passant_square.is_none());
        assert_eq!(state.halfmove_clock, 0);
        assert!(state.captured_piece.is_none());
        assert_eq!(state.zobrist_hash, 0);
        assert!(state.castling_rights[Color::White as usize].is_empty());
        assert!(state.castling_rights[Color::Black as usize].is_empty());
    }
}
