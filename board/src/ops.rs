use crate::error::MoveError;
use crate::pst;
use crate::state_info::StateInfo;
use crate::{Board, BoardError, MAX_SEARCH_DEPTH, Result};
use aether_core::{
    BitBoard, CastlingRights, Color, File, Move, Piece, Square, compute_slider_blockers,
    is_square_attacked, line_through,
};

impl Board {
    #[inline(always)]
    pub fn make_move(&mut self, mv: &Move) -> Result<()> {
        let side = self.side_to_move;
        let opponent = side.opponent();

        self.zobrist_history.push(self.state.zobrist_hash);

        let buffer_idx = self.history_index % MAX_SEARCH_DEPTH;
        self.state_history[buffer_idx] = self.state;
        self.state_history[buffer_idx].captured_piece = mv.capture.map(|p| (p, opponent));
        self.history_index += 1;

        if let Some(captured) = mv.capture {
            let phase_delta = (Self::phase_weight(captured) as i32 * crate::MAX_GAME_PHASE
                / crate::PHASE_TOTAL as i32) as i16;
            self.state.game_phase = (self.state.game_phase - phase_delta).max(0);
        }

        if let Some(promo) = mv.promotion {
            let phase_delta = (Self::phase_weight(promo) as i32 * crate::MAX_GAME_PHASE
                / crate::PHASE_TOTAL as i32) as i16;
            self.state.game_phase =
                (self.state.game_phase + phase_delta).min(crate::MAX_GAME_PHASE as i16);
        }

        if let Some(ep_sq) = self.state.en_passant_square {
            self.zobrist_toggle_en_passant(ep_sq.file());
        }

        let (from_mg, from_eg) = pst::piece_value(mv.piece, mv.from, side);
        self.state.pst_mg -= from_mg;
        self.state.pst_eg -= from_eg;
        self.remove_piece_known(mv.from, mv.piece, side);
        self.zobrist_toggle_piece(mv.from, mv.piece, side);

        if let Some(captured) = mv.capture {
            if mv.flags.is_en_passant {
                let captured_sq = mv.to.down(side).expect("Invalid en passant square");
                let (cap_mg, cap_eg) = pst::piece_value(Piece::Pawn, captured_sq, opponent);
                self.state.pst_mg -= cap_mg;
                self.state.pst_eg -= cap_eg;
                self.remove_piece_known(captured_sq, Piece::Pawn, opponent);
                self.zobrist_toggle_piece(captured_sq, Piece::Pawn, opponent);
            } else {
                let (cap_mg, cap_eg) = pst::piece_value(captured, mv.to, opponent);
                self.state.pst_mg -= cap_mg;
                self.state.pst_eg -= cap_eg;
                self.remove_piece_known(mv.to, captured, opponent);
                self.zobrist_toggle_piece(mv.to, captured, opponent);
            }
        }

        let final_piece = mv.promotion.unwrap_or(mv.piece);
        let (to_mg, to_eg) = pst::piece_value(final_piece, mv.to, side);
        self.state.pst_mg += to_mg;
        self.state.pst_eg += to_eg;
        self.place_piece_internal(mv.to, final_piece, side);
        self.zobrist_toggle_piece(mv.to, final_piece, side);

        if mv.flags.is_castle {
            let (rook_from, rook_to) = Self::get_castling_rook_squares(mv.to, side)?;
            let (rf_mg, rf_eg) = pst::piece_value(Piece::Rook, rook_from, side);
            let (rt_mg, rt_eg) = pst::piece_value(Piece::Rook, rook_to, side);
            self.state.pst_mg += rt_mg - rf_mg;
            self.state.pst_eg += rt_eg - rf_eg;
            self.remove_piece_known(rook_from, Piece::Rook, side);
            self.place_piece_internal(rook_to, Piece::Rook, side);
            self.zobrist_toggle_piece(rook_from, Piece::Rook, side);
            self.zobrist_toggle_piece(rook_to, Piece::Rook, side);
        }

        let old_castling = self.state.castling_rights;
        self.update_castling_rights_after_move(mv);
        let new_castling = self.state.castling_rights;
        self.zobrist_update_castling(&old_castling, &new_castling);

        self.state.en_passant_square = if mv.flags.is_double_pawn_push {
            mv.from.up(side)
        } else {
            None
        };

        if let Some(ep_sq) = self.state.en_passant_square {
            self.zobrist_toggle_en_passant(ep_sq.file());
        }

        if mv.piece == Piece::Pawn || mv.capture.is_some() {
            self.state.halfmove_clock = 0;
        } else {
            self.state.halfmove_clock += 1;
        }

        if mv.piece == Piece::King {
            self.state.king_square[side as usize] = mv.to;
        }

        self.side_to_move = opponent;
        if self.side_to_move == Color::White {
            self.fullmove_number = self.fullmove_number.saturating_add(1);
        }
        self.zobrist_toggle_side();

        let new_king_sq = self.state.king_square[opponent as usize];
        self.state.checkers = self.attackers_to_square(new_king_sq, side);

        self.update_blockers();

        Ok(())
    }

