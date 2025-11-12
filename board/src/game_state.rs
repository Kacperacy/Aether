use aether_types::{CastlingRights, Color, Square};

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

    /// Fullmove number (starts at 1, increments after Black
    pub fullmove_number: u16,
}

impl GameState {
    /// Creates a new GameState with default values
    pub fn new() -> Self {
        Self {
            side_to_move: Color::White,
            castling_rights: [CastlingRights::EMPTY; 2],
            en_passant_square: None,
            halfmove_clock: 0,
            fullmove_number: 1,
        }
    }

    /// Creates a GameState representing the standard starting position
    pub fn starting_position() -> Self {
        use aether_types::File;
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
            self.fullmove_number += 1;
        }
    }
}

impl Default for GameState {
    /// Creates a default GameState
    fn default() -> Self {
        Self::new()
    }
}
