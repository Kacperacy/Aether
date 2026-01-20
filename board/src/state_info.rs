use aether_core::{BitBoard, CastlingRights, Color, Piece, Square};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct StateInfo {
    pub castling_rights: [CastlingRights; 2],
    pub en_passant_square: Option<Square>,
    pub halfmove_clock: u16,
    pub captured_piece: Option<(Piece, Color)>,

    pub zobrist_hash: u64,
    pub game_phase: i16,
    pub pst_mg: i32,
    pub pst_eg: i32,

    pub king_square: [Square; 2],
    pub checkers: BitBoard,
    pub blockers_for_king: [BitBoard; 2],
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
