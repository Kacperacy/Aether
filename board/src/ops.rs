use crate::error::BoardError::ChessMoveError;
use crate::error::MoveError::{
    InvalidRookForCastling, NoMoveToUnmake, NoPieceAtSource, NoRookForCastling, PieceMismatch,
};
use crate::query::BoardQuery;
use crate::{Board, Result};
use aether_core::{ALL_PIECES, BitBoard, Color, File, Move, MoveState, Piece, Rank, Square};

/// Trait for board operations
pub trait BoardOps: BoardQuery + Clone {
    /// Place a piece on the board at the given square.
    fn place_piece(&mut self, square: Square, piece: Piece, color: Color);

    /// Remove a piece from the board at the given square, returning the removed piece and color if any.
    fn remove_piece(&mut self, square: Square) -> Option<(Piece, Color)>;

    /// Make a move on the board, updating the position accordingly.
    fn make_move(&mut self, mv: &Move) -> Result<()>;

    /// Unmake a move on the board, restoring the previous position.
    fn unmake_move(&mut self, mv: &Move) -> Result<()>;

    /// Is the position in check for the side to move?
    fn is_in_check(&mut self) -> bool;
}

impl BoardOps for Board {
    fn place_piece(&mut self, square: Square, piece: Piece, color: Color) {
        let bb = BitBoard::from_square(square);
        self.pieces[color as usize][piece as usize] |= bb;

        // Update combined bitboards
        self.cache.color_combined[color as usize] |= bb;
        self.cache.occupied |= bb;
    }

    fn remove_piece(&mut self, square: Square) -> Option<(Piece, Color)> {
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
        for (i, piece_bb) in self.pieces[color as usize].iter().enumerate() {
            if piece_bb.has(square) {
                self.pieces[color as usize][i] &= !bb;
                self.cache.color_combined[color as usize] &= !bb;
                self.cache.occupied &= !bb;
                return Some((ALL_PIECES[i], color));
            }
        }

        None
    }

    fn make_move(&mut self, mv: &Move) -> Result<()> {
        let stm = self.game_state.side_to_move;

        let move_state = MoveState {
            captured_piece: None, // Placeholder for captured piece logic
            mv_from: mv.from,
            mv_to: mv.to,
            promotion: mv.promotion,
            old_zobrist_hash: self.zobrist_hash,
            old_en_passant: self.game_state.en_passant_square,
            old_castling_rights: self.game_state.castling_rights,
            old_halfmove_clock: self.game_state.halfmove_clock,
        };

        let moving_piece = self
            .remove_piece(mv.from)
            .ok_or_else(|| ChessMoveError(NoPieceAtSource { square: mv.from }))?;

        if moving_piece.0 != mv.piece || moving_piece.1 != stm {
            return Err(ChessMoveError(PieceMismatch {
                expected: format!("{:?} {:?}", mv.piece, stm),
                found: format!("{:?} {:?}", moving_piece.0, moving_piece.1),
            }));
        }

        let mut captured_piece = None;

        if mv.flags.is_castle {
            // Castling: move the rook
            let (rook_from, rook_to) = if mv.to.file() > mv.from.file() {
                (
                    Square::new(File::H, mv.from.rank()),
                    Square::new(File::F, mv.from.rank()),
                )
            } else {
                (
                    Square::new(File::A, mv.from.rank()),
                    Square::new(File::D, mv.from.rank()),
                )
            };

            if let Some((rook, rook_color)) = self.remove_piece(rook_from) {
                if rook != Piece::Rook || rook_color != stm {
                    return Err(ChessMoveError(InvalidRookForCastling { square: rook_from }));
                }

                self.place_piece(rook_to, Piece::Rook, stm);
            } else {
                return Err(ChessMoveError(NoRookForCastling { square: rook_from }));
            }
        } else if mv.flags.is_en_passant {
            let captured_pawn_square = if stm == Color::White {
                Square::new(mv.to.file(), Rank::from_index(4))
            } else {
                Square::new(mv.to.file(), Rank::from_index(3))
            };

            captured_piece = self.remove_piece(captured_pawn_square);
        } else {
            captured_piece = self.remove_piece(mv.to);
        }

        let final_piece = mv.promotion.unwrap_or(mv.piece);
        self.place_piece(mv.to, final_piece, stm);

        let mut final_move_state = move_state;
        final_move_state.captured_piece = captured_piece;

        if mv.piece == Piece::Pawn || captured_piece.is_none() {
            self.game_state.halfmove_clock = 0;
        } else {
            self.game_state.halfmove_clock += 1;
        }

        if stm == Color::Black {
            self.game_state.fullmove_number += 1;
        }

        self.update_castling_rights_after_move(mv);

        self.game_state.en_passant_square = if mv.flags.is_double_pawn_push {
            let ep_rank = if stm == Color::White {
                Rank::Three
            } else {
                Rank::Six
            };
            Some(Square::new(mv.from.file(), ep_rank))
        } else {
            None
        };

        self.zobrist_hash = self.calculate_zobrist_hash();

        self.move_history.push(final_move_state);

        self.change_side_to_move();

        self.invalidate_cache();

        return Ok(());
    }