    #[inline(always)]
    pub fn unmake_move(&mut self, mv: &Move) -> Result<()> {
        if self.history_index == 0 {
            return Err(BoardError::ChessMoveError(MoveError::NoMoveToUnmake));
        }

        self.history_index -= 1;
        let buffer_idx = self.history_index % MAX_SEARCH_DEPTH;
        let saved_state = self.state_history[buffer_idx];

        self.zobrist_history.pop();

        self.side_to_move = self.side_to_move.opponent();
        let side = self.side_to_move;

        if side == Color::Black {
            self.fullmove_number = self.fullmove_number.saturating_sub(1);
        }

        let final_piece = mv.promotion.unwrap_or(mv.piece);
        self.remove_piece_known(mv.to, final_piece, side);
        self.place_piece_internal(mv.from, mv.piece, side);

        if let Some((captured_piece, captured_color)) = saved_state.captured_piece {
            if mv.flags.is_en_passant {
                let captured_sq = mv.to.down(side).expect("Invalid en passant square");
                self.place_piece_internal(captured_sq, captured_piece, captured_color);
            } else {
                self.place_piece_internal(mv.to, captured_piece, captured_color);
            }
        }

        if mv.flags.is_castle {
            let (rook_from, rook_to) = Self::get_castling_rook_squares(mv.to, side)?;
            self.remove_piece_known(rook_to, Piece::Rook, side);
            self.place_piece_internal(rook_from, Piece::Rook, side);
        }

        self.state = StateInfo {
            castling_rights: saved_state.castling_rights,
            en_passant_square: saved_state.en_passant_square,
            halfmove_clock: saved_state.halfmove_clock,
            captured_piece: None,
            zobrist_hash: saved_state.zobrist_hash,
            game_phase: saved_state.game_phase,
            pst_mg: saved_state.pst_mg,
            pst_eg: saved_state.pst_eg,
            king_square: saved_state.king_square,
            checkers: saved_state.checkers,
            blockers_for_king: saved_state.blockers_for_king,
            pinners: saved_state.pinners,
        };

        Ok(())
    }

    pub fn make_null_move(&mut self) {
        let side = self.side_to_move;
        let opponent = side.opponent();
        self.zobrist_history.push(self.state.zobrist_hash);

        let buffer_idx = self.history_index % MAX_SEARCH_DEPTH;
        self.state_history[buffer_idx] = self.state;
        self.state_history[buffer_idx].captured_piece = None;
        self.history_index += 1;

        if let Some(ep_sq) = self.state.en_passant_square {
            self.zobrist_toggle_en_passant(ep_sq.file());
            self.state.en_passant_square = None;
        }

        self.side_to_move = opponent;
        if self.side_to_move == Color::White {
            self.fullmove_number = self.fullmove_number.saturating_add(1);
        }
        self.zobrist_toggle_side();

        let new_king_sq = self.state.king_square[opponent as usize];
        self.state.checkers = self.attackers_to_square(new_king_sq, side);
    }

    pub fn unmake_null_move(&mut self) {
        if self.history_index == 0 {
            return;
        }
        self.history_index -= 1;

        let buffer_idx = self.history_index % MAX_SEARCH_DEPTH;
        let saved_state = self.state_history[buffer_idx];

        self.zobrist_history.pop();

        self.side_to_move = self.side_to_move.opponent();

        if self.side_to_move == Color::Black {
            self.fullmove_number = self.fullmove_number.saturating_sub(1);
        }

        self.state.en_passant_square = saved_state.en_passant_square;
        self.state.zobrist_hash = saved_state.zobrist_hash;
        self.state.pst_mg = saved_state.pst_mg;
        self.state.pst_eg = saved_state.pst_eg;
        self.state.checkers = saved_state.checkers;
        self.state.blockers_for_king = saved_state.blockers_for_king;
        self.state.pinners = saved_state.pinners;
    }

