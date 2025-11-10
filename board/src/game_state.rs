//! Game state management.
//!
//! This module defines the `GameState` struct which tracks non-positional
//! information required for a chess game: side to move, castling rights,
//! en passant square, halfmove clock, and fullmove number.

use aether_types::{CastlingRights, Color, Square};

/// Represents the non-positional state of a chess game.
///
/// This includes information required by chess rules that isn't captured
/// by piece positions alone: whose turn it is, castling availability,
/// en passant targets, and move counters.
#[derive(Debug, Clone, PartialEq)]
pub struct GameState {
    pub side_to_move: Color,
    pub castling_rights: [CastlingRights; 2],
    pub en_passant_square: Option<Square>,
    pub halfmove_clock: u16,
    pub fullmove_number: u16,
}

impl GameState {
    /// Creates a new empty game state.
    ///
    /// Side to move is White, no castling rights, no en passant, counters at 0/1.
    pub fn new() -> Self {
        Self {
            side_to_move: Color::White,
            castling_rights: [CastlingRights::EMPTY; 2],
            en_passant_square: None,
            halfmove_clock: 0,
            fullmove_number: 1,
        }
    }

    /// Creates game state for the standard chess starting position.
    ///
    /// Both sides have full castling rights (kingside and queenside).
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

    /// Switches the side to move.
    ///
    /// If switching from Black to White, increments the fullmove number.
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
