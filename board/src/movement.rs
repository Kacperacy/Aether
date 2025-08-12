use crate::Board;
use aether_types::{BitBoard, Color, Piece, Square};

impl Board {
    pub fn place_piece(&mut self, square: Square, piece: Piece, color: Color) {
        let bb = BitBoard::from_square(square);
        self.pieces[color as usize][piece as usize] |= bb;

        // Update combined bitboards
        self.cache.color_combined[color as usize] |= bb;
        self.cache.occupied |= bb;
    }

    pub fn remove_piece(&mut self, square: Square) -> Option<(Piece, Color)> {
        let bb = BitBoard::from_square(square);
        if !self.cache.occupied.has(square) {
            return None;
        }
        // Determine color using combined occupancy
        let color = if self.cache.color_combined[Color::White as usize].has(square) {
            Color::White
        } else {
            Color::Black
        };
        // Find piece for that color
        for piece in Piece::all() {
            if self.pieces[color as usize][piece as usize].has(square) {
                // Clear piece and update caches
                self.pieces[color as usize][piece as usize] &= !bb;
                self.cache.color_combined[color as usize] &= !bb;
                self.cache.occupied &= !bb;
                return Some((piece, color));
            }
        }
        None
    }

    pub fn make_move(&mut self, from: Square, to: Square) -> Option<(Piece, Color)> {
        if let Some((piece, color)) = self.remove_piece(from) {
            // Capture if any piece on destination
            let _ = self.remove_piece(to);
            self.place_piece(to, piece, color);
            self.change_side_to_move();
            return Some((piece, color));
        }
        None
    }
}
