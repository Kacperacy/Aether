//! Check detection for chess positions.
//!
//! This module implements check detection and attacker computation
//! using attack tables from the movegen crate.

use aether_types::{BitBoard, BoardQuery, Color, Square};
use movegen::attacks::attackers_to_square_with_occ;

use crate::Board;

impl Board {
    /// Returns whether the specified color's king is in check.
    pub fn is_in_check(&self, color: Color) -> bool {
        let Some(king_sq) = self.get_king_square(color) else {
            return false;
        };

        !self
            .attackers_to_square(king_sq, color.opponent())
            .is_empty()
    }

    /// Returns a bitboard of all pieces of the given color attacking the square.
    pub fn attackers_to_square(&self, sq: Square, color: Color) -> BitBoard {
        let occ = self.cache.occupied;
        let their = &self.pieces[color as usize];
        attackers_to_square_with_occ(sq, color, occ, their)
    }
}
