use crate::error::MoveError;
use crate::query::BoardQuery;
use crate::{Board, BoardError, MAX_SEARCH_DEPTH, Result};
use aether_core::{BitBoard, CastlingRights, Color, File, Move, MoveState, Piece, Square};

/// Trait for board operations
pub trait BoardOps: BoardQuery + Clone {
    /// Make a move on the board, updating the position accordingly
    fn make_move(&mut self, mv: &Move) -> Result<()>;

    /// Unmake a move on the board, restoring the previous position
    fn unmake_move(&mut self, mv: &Move) -> Result<()>;
    ///  Make a null move (pass the turn without moving any piece)
    fn make_null_move(&mut self);
    /// Unmake a null move
    fn unmake_null_move(&mut self);

    /// Is the position in check for the given color
    fn is_in_check(&self, color: Color) -> bool;
}

impl BoardOps for Board {
    fn make_move(&mut self, mv: &Move) -> Result<()> {
        let side = self.game_state.side_to_move;
        let opponent = side.opponent();

        debug_assert!(
            self.history_count < MAX_SEARCH_DEPTH,
            "Exceeded maximum search depth"
        );

        // 1. Save state for unmake
        self.move_history[self.history_count] = MoveState {
            captured_piece: mv.capture.map(|p| (p, opponent)),
            mv_from: mv.from,
            mv_to: mv.to,
            promotion: mv.promotion,
            old_zobrist_hash: self.zobrist_hash,
            old_en_passant: self.game_state.en_passant_square,
            old_castling_rights: self.game_state.castling_rights,
            old_halfmove_clock: self.game_state.halfmove_clock,
        };

        self.history_count += 1;

        // 2. Update zobrist for en passant (remove old)
        if let Some(ep_sq) = self.game_state.en_passant_square {
            self.zobrist_toggle_en_passant(ep_sq.file());
        }

        // 3. Remove piece from source (we know the piece type and color)
        self.remove_piece_known(mv.from, mv.piece, side);
        self.zobrist_toggle_piece(mv.from, mv.piece, side);

        // 4. Handle captures (we know the captured piece type)
        if let Some(captured) = mv.capture {
            if mv.flags.is_en_passant {
                let captured_sq = mv.to.down(side).expect("Invalid en passant square");
                self.remove_piece_known(captured_sq, Piece::Pawn, opponent);
                self.zobrist_toggle_piece(captured_sq, Piece::Pawn, opponent);
            } else {
                self.remove_piece_known(mv.to, captured, opponent);
                self.zobrist_toggle_piece(mv.to, captured, opponent);
            }
        }

        // 5. Place piece at destination (with promotion if applicable)
        let final_piece = mv.promotion.unwrap_or(mv.piece);
        self.place_piece_internal(mv.to, final_piece, side);
        self.zobrist_toggle_piece(mv.to, final_piece, side);

        // 6. Handle castling rook movement
        if mv.flags.is_castle {
            let (rook_from, rook_to) = Self::get_castling_rook_squares(mv.to, side)?;
            self.remove_piece_known(rook_from, Piece::Rook, side);
            self.place_piece_internal(rook_to, Piece::Rook, side);
            self.zobrist_toggle_piece(rook_from, Piece::Rook, side);
            self.zobrist_toggle_piece(rook_to, Piece::Rook, side);
        }

        // 7. Update castling rights
        let old_castling = self.game_state.castling_rights;
        self.update_castling_rights_after_move(mv);
        let new_castling = self.game_state.castling_rights;
        self.zobrist_update_castling(&old_castling, &new_castling);

        // 8. Update en passant square
        self.game_state.en_passant_square = if mv.flags.is_double_pawn_push {
            mv.from.up(side)
        } else {
            None
        };

        // Update zobrist for new en passant
        if let Some(ep_sq) = self.game_state.en_passant_square {
            self.zobrist_toggle_en_passant(ep_sq.file());
        }

        // 9. Update halfmove clock
        if mv.piece == Piece::Pawn || mv.capture.is_some() {
            self.game_state.halfmove_clock = 0;
        } else {
            self.game_state.halfmove_clock += 1;
        }

        // 10. Switch side to move
        self.game_state.switch_side();
        self.zobrist_toggle_side();

        Ok(())
    }

