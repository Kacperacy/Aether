use aether_core::{CastlingRights, Color, File, Square};

/// Represents the state of a chess game
#[derive(Debug, Clone, PartialEq)]
pub struct GameState {
    /// The side to move
    pub side_to_move: Color,

    /// Castling rights for both colors [White, Black]
    pub castling_rights: [CastlingRights; 2],

    /// En passant target square, if any
    pub en_passant_square: Option<Square>,

    /// Halfmove clock for fifty-move rule
    pub halfmove_clock: u16,

    /// Fullmove number (starts at 1, increments after Black's move)
    pub fullmove_number: u16,
}

impl GameState {
    /// Creates a new GameState with default values
    pub const fn new() -> Self {
        Self {
            side_to_move: Color::White,
            castling_rights: [CastlingRights::EMPTY; 2],
            en_passant_square: None,
            halfmove_clock: 0,
            fullmove_number: 1,
        }
    }

    /// Creates a GameState representing the standard starting position
    pub const fn starting_position() -> Self {
        Self {
            side_to_move: Color::White,
            castling_rights: [
                CastlingRights {
                    short: Some(File::H),
                    long: Some(File::A),
                },
                CastlingRights {
                    short: Some(File::H),
                    long: Some(File::A),
                },
            ],
            en_passant_square: None,
            halfmove_clock: 0,
            fullmove_number: 1,
        }
    }

    /// Switches the side to move and updates the fullmove number if needed
    pub fn switch_side(&mut self) {
        self.side_to_move = self.side_to_move.opponent();
        if self.side_to_move == Color::White {
            self.fullmove_number = self.fullmove_number.saturating_add(1);
        }
    }

    /// Returns true if the given color can castle kingside
    #[inline]
    pub fn can_castle_short(&self, color: Color) -> bool {
        self.castling_rights[color as usize].short.is_some()
    }

    /// Returns true if the given color can castle queenside
    #[inline]
    pub fn can_castle_long(&self, color: Color) -> bool {
        self.castling_rights[color as usize].long.is_some()
    }

    /// Returns true if the given color has any castling rights
    #[inline]
    pub fn can_castle(&self, color: Color) -> bool {
        !self.castling_rights[color as usize].is_empty()
    }

    /// Removes all castling rights for the given color
    #[inline]
    pub fn remove_castling_rights(&mut self, color: Color) {
        self.castling_rights[color as usize] = CastlingRights::EMPTY;
    }

    /// Removes kingside castling right for the given color
    #[inline]
    pub fn remove_short_castling(&mut self, color: Color) {
        self.castling_rights[color as usize].short = None;
    }

    /// Removes queenside castling right for the given color
    #[inline]
    pub fn remove_long_castling(&mut self, color: Color) {
        self.castling_rights[color as usize].long = None;
    }
}

impl Default for GameState {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_game_state() {
        let state = GameState::new();
        assert_eq!(state.side_to_move, Color::White);
        assert!(state.en_passant_square.is_none());
        assert_eq!(state.halfmove_clock, 0);
        assert_eq!(state.fullmove_number, 1);
        assert!(!state.can_castle(Color::White));
        assert!(!state.can_castle(Color::Black));
    }

    #[test]
    fn test_starting_position() {
        let state = GameState::starting_position();
        assert_eq!(state.side_to_move, Color::White);
        assert!(state.can_castle_short(Color::White));
        assert!(state.can_castle_long(Color::White));
        assert!(state.can_castle_short(Color::Black));
        assert!(state.can_castle_long(Color::Black));
    }

    #[test]
    fn test_switch_side() {
        let mut state = GameState::starting_position();

        state.switch_side();
        assert_eq!(state.side_to_move, Color::Black);
        assert_eq!(state.fullmove_number, 1);

        state.switch_side();
        assert_eq!(state.side_to_move, Color::White);
        assert_eq!(state.fullmove_number, 2);
    }

    #[test]
    fn test_remove_castling_rights() {
        let mut state = GameState::starting_position();

        state.remove_short_castling(Color::White);
        assert!(!state.can_castle_short(Color::White));
        assert!(state.can_castle_long(Color::White));

        state.remove_castling_rights(Color::Black);
        assert!(!state.can_castle(Color::Black));
    }
}