    fn unmake_move(&mut self, mv: &Move) -> Result<()> {
        let move_state = self
            .move_history
            .pop()
            .ok_or(ChessMoveError(NoMoveToUnmake))?;

        self.change_side_to_move();
        let stm = self.game_state.side_to_move;

        let piece_at_destination = self
            .remove_piece(move_state.mv_to)
            .ok_or(ChessMoveError(NoPieceAtSource { square: mv.to }))?;

        let original_piece = if move_state.promotion.is_some() {
            Piece::Pawn
        } else {
            piece_at_destination.0
        };

        self.place_piece(move_state.mv_from, original_piece, stm);

        if let Some((captured_piece, captured_color)) = move_state.captured_piece {
            let capture_square = if original_piece == Piece::Pawn
                && move_state.mv_from.file() != move_state.mv_to.file()
                && piece_at_destination.0 == Piece::Pawn
            {
                let ep_rank = if stm == Color::White {
                    Rank::from_index(4)
                } else {
                    Rank::from_index(3)
                };

                Square::new(move_state.mv_to.file(), ep_rank)
            } else {
                move_state.mv_to
            };

            self.place_piece(capture_square, captured_piece, captured_color);
        }

        if original_piece == Piece::King {
            let file_diff =
                move_state.mv_to.file().to_index() - move_state.mv_from.file().to_index();

            if file_diff.abs() == 2 {
                let (rook_current, rook_original) = if file_diff > 0 {
                    (
                        Square::new(File::F, move_state.mv_from.rank()),
                        Square::new(File::H, move_state.mv_from.rank()),
                    )
                } else {
                    (
                        Square::new(File::D, move_state.mv_from.rank()),
                        Square::new(File::A, move_state.mv_from.rank()),
                    )
                };

                if let Some((rook, rook_color)) = self.remove_piece(rook_current) {
                    self.place_piece(rook_original, rook, rook_color);
                }
            }
        }

        self.game_state.en_passant_square = move_state.old_en_passant;
        self.game_state.castling_rights = move_state.old_castling_rights;
        self.game_state.halfmove_clock = move_state.old_halfmove_clock;
        self.zobrist_hash = move_state.old_zobrist_hash;

        if stm == Color::Black {
            self.game_state.fullmove_number = self.game_state.fullmove_number.saturating_sub(1);
        }

        self.invalidate_cache();

        Ok(())
    }

    fn is_in_check(&mut self) -> bool {
        let color = self.game_state.side_to_move;

        if let Some(is_check) = self.cache.cached_check_status[color as usize] {
            return is_check;
        }

        let king_square = self.get_king_square(color);
        if king_square.is_none() {
            return false;
        }

        let in_check = self.is_square_attacked(king_square.unwrap(), color.opponent());
        self.cache.set_cached_check_status(color, in_check);
        in_check
    }
}

impl Board {
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