    #[inline(always)]
    pub fn is_in_check(&self, color: Color) -> bool {
        if color == self.side_to_move {
            !self.state.checkers.is_empty()
        } else {
            let king_sq = self.get_king_square(color);
            self.is_square_attacked(king_sq, color.opponent())
        }
    }

    #[inline(always)]
    pub fn checkers(&self) -> BitBoard {
        self.state.checkers
    }

    #[inline]
    pub fn would_leave_king_in_check(&self, mv: &Move) -> bool {
        let side = self.side_to_move;
        let us = side as usize;

        if mv.piece == Piece::King {
            return self.king_move_is_illegal(mv, side);
        }

        if mv.flags.is_en_passant {
            return self.en_passant_is_illegal(mv, side);
        }

        if !self.state.checkers.is_empty() {
            return self.king_still_attacked_after(mv, side);
        }

        let from_bb = mv.from.bitboard();
        let blockers = self.state.blockers_for_king[us];

        if (blockers & from_bb).is_empty() {
            return false;
        }

        let king_sq = self.state.king_square[us];
        let pin_line = line_through(king_sq, mv.from);
        (pin_line & mv.to.bitboard()).is_empty()
    }

    #[inline]
    fn king_still_attacked_after(&self, mv: &Move, side: Color) -> bool {
        let opponent = side.opponent();
        let them = opponent as usize;

        let king_sq = self.state.king_square[side as usize];

        let mut occupied = self.cache.occupied;
        occupied &= !mv.from.bitboard();
        occupied |= mv.to.bitboard();

        let mut their_pieces = self.pieces[them];
        if let Some(captured) = mv.capture {
            their_pieces[captured as usize] &= !mv.to.bitboard();
        }

        is_square_attacked(king_sq, opponent, occupied, &their_pieces)
    }

    #[inline]
    fn king_move_is_illegal(&self, mv: &Move, side: Color) -> bool {
        let opponent = side.opponent();
        let them = opponent as usize;

        if mv.flags.is_castle {
            let occupied = (self.cache.occupied & !mv.from.bitboard()) | mv.to.bitboard();
            return is_square_attacked(mv.to, opponent, occupied, &self.pieces[them]);
        }

        let mut occupied = self.cache.occupied;
        occupied &= !mv.from.bitboard();
        occupied |= mv.to.bitboard();

        let mut their_pieces = self.pieces[them];
        if let Some(captured) = mv.capture {
            their_pieces[captured as usize] &= !mv.to.bitboard();
        }

        is_square_attacked(mv.to, opponent, occupied, &their_pieces)
    }

    #[inline]
    fn en_passant_is_illegal(&self, mv: &Move, side: Color) -> bool {
        let opponent = side.opponent();
        let us = side as usize;
        let them = opponent as usize;

        let king_sq = self.state.king_square[us];
        let captured_sq = mv.to.down(side).expect("Invalid en passant");

        let mut occupied = self.cache.occupied;
        occupied &= !mv.from.bitboard();
        occupied &= !captured_sq.bitboard();
        occupied |= mv.to.bitboard();

        let mut their_pieces = self.pieces[them];
        their_pieces[Piece::Pawn as usize] &= !captured_sq.bitboard();

        is_square_attacked(king_sq, opponent, occupied, &their_pieces)
    }

    #[inline(always)]
    pub(crate) fn place_piece_internal(&mut self, square: Square, piece: Piece, color: Color) {
        let bb = square.bitboard();
        self.pieces[color as usize][piece as usize] |= bb;
        self.cache.add_piece(square, color);
        self.mailbox[square.to_index() as usize] = Some((piece, color));
    }

    #[inline(always)]
    pub(crate) fn remove_piece_known(&mut self, square: Square, piece: Piece, color: Color) {
        let bb = square.bitboard();
        self.pieces[color as usize][piece as usize] &= !bb;
        self.cache.remove_piece(square, color);
        self.mailbox[square.to_index() as usize] = None;
    }

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

