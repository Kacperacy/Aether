use crate::Board;
use aether_types::{BitBoard, Color, Piece, Square};

impl Board {
    pub fn place_piece(&mut self, square: Square, piece: Piece, color: Color) {
        self.pieces[piece as usize] |= BitBoard::from_square(square);
        self.colors[color as usize] |= BitBoard::from_square(square);
    }

    pub fn remove_piece(&mut self, square: Square) -> Option<(Piece, Color)> {
        let index = square.to_index();
        for piece in Piece::all() {
            if self.pieces[piece as usize].is_set_index(index) {
                let color = if self.colors[Color::White as usize].is_set_index(index) {
                    Color::White
                } else {
                    Color::Black
                };
                self.pieces[piece as usize] -= BitBoard::from_square(square);
                self.colors[color as usize] -= BitBoard::from_square(square);
                return Some((piece, color));
            }
        }
        None
    }

    pub fn piece_at(&self, square: Square) -> Option<(Piece, Color)> {
        let index = square.to_index();
        for piece in Piece::all() {
            if self.pieces[piece as usize].is_set_index(index) {
                let color = if self.colors[Color::White as usize].is_set_index(index) {
                    Color::White
                } else {
                    Color::Black
                };
                return Some((piece, color));
            }
        }
        None
    }
}
