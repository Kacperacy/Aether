use aether_types::{BitBoard, Color, Piece, Square};
use movegen::{
    magic::{get_bishop_moves, get_rook_moves},
    pieces::{get_king_moves, knight::get_knight_attacks, pawn::get_pawn_attacks_to_square},
};

use crate::Board;

impl Board {
    pub fn is_in_check(&self, color: Color) -> bool {
        let king_square = self.get_king_square(color);

        if king_square.is_none() {
            return false; // No king on the board, cannot be in check
        }

        let king_square = king_square.unwrap();

        self.attackers_to_square(king_square, color.opponent()) != BitBoard::EMPTY
    }

    pub fn attackers_to_square(&self, square: Square, color: Color) -> BitBoard {
        let mut attackers = BitBoard::EMPTY;
        let occupied = self.occupied;

        let attacking_pieces = self.color_combined[color as usize];

        if attacking_pieces.is_empty() {
            return attackers;
        }

        let pawn_attacks = get_pawn_attacks_to_square(square, color);
        attackers |= pawn_attacks & self.pieces[color as usize][Piece::Pawn as usize];

        let knight_attacks = get_knight_attacks(square);
        attackers |= knight_attacks & self.pieces[color as usize][Piece::Knight as usize];

        let bishop_attacks = get_bishop_moves(square, occupied);
        attackers |= bishop_attacks
            & (self.pieces[color as usize][Piece::Bishop as usize]
            | self.pieces[color as usize][Piece::Queen as usize]);

        let rook_attacks = get_rook_moves(square, occupied);
        attackers |= rook_attacks
            & (self.pieces[color as usize][Piece::Rook as usize]
            | self.pieces[color as usize][Piece::Queen as usize]);

        let king_attacks = get_king_moves(square);
        attackers |= king_attacks & self.pieces[color as usize][Piece::King as usize];

        attackers
    }
}