    fn update_castling_rights_after_move(&mut self, mv: &Move) {
        let side = self.side_to_move;
        let opponent = side.opponent();

        if mv.piece == Piece::King {
            self.state.castling_rights[side as usize] = CastlingRights::EMPTY;
        }

        if mv.piece == Piece::Rook {
            let back_rank = side.back_rank();
            if mv.from == Square::new(File::H, back_rank) {
                self.state.castling_rights[side as usize].short = None;
            } else if mv.from == Square::new(File::A, back_rank) {
                self.state.castling_rights[side as usize].long = None;
            }
        }

        if mv.capture == Some(Piece::Rook) {
            let opp_back_rank = opponent.back_rank();
            if mv.to == Square::new(File::H, opp_back_rank) {
                self.state.castling_rights[opponent as usize].short = None;
            } else if mv.to == Square::new(File::A, opp_back_rank) {
                self.state.castling_rights[opponent as usize].long = None;
            }
        }
    }

    #[inline]
    fn update_blockers(&mut self) {
        let white_king_sq = self.state.king_square[Color::White as usize];
        let black_king_sq = self.state.king_square[Color::Black as usize];
        let white_occ = self.cache.color_combined[Color::White as usize];
        let black_occ = self.cache.color_combined[Color::Black as usize];
        let occupied = self.cache.occupied;

        let (white_blockers, white_pinners) = compute_slider_blockers(
            white_king_sq,
            white_occ,
            &self.pieces[Color::Black as usize],
            occupied,
        );
        let (black_blockers, black_pinners) = compute_slider_blockers(
            black_king_sq,
            black_occ,
            &self.pieces[Color::White as usize],
            occupied,
        );

        self.state.blockers_for_king = [white_blockers, black_blockers];
        self.state.pinners = [white_pinners, black_pinners];
    }

    #[inline(always)]
    pub fn blockers_for_king(&self, color: Color) -> BitBoard {
        self.state.blockers_for_king[color as usize]
    }

