use crate::query::BoardQuery;
use crate::{Board, Result};
use aether_core::{
    ALL_COLORS, ALL_PIECES, BitBoard, Color, File, Move, MoveState, Piece, Rank, Square,
};

/// Trait for board operations
pub trait BoardOps: BoardQuery + Clone {
    /// Make a move on the board, updating the position accordingly.
    fn make_move(&mut self, mv: &Move) -> Result<()>;

    /// Unmake a move on the board, restoring the previous position.
    fn unmake_move(&mut self, mv: &Move) -> Result<()>;

    /// Is the position in check for the side to move?
    fn is_in_check(&self, color: Color) -> bool;
}

impl BoardOps for Board {
    fn make_move(&mut self, mv: &Move) -> Result<()> {}

    fn unmake_move(&mut self, mv: &Move) -> Result<()> {}

    fn is_in_check(&self, color: Color) -> bool {
        let king_sq = match self.get_king_square(color) {
            Some(sq) => sq,
            None => return false, // No king found, cannot be in check
        };

        self.is_square_attacked(king_sq, color.opponent())
    }
}

impl Board {
    /// Internal method to place a piece on the board without updating game state or cache.
    #[inline(always)]
    pub(crate) fn place_piece_internal(&mut self, square: Square, piece: Piece, color: Color) {
        let bb = BitBoard::from_square(square);
        self.pieces[color as usize][piece as usize] |= bb;
    }

    #[inline(always)]
    pub(crate) fn remove_piece_internal(&mut self, square: Square) -> Option<(Piece, Color)> {
        let bb = BitBoard::from_square(square);

        for color in ALL_COLORS {
            for (piece_idx, piece_bb) in self.pieces[color as usize].iter_mut().enumerate() {
                if piece_bb.has(square) {
                    *piece_bb &= !bb;
                    return Some((ALL_PIECES[piece_idx], color));
                }
            }
        }

        None
    }

    fn update_castling_rights_after_move(&mut self, mv: &Move) {
        if mv.piece == Piece::King {
            self.game_state.castling_rights[self.game_state.side_to_move as usize].short = None;
        }

        if mv.piece == Piece::Rook {
            let rank = match self.game_state.side_to_move {
                Color::White => Rank::One,
                Color::Black => Rank::Eight,
            };

            if mv.from == Square::new(File::H, rank) {
                self.game_state.castling_rights[self.game_state.side_to_move as usize].short = None;
            } else if mv.from == Square::new(File::A, rank) {
                self.game_state.castling_rights[self.game_state.side_to_move as usize].long = None;
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{FenOps, STARTING_POSITION_FEN};
    use aether_core::Square;

    #[test]
    fn test_is_in_check_starting_position() {
        let mut board = Board::from_fen(STARTING_POSITION_FEN).unwrap();

        assert!(!board.is_in_check(Color::White));
        assert!(!board.is_in_check(Color::Black));
    }

    #[test]
    fn test_is_in_check_white_checked_by_rook() {
        let mut board = Board::from_fen("3kr3/8/8/8/8/8/8/4K3 w - - 0 1").unwrap();

        assert!(board.is_in_check(Color::White));
        assert!(!board.is_in_check(Color::Black));
    }

    #[test]
    fn test_is_in_check_black_checked_by_queen() {
        let mut board = Board::from_fen("4k3/8/8/8/8/8/8/4QK2 b - - 0 1").unwrap();

        assert!(!board.is_in_check(Color::White));
        assert!(board.is_in_check(Color::Black));
    }

    #[test]
    fn test_is_in_check_blocked_attack() {
        let mut board = Board::from_fen("3kr3/8/8/8/4P3/8/8/4K3 w - - 0 1").unwrap();

        assert!(!board.is_in_check(Color::White));
    }

    #[test]
    fn test_attackers_to_square_multiple() {
        let mut board = Board::from_fen("7k/8/8/8/4p3/8/3N4/1B5K w - - 0 1").unwrap();

        let attackers = board.attackers_to_square(Square::E4, Color::White);

        assert_eq!(attackers.count(), 2);
        assert!(attackers.has(Square::D2));
        assert!(attackers.has(Square::B1));
    }

    #[test]
    fn test_attackers_to_square_none() {
        let mut board = Board::from_fen("4k3/8/8/8/8/8/8/4K3 w - - 0 1").unwrap();

        let white_attackers = board.attackers_to_square(Square::E4, Color::White);
        let black_attackers = board.attackers_to_square(Square::E4, Color::Black);

        assert!(white_attackers.is_empty());
        assert!(black_attackers.is_empty());
    }

    #[test]
    fn test_is_in_check_knight_check() {
        let mut board = Board::from_fen("4k3/8/3N4/8/8/8/8/4K3 b - - 0 1").unwrap();

        assert!(board.is_in_check(Color::Black));
        assert!(!board.is_in_check(Color::White));
    }

    #[test]
    fn test_is_in_check_pawn_check() {
        let mut board = Board::from_fen("8/8/8/4k3/3P1P2/8/8/4K3 b - - 0 1").unwrap();

        assert!(board.is_in_check(Color::Black));
    }
}