    fn unmake_move(&mut self, mv: &Move) -> Result<()> {
        if self.history_count == 0 {
            return Err(BoardError::ChessMoveError(MoveError::NoMoveToUnmake));
        }

        self.history_count -= 1;
        let state = self.move_history[self.history_count];

        // Restore side to move (switch back)
        self.game_state.side_to_move = self.game_state.side_to_move.opponent();
        let side = self.game_state.side_to_move;

        // Remove piece from destination (we know the piece - it's the moved piece or promotion)
        let final_piece = mv.promotion.unwrap_or(mv.piece);
        self.remove_piece_known(mv.to, final_piece, side);

        // Place original piece at source
        self.place_piece_internal(mv.from, mv.piece, side);

        // Restore captured piece
        if let Some((captured_piece, captured_color)) = state.captured_piece {
            if mv.flags.is_en_passant {
                let captured_sq = mv.to.down(side).expect("Invalid en passant square");
                self.place_piece_internal(captured_sq, captured_piece, captured_color);
            } else {
                self.place_piece_internal(mv.to, captured_piece, captured_color);
            }
        }

        // Unmake castling rook movement
        if mv.flags.is_castle {
            let (rook_from, rook_to) = Self::get_castling_rook_squares(mv.to, side)?;
            self.remove_piece_known(rook_to, Piece::Rook, side);
            self.place_piece_internal(rook_from, Piece::Rook, side);
        }

        // Restore game state
        self.game_state.en_passant_square = state.old_en_passant;
        self.game_state.castling_rights = state.old_castling_rights;
        self.game_state.halfmove_clock = state.old_halfmove_clock;
        self.zobrist_hash = state.old_zobrist_hash;

        Ok(())
    }

    fn make_null_move(&mut self) {
        let state = MoveState {
            captured_piece: None,
            mv_from: Square::A1, // Dummy values
            mv_to: Square::A1,   // Dummy values
            promotion: None,
            old_zobrist_hash: self.zobrist_hash,
            old_en_passant: self.game_state.en_passant_square,
            old_castling_rights: self.game_state.castling_rights,
            old_halfmove_clock: self.game_state.halfmove_clock,
        };
        self.move_history[self.history_count] = state;
        self.history_count += 1;

        if let Some(ep_sq) = self.game_state.en_passant_square {
            self.zobrist_toggle_en_passant(ep_sq.file());
            self.game_state.en_passant_square = None;
        }

        self.game_state.switch_side();
        self.zobrist_toggle_side();
    }

    fn unmake_null_move(&mut self) {
        if self.history_count == 0 {
            return;
        }
        self.history_count -= 1;

        let state = self.move_history[self.history_count];
        self.game_state.side_to_move = self.game_state.side_to_move.opponent();
        self.game_state.en_passant_square = state.old_en_passant;
        self.zobrist_hash = state.old_zobrist_hash;
    }

    fn is_in_check(&self, color: Color) -> bool {
        let king_sq = match self.get_king_square(color) {
            Some(sq) => sq,
            None => return false,
        };

        self.is_square_attacked(king_sq, color.opponent())
    }
}

impl Board {
    /// Internal method to place a piece on the board without updating game state
    #[inline(always)]
    pub(crate) fn place_piece_internal(&mut self, square: Square, piece: Piece, color: Color) {
        let bb = BitBoard::from_square(square);
        self.pieces[color as usize][piece as usize] |= bb;
        self.cache.add_piece(square, color);
        self.mailbox[square.to_index() as usize] = Some((piece, color));
    }

    /// Fast remove when piece type and color are known
    #[inline(always)]
    pub(crate) fn remove_piece_known(&mut self, square: Square, piece: Piece, color: Color) {
        let bb = BitBoard::from_square(square);
        self.pieces[color as usize][piece as usize] &= !bb;
        self.cache.remove_piece(square, color);
        self.mailbox[square.to_index() as usize] = None;
    }

    /// Get the source and destination squares for the castling rook
    #[inline]
    fn get_castling_rook_squares(king_to: Square, side: Color) -> Result<(Square, Square)> {
        match (side, king_to) {
            (Color::White, Square::G1) => Ok((Square::H1, Square::F1)),
            (Color::White, Square::C1) => Ok((Square::A1, Square::D1)),
            (Color::Black, Square::G8) => Ok((Square::H8, Square::F8)),
            (Color::Black, Square::C8) => Ok((Square::A8, Square::D8)),
            _ => Err(BoardError::InvalidCastlingDestination {
                square: king_to,
                color: side,
            }),
        }
    }

