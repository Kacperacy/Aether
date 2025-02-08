use crate::board::{Board, Color, Piece};
use crate::constants::*;

pub struct PieceAt {
    pub piece: Piece,
    pub color: Color,
}

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
        self.colors[Color::White as usize]
            .and(&self.colors[Color::Black as usize])
            .is_set(index)
    }

    pub fn is_index_in_bounds(index: i32) -> bool {
        index >= 0 && index < BOARD_SIZE as i32
    }

    pub fn is_enemy(&self, index: usize) -> bool {
        self.colors[self.turn.opposite() as usize].is_set(index)
    }

    pub fn piece_at(&self, index: usize) -> Option<PieceAt> {
        for color in [Color::White, Color::Black].iter() {
            if !self.colors[*color as usize].is_set(index) {
                continue;
            }
            for piece in [
                Piece::Pawn,
                Piece::Knight,
                Piece::Bishop,
                Piece::Rook,
                Piece::Queen,
                Piece::King,
            ]
            .iter()
            {
                if self.pieces[*piece as usize].is_set(index) {
                    return Some(PieceAt {
                        piece: *piece,
                        color: *color,
                    });
                }
            }
        }

        None
    }
}
