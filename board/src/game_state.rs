use aether_core::{CastlingRights, Color, File, Square};

#[derive(Debug, Clone, PartialEq)]
pub struct GameState {
    pub side_to_move: Color,
    pub castling_rights: [CastlingRights; 2],
    pub en_passant_square: Option<Square>,
    pub halfmove_clock: u16,
    pub fullmove_number: u16,
}

impl GameState {
    pub const fn new() -> Self {
        Self {
            side_to_move: Color::White,
            castling_rights: [CastlingRights::EMPTY; 2],
            en_passant_square: None,
            halfmove_clock: 0,
            fullmove_number: 1,
        }
    }

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

    pub fn switch_side(&mut self) {
        self.side_to_move = self.side_to_move.opponent();
        if self.side_to_move == Color::White {
            self.fullmove_number = self.fullmove_number.saturating_add(1);
        }
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
        assert!(state.castling_rights[Color::White as usize].is_empty());
        assert!(state.castling_rights[Color::Black as usize].is_empty());
    }

    #[test]
    fn test_starting_position() {
        let state = GameState::starting_position();
        assert_eq!(state.side_to_move, Color::White);
        assert!(state.castling_rights[Color::White as usize].short.is_some());
        assert!(state.castling_rights[Color::White as usize].long.is_some());
        assert!(state.castling_rights[Color::Black as usize].short.is_some());
        assert!(state.castling_rights[Color::Black as usize].long.is_some());
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
}