    /// Update castling rights after a move
    fn update_castling_rights_after_move(&mut self, mv: &Move) {
        let side = self.game_state.side_to_move;
        let opponent = side.opponent();

        // King move removes all castling rights for that side
        if mv.piece == Piece::King {
            self.game_state.castling_rights[side as usize] = CastlingRights::EMPTY;
        }

        // Rook move removes castling right for that side
        if mv.piece == Piece::Rook {
            let back_rank = side.back_rank();
            if mv.from == Square::new(File::H, back_rank) {
                self.game_state.castling_rights[side as usize].short = None;
            } else if mv.from == Square::new(File::A, back_rank) {
                self.game_state.castling_rights[side as usize].long = None;
            }
        }

        // Capturing opponent's rook removes their castling right
        if mv.capture == Some(Piece::Rook) {
            let opp_back_rank = opponent.back_rank();
            if mv.to == Square::new(File::H, opp_back_rank) {
                self.game_state.castling_rights[opponent as usize].short = None;
            } else if mv.to == Square::new(File::A, opp_back_rank) {
                self.game_state.castling_rights[opponent as usize].long = None;
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{FenOps, STARTING_POSITION_FEN};

    #[test]
    fn test_is_in_check_starting_position() {
        let board = Board::from_fen(STARTING_POSITION_FEN).unwrap();

        assert!(!board.is_in_check(Color::White));
        assert!(!board.is_in_check(Color::Black));
    }

    #[test]
    fn test_is_in_check_white_checked_by_rook() {
        let board = Board::from_fen("3kr3/8/8/8/8/8/8/4K3 w - - 0 1").unwrap();

        assert!(board.is_in_check(Color::White));
        assert!(!board.is_in_check(Color::Black));
    }

    #[test]
    fn test_is_in_check_black_checked_by_queen() {
        let board = Board::from_fen("4k3/8/8/8/8/8/8/4QK2 b - - 0 1").unwrap();

        assert!(!board.is_in_check(Color::White));
        assert!(board.is_in_check(Color::Black));
    }

    #[test]
    fn test_is_in_check_blocked_attack() {
        let board = Board::from_fen("3kr3/8/8/8/4P3/8/8/4K3 w - - 0 1").unwrap();

        assert!(!board.is_in_check(Color::White));
    }

    #[test]
    fn test_attackers_to_square_multiple() {
        let board = Board::from_fen("7k/8/8/8/4p3/8/3N4/1B5K w - - 0 1").unwrap();

        let attackers = board.attackers_to_square(Square::E4, Color::White);

        assert_eq!(attackers.count(), 2);
        assert!(attackers.has(Square::D2));
        assert!(attackers.has(Square::B1));
    }

    #[test]
    fn test_attackers_to_square_none() {
        let board = Board::from_fen("4k3/8/8/8/8/8/8/4K3 w - - 0 1").unwrap();

        let white_attackers = board.attackers_to_square(Square::E4, Color::White);
        let black_attackers = board.attackers_to_square(Square::E4, Color::Black);

        assert!(white_attackers.is_empty());
        assert!(black_attackers.is_empty());
    }

    #[test]
    fn test_is_in_check_knight_check() {
        let board = Board::from_fen("4k3/8/3N4/8/8/8/8/4K3 b - - 0 1").unwrap();

        assert!(board.is_in_check(Color::Black));
        assert!(!board.is_in_check(Color::White));
    }

    #[test]
    fn test_is_in_check_pawn_check() {
        let board = Board::from_fen("8/8/8/4k3/3P1P2/8/8/4K3 b - - 0 1").unwrap();

        assert!(board.is_in_check(Color::Black));
    }

    #[test]
    fn test_make_unmake_simple_pawn_move() {
        let mut board = Board::from_fen(STARTING_POSITION_FEN).unwrap();
        let original_fen = board.to_fen();

        let mv =
            Move::new(Square::E2, Square::E4, Piece::Pawn).with_flags(aether_core::MoveFlags {
                is_double_pawn_push: true,
                ..Default::default()
            });

        board.make_move(&mv).unwrap();

        // Verify move was made
        assert!(board.piece_at(Square::E4).is_some());
        assert!(board.piece_at(Square::E2).is_none());
        assert_eq!(board.side_to_move(), Color::Black);

        board.unmake_move(&mv).unwrap();

        // Verify position restored
        assert_eq!(board.to_fen(), original_fen);
    }

    #[test]
    fn test_make_unmake_capture() {
        let mut board =
            Board::from_fen("rnbqkbnr/ppp1pppp/8/3p4/4P3/8/PPPP1PPP/RNBQKBNR w KQkq d6 0 2")
                .unwrap();
        let original_fen = board.to_fen();

        let mv = Move::new(Square::E4, Square::D5, Piece::Pawn).with_capture(Piece::Pawn);

        board.make_move(&mv).unwrap();
        board.unmake_move(&mv).unwrap();

        assert_eq!(board.to_fen(), original_fen);
    }

    #[test]
    fn test_make_unmake_castling_kingside() {
        let mut board =
            Board::from_fen("r3k2r/pppppppp/8/8/8/8/PPPPPPPP/R3K2R w KQkq - 0 1").unwrap();
        let original_fen = board.to_fen();

        let mv =
            Move::new(Square::E1, Square::G1, Piece::King).with_flags(aether_core::MoveFlags {
                is_castle: true,
                ..Default::default()
            });

        board.make_move(&mv).unwrap();

        // Verify castling
        assert!(board.piece_at(Square::G1).is_some());
        assert!(board.piece_at(Square::F1).is_some());
        assert!(board.piece_at(Square::E1).is_none());
        assert!(board.piece_at(Square::H1).is_none());

        board.unmake_move(&mv).unwrap();

        assert_eq!(board.to_fen(), original_fen);
    }

    #[test]
    fn test_make_unmake_en_passant() {
        let mut board =
            Board::from_fen("rnbqkbnr/pppp1ppp/8/3Pp3/8/8/PPP1PPPP/RNBQKBNR w KQkq e6 0 1")
                .unwrap();
        let original_fen = board.to_fen();

        let mv = Move::new(Square::D5, Square::E6, Piece::Pawn)
            .with_capture(Piece::Pawn)
            .with_flags(aether_core::MoveFlags {
                is_en_passant: true,
                ..Default::default()
            });

        board.make_move(&mv).unwrap();

        // Verify en passant capture
        assert!(board.piece_at(Square::E6).is_some());
        assert!(board.piece_at(Square::E5).is_none()); // Captured pawn removed
        assert!(board.piece_at(Square::D5).is_none());

        board.unmake_move(&mv).unwrap();

        assert_eq!(board.to_fen(), original_fen);
    }

    #[test]
    fn test_make_unmake_promotion() {
        let mut board = Board::from_fen("8/P7/8/8/8/8/8/4K2k w - - 0 1").unwrap();
        let original_fen = board.to_fen();

        let mv = Move::new(Square::A7, Square::A8, Piece::Pawn).with_promotion(Piece::Queen);

        board.make_move(&mv).unwrap();

        // Verify promotion
        let (piece, _) = board.piece_at(Square::A8).unwrap();
        assert_eq!(piece, Piece::Queen);

        board.unmake_move(&mv).unwrap();

        assert_eq!(board.to_fen(), original_fen);
    }

    #[test]
    fn test_halfmove_clock_reset_on_pawn_move() {
        let mut board =
            Board::from_fen("rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 5 3").unwrap();

        let mv =
            Move::new(Square::E2, Square::E4, Piece::Pawn).with_flags(aether_core::MoveFlags {
                is_double_pawn_push: true,
                ..Default::default()
            });

        board.make_move(&mv).unwrap();

        assert_eq!(board.game_state().halfmove_clock, 0);
    }

    #[test]
    fn test_halfmove_clock_reset_on_capture() {
        let mut board =
            Board::from_fen("rnbqkbnr/ppp1pppp/8/3p4/4P3/8/PPPP1PPP/RNBQKBNR w KQkq - 5 2")
                .unwrap();

        let mv = Move::new(Square::E4, Square::D5, Piece::Pawn).with_capture(Piece::Pawn);

        board.make_move(&mv).unwrap();

        assert_eq!(board.game_state().halfmove_clock, 0);
    }

    #[test]
    fn test_halfmove_clock_increment() {
        let mut board =
            Board::from_fen("rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1").unwrap();

        let mv = Move::new(Square::G1, Square::F3, Piece::Knight);

        board.make_move(&mv).unwrap();

        assert_eq!(board.game_state().halfmove_clock, 1);
    }

    #[test]
    fn test_castling_rights_removed_on_king_move() {
        let mut board =
            Board::from_fen("r3k2r/pppppppp/8/8/8/8/PPPPPPPP/R3K2R w KQkq - 0 1").unwrap();

        let mv = Move::new(Square::E1, Square::F1, Piece::King);

        board.make_move(&mv).unwrap();

        assert!(!board.can_castle_short(Color::White));
        assert!(!board.can_castle_long(Color::White));
        // Black rights unchanged
        assert!(board.can_castle_short(Color::Black));
        assert!(board.can_castle_long(Color::Black));
    }

    #[test]
    fn test_castling_rights_removed_on_rook_move() {
        let mut board =
            Board::from_fen("r3k2r/pppppppp/8/8/8/8/PPPPPPPP/R3K2R w KQkq - 0 1").unwrap();

        let mv = Move::new(Square::H1, Square::G1, Piece::Rook);

        board.make_move(&mv).unwrap();

        assert!(!board.can_castle_short(Color::White));
        assert!(board.can_castle_long(Color::White)); // Queenside still available
    }

    #[test]
    fn test_castling_rights_removed_on_rook_capture() {
        let mut board =
            Board::from_fen("r3k2r/pppppppp/8/8/8/7B/PPPPPPPP/R3K2R w KQkq - 0 1").unwrap();

        let mv = Move::new(Square::H3, Square::H8, Piece::Bishop).with_capture(Piece::Rook);

        board.make_move(&mv).unwrap();

        assert!(!board.can_castle_short(Color::Black));
        assert!(board.can_castle_long(Color::Black)); // Queenside still available
    }
}
