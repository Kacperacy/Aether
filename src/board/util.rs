use crate::board::{Board, Color, Piece};
use crate::constants::*;

impl Board {
    pub fn square_to_index(square: &str) -> usize {
        let col = square.chars().nth(0).unwrap() as usize - 'a' as usize;
        let row = square.chars().nth(1).unwrap() as usize - '1' as usize;
        row * BOARD_WIDTH + col
    }

    pub fn index_to_square(index: usize) -> String {
        let col = (index % BOARD_WIDTH) as u8 + b'a';
        let row = (index / BOARD_WIDTH) as u8 + b'1';
        format!("{}{}", col as char, row as char)
    }

    pub fn is_square_empty(&self, index: usize) -> bool {
        !self.white_occupancy.is_set(index) && !self.black_occupancy.is_set(index)
    }

    pub fn is_index_in_bounds(index: i32) -> bool {
        index >= 0 && index < BOARD_SIZE as i32
    }

    pub fn is_enemy(&self, index: usize) -> bool {
        match self.turn {
            Color::White => self.black_occupancy.is_set(index),
            Color::Black => self.white_occupancy.is_set(index),
        }
    }

    pub fn piece_at(&self, index: usize) -> Option<Piece> {
        if self.white_occupancy.is_set(index) {
            if self.white_pieces.pawns.is_set(index) {
                return Some(Piece::Pawn);
            } else if self.white_pieces.knights.is_set(index) {
                return Some(Piece::Knight);
            } else if self.white_pieces.bishops.is_set(index) {
                return Some(Piece::Bishop);
            } else if self.white_pieces.rooks.is_set(index) {
                return Some(Piece::Rook);
            } else if self.white_pieces.queens.is_set(index) {
                return Some(Piece::Queen);
            } else if self.white_pieces.king.is_set(index) {
                return Some(Piece::King);
            }
        } else if self.black_occupancy.is_set(index) {
            if self.black_pieces.pawns.is_set(index) {
                return Some(Piece::Pawn);
            } else if self.black_pieces.knights.is_set(index) {
                return Some(Piece::Knight);
            } else if self.black_pieces.bishops.is_set(index) {
                return Some(Piece::Bishop);
            } else if self.black_pieces.rooks.is_set(index) {
                return Some(Piece::Rook);
            } else if self.black_pieces.queens.is_set(index) {
                return Some(Piece::Queen);
            } else if self.black_pieces.king.is_set(index) {
                return Some(Piece::King);
            }
        }
        None
    }
}
