use aether_types::{BitBoard, BoardQuery, Color, Piece, Square};
use movegen::{
    magic::{get_bishop_attacks, get_rook_attacks},
    pieces::{get_king_moves, knight::get_knight_attacks, pawn::get_pawn_attacks_to_square},
};

use crate::Board;

impl Board {
    pub fn is_in_check(&self, color: Color) -> bool {
        let Some(king_sq) = self.get_king_square(color) else {
            return false;
        };

        !self
            .attackers_to_square(king_sq, color.opponent())
            .is_empty()
    }

    pub fn attackers_to_square(&self, sq: Square, color: Color) -> BitBoard {
        let occ = self.cache.occupied;
        let their = &self.pieces[color as usize];

        [
            get_pawn_attacks_to_square(sq, color) & their[Piece::Pawn as usize],
            get_knight_attacks(sq) & their[Piece::Knight as usize],
            get_bishop_attacks(sq, occ)
                & (their[Piece::Bishop as usize] | their[Piece::Queen as usize]),
            get_rook_attacks(sq, occ)
                & (their[Piece::Rook as usize] | their[Piece::Queen as usize]),
            get_king_moves(sq) & their[Piece::King as usize],
        ]
        .into_iter()
        .fold(BitBoard::new(), |acc, bb| acc | bb)
    }
}
