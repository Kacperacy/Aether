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

    pub fn make_move(&mut self, from: Square, to: Square) -> Option<(Piece, Color)> {
        let piece_info = self.remove_piece(from);
        if let Some((piece, color)) = piece_info {
            if self.piece_at(to).is_some() {
                self.remove_piece(to);
            }
            self.place_piece(to, piece, color);
            return Some((piece, color));
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