    /// Returns enemy pieces that are pinning our pieces to our king.
    #[inline(always)]
    pub fn pinners(&self, color: Color) -> BitBoard {
        self.state.pinners[color as usize]
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::STARTING_POSITION_FEN;

    #[test]
    fn test_is_in_check_starting_position() {
        let board: Board = STARTING_POSITION_FEN.parse().unwrap();

        assert!(!board.is_in_check(Color::White));
        assert!(!board.is_in_check(Color::Black));
    }

    #[test]
    fn test_is_in_check_white_checked_by_rook() {
        let board: Board = "3kr3/8/8/8/8/8/8/4K3 w - - 0 1".parse().unwrap();

        assert!(board.is_in_check(Color::White));
        assert!(!board.is_in_check(Color::Black));
    }

    #[test]
    fn test_is_in_check_black_checked_by_queen() {
        let board: Board = "4k3/8/8/8/8/8/8/4QK2 b - - 0 1".parse().unwrap();

        assert!(!board.is_in_check(Color::White));
        assert!(board.is_in_check(Color::Black));
    }

    #[test]
    fn test_is_in_check_blocked_attack() {
        let board: Board = "3kr3/8/8/8/4P3/8/8/4K3 w - - 0 1".parse().unwrap();

        assert!(!board.is_in_check(Color::White));
    }

    #[test]
    fn test_attackers_to_square_multiple() {
        let board: Board = "7k/8/8/8/4p3/8/3N4/1B5K w - - 0 1".parse().unwrap();

        let attackers = board.attackers_to_square(Square::E4, Color::White);

        assert_eq!(attackers.count(), 2);
        assert!(attackers.has(Square::D2));
        assert!(attackers.has(Square::B1));
    }

    #[test]
    fn test_attackers_to_square_none() {
        let board: Board = "4k3/8/8/8/8/8/8/4K3 w - - 0 1".parse().unwrap();

        let white_attackers = board.attackers_to_square(Square::E4, Color::White);
        let black_attackers = board.attackers_to_square(Square::E4, Color::Black);

        assert!(white_attackers.is_empty());
        assert!(black_attackers.is_empty());
    }

    #[test]
    fn test_is_in_check_knight_check() {
        let board: Board = "4k3/8/3N4/8/8/8/8/4K3 b - - 0 1".parse().unwrap();

        assert!(board.is_in_check(Color::Black));
        assert!(!board.is_in_check(Color::White));
    }

    #[test]
    fn test_is_in_check_pawn_check() {
        let board: Board = "8/8/8/4k3/3P1P2/8/8/4K3 b - - 0 1".parse().unwrap();

        assert!(board.is_in_check(Color::Black));
    }

    #[test]
    fn test_make_unmake_simple_pawn_move() {
        let mut board: Board = STARTING_POSITION_FEN.parse().unwrap();
        let original_fen = board.to_string();

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
        assert_eq!(board.to_string(), original_fen);
    }

    #[test]
    fn test_make_unmake_capture() {
        let mut board: Board = "rnbqkbnr/ppp1pppp/8/3p4/4P3/8/PPPP1PPP/RNBQKBNR w KQkq d6 0 2"
            .parse()
            .unwrap();
        let original_fen = board.to_string();

        let mv = Move::new(Square::E4, Square::D5, Piece::Pawn).with_capture(Piece::Pawn);

        board.make_move(&mv).unwrap();
        board.unmake_move(&mv).unwrap();

        assert_eq!(board.to_string(), original_fen);
    }

    #[test]
    fn test_make_unmake_castling_kingside() {
        let mut board: Board = "r3k2r/pppppppp/8/8/8/8/PPPPPPPP/R3K2R w KQkq - 0 1"
            .parse()
            .unwrap();
        let original_fen = board.to_string();

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

        assert_eq!(board.to_string(), original_fen);
    }

    #[test]
    fn test_make_unmake_en_passant() {
        let mut board: Board = "rnbqkbnr/pppp1ppp/8/3Pp3/8/8/PPP1PPPP/RNBQKBNR w KQkq e6 0 1"
            .parse()
            .unwrap();
        let original_fen = board.to_string();

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

        assert_eq!(board.to_string(), original_fen);
    }

    #[test]
    fn test_make_unmake_promotion() {
        let mut board: Board = "8/P7/8/8/8/8/8/4K2k w - - 0 1".parse().unwrap();
        let original_fen = board.to_string();

        let mv = Move::new(Square::A7, Square::A8, Piece::Pawn).with_promotion(Piece::Queen);

        board.make_move(&mv).unwrap();

        // Verify promotion
        let (piece, _) = board.piece_at(Square::A8).unwrap();
        assert_eq!(piece, Piece::Queen);

        board.unmake_move(&mv).unwrap();

        assert_eq!(board.to_string(), original_fen);
    }

    #[test]
    fn test_halfmove_clock_reset_on_pawn_move() {
        let mut board: Board = "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 5 3"
            .parse()
            .unwrap();

        let mv =
            Move::new(Square::E2, Square::E4, Piece::Pawn).with_flags(aether_core::MoveFlags {
                is_double_pawn_push: true,
                ..Default::default()
            });

        board.make_move(&mv).unwrap();

        assert_eq!(board.halfmove_clock(), 0);
    }

    #[test]
    fn test_halfmove_clock_reset_on_capture() {
        let mut board: Board = "rnbqkbnr/ppp1pppp/8/3p4/4P3/8/PPPP1PPP/RNBQKBNR w KQkq - 5 2"
            .parse()
            .unwrap();

        let mv = Move::new(Square::E4, Square::D5, Piece::Pawn).with_capture(Piece::Pawn);

        board.make_move(&mv).unwrap();

        assert_eq!(board.halfmove_clock(), 0);
    }

    #[test]
    fn test_halfmove_clock_increment() {
        let mut board: Board = "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1"
            .parse()
            .unwrap();

        let mv = Move::new(Square::G1, Square::F3, Piece::Knight);

        board.make_move(&mv).unwrap();

        assert_eq!(board.halfmove_clock(), 1);
    }

    #[test]
    fn test_castling_rights_removed_on_king_move() {
        let mut board: Board = "r3k2r/pppppppp/8/8/8/8/PPPPPPPP/R3K2R w KQkq - 0 1"
            .parse()
            .unwrap();

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
        let mut board: Board = "r3k2r/pppppppp/8/8/8/8/PPPPPPPP/R3K2R w KQkq - 0 1"
            .parse()
            .unwrap();

        let mv = Move::new(Square::H1, Square::G1, Piece::Rook);

        board.make_move(&mv).unwrap();

        assert!(!board.can_castle_short(Color::White));
        assert!(board.can_castle_long(Color::White)); // Queenside still available
    }

    #[test]
    fn test_castling_rights_removed_on_rook_capture() {
        let mut board: Board = "r3k2r/pppppppp/8/8/8/7B/PPPPPPPP/R3K2R w KQkq - 0 1"
            .parse()
            .unwrap();

        let mv = Move::new(Square::H3, Square::H8, Piece::Bishop).with_capture(Piece::Rook);

        board.make_move(&mv).unwrap();

        assert!(!board.can_castle_short(Color::Black));
        assert!(board.can_castle_long(Color::Black)); // Queenside still available
    }
}
