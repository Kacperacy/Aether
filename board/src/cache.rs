use aether_core::{ALL_COLORS, BitBoard, Color, combine_piece_bitboards};

#[derive(Debug, Clone, PartialEq)]
pub struct BoardCache {
    pub color_combined: [BitBoard; 2],
    pub occupied: BitBoard,
}

impl BoardCache {
    /// Creates a new, empty BoardCache
    pub fn new() -> Self {
        Self {
            color_combined: [BitBoard::EMPTY; 2],
            occupied: BitBoard::EMPTY,
        }
    }

    /// Fully refreshes the cache based on the provided piece bitboards
    pub fn refresh(&mut self, pieces: &[[BitBoard; 6]; 2]) {
        for color in ALL_COLORS {
            self.color_combined[color as usize] = pieces[color as usize][0] // Pawn
                | pieces[color as usize][1] // Knight
                | pieces[color as usize][2] // Bishop
                | pieces[color as usize][3] // Rook
                | pieces[color as usize][4] // Queen
                | pieces[color as usize][5]; // King
        }
        self.occupied = self.color_combined[0] | self.color_combined[1];
    }
}

impl Default for BoardCache {
    /// Creates a default BoardCache (empty)
    fn default() -> Self {
        Self::new()
    }
}
