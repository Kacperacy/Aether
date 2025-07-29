use aether_types::{CastlingRights, Color, Square};

#[derive(Debug, Clone, PartialEq)]
pub struct GameState {
    pub side_to_move: Color,
    pub castling_rights: [CastlingRights; 2],
    pub en_passant_square: Option<Square>,
    pub halfmove_clock: u16,
    pub fullmove_number: u16,
}

impl GameState {
    pub fn new() -> Self {
        Self {
            side_to_move: Color::White,
            castling_rights: [CastlingRights::EMPTY; 2],
            en_passant_square: None,
            halfmove_clock: 0,
            fullmove_number: 1,
        }
    }

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

    pub fn switch_side(&mut self) {
        self.side_to_move = self.side_to_move.opponent();
        if self.side_to_move == Color::White {
            self.fullmove_number += 1;
        }
    }
}

impl Default for GameState {
    fn default() -> Self {
        Self::new()
    }
}
