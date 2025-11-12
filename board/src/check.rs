use aether_types::{BitBoard, Color, Square};

use crate::Board;
use crate::query::BoardQuery;

impl Board {
    /// Returns true if the given color is in check
    pub fn is_in_check(&self, color: Color) -> bool {
        let Some(king_sq) = self.get_king_square(color) else {
            return false;
        };

        !self
            .attackers_to_square(king_sq, color.opponent())
            .is_empty()
    }

    /// Returns a bitboard of pieces (of `color`) that attack `sq`.
    pub fn attackers_to_square(&self, sq: Square, color: Color) -> BitBoard {
        let occ = self.cache.occupied;
        let their = &self.pieces[color as usize];
        // attackers_to_square_with_occ(sq, color, occ, their)
        todo!()
    }
}
