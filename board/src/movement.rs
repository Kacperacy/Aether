use crate::{Board, BoardOps, Result};
use aether_types::{BitBoard, BoardQuery, Color, File, Move, MoveState, Piece, Rank, Square};

impl Board {
    /// Places a piece on the board at the given square.
    /// Does not update Zobrist hash - caller is responsible.
    pub fn place_piece(&mut self, square: Square, piece: Piece, color: Color) {
        let bb = BitBoard::from_square(square);
        self.pieces[color as usize][piece as usize] |= bb;

        // Update combined bitboards
        self.cache.color_combined[color as usize] |= bb;
        self.cache.occupied |= bb;
    }

    /// Removes a piece from the board at the given square.
    /// Returns the piece and color if one was present.
    /// Does not update Zobrist hash - caller is responsible.
    pub fn remove_piece(&mut self, square: Square) -> Option<(Piece, Color)> {
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
                return Some((Piece::all()[i], color));
            }
        }
        None
    }

    /// Simple move without state management (for backward compatibility).
    /// Use BoardOps::make_move for full move execution with state tracking.
    pub fn make_simple_move(&mut self, from: Square, to: Square) -> Option<(Piece, Color)> {
        if let Some((piece, color)) = self.remove_piece(from) {
            // Capture if any piece on destination
            let _ = self.remove_piece(to);
            self.place_piece(to, piece, color);
            self.change_side_to_move();
            return Some((piece, color));
        }
        None
    }
}

impl BoardOps for Board {
    /// Execute a move on the board with full state management.
    ///
    /// Handles:
    /// - Normal moves and captures
    /// - Castling (king and rook movement)
    /// - En passant captures
    /// - Pawn promotions
    /// - Halfmove clock (reset on pawn moves and captures)
    /// - Fullmove number increment
    /// - Castling rights updates
    /// - En passant square setting/clearing
    /// - Zobrist hash updates
    /// - Move history for unmake
    fn make_move(&mut self, mv: Move) -> Result<()> {
        let stm = self.game_state.side_to_move;

        // Save current state for unmake
        let move_state = MoveState {
            captured_piece: None, // Will be updated if capture occurs
            mv_from: mv.from,
            mv_to: mv.to,
            promotion: mv.promotion,
            old_zobrist_hash: self.zobrist_hash,
            old_en_passant: self.game_state.en_passant_square,
            old_castling_rights: self.game_state.castling_rights.clone(),
            old_halfmove_clock: self.game_state.halfmove_clock,
        };

        // Remove piece from source square
        let moving_piece = self.remove_piece(mv.from)
            .ok_or_else(|| crate::BoardError::InvalidMove {
                reason: format!("No piece at source square {}", mv.from)
            })?;

        // Validate piece matches move
        if moving_piece.0 != mv.piece || moving_piece.1 != stm {
            return Err(crate::BoardError::InvalidMove {
                reason: format!(
                    "Piece mismatch: expected {:?} {:?}, found {:?} {:?}",
                    mv.piece, stm, moving_piece.0, moving_piece.1
                )
            }.into());
        }

        let mut captured_piece = None;

        // Handle special moves
        if mv.flags.is_castle {
            // Castling: move the rook
            let (rook_from, rook_to) = if mv.to.file() > mv.from.file() {
                // Kingside castling
                let rook_file = File::H;
                let new_rook_file = File::F;
                (
                    Square::new(rook_file, mv.from.rank()),
                    Square::new(new_rook_file, mv.from.rank()),
                )
            } else {
                // Queenside castling
                let rook_file = File::A;
                let new_rook_file = File::D;
                (
                    Square::new(rook_file, mv.from.rank()),
                    Square::new(new_rook_file, mv.from.rank()),
                )
            };

            // Move rook
            if let Some((rook, rook_color)) = self.remove_piece(rook_from) {
                if rook != Piece::Rook || rook_color != stm {
                    return Err(crate::BoardError::InvalidMove {
                        reason: format!("Invalid rook for castling at {}", rook_from)
                    }.into());
                }
                self.place_piece(rook_to, Piece::Rook, stm);
            } else {
                return Err(crate::BoardError::InvalidMove {
                    reason: format!("No rook found at {} for castling", rook_from)
                }.into());
            }
        } else if mv.flags.is_en_passant {
            // En passant: remove the captured pawn
            let captured_pawn_square = if stm == Color::White {
                Square::new(mv.to.file(), Rank::new(4)) // 5th rank (0-indexed = 4)
            } else {
                Square::new(mv.to.file(), Rank::new(3)) // 4th rank
            };

            captured_piece = self.remove_piece(captured_pawn_square);
        } else {
            // Normal move: handle capture at destination
            captured_piece = self.remove_piece(mv.to);
        }

        // Place piece at destination (with promotion if applicable)
        let final_piece = mv.promotion.unwrap_or(mv.piece);
        self.place_piece(mv.to, final_piece, stm);

        // Update move state with captured piece
        let mut final_move_state = move_state;
        final_move_state.captured_piece = captured_piece;

        // Update halfmove clock
        if mv.piece == Piece::Pawn || captured_piece.is_some() {
            self.game_state.halfmove_clock = 0;
        } else {
            self.game_state.halfmove_clock += 1;
        }

        // Update fullmove number (increments after Black's move)
        if stm == Color::Black {
            self.game_state.fullmove_number += 1;
        }

        // Update castling rights
        self.update_castling_rights_after_move(mv.from, mv.to);

        // Update en passant square
        self.game_state.en_passant_square = if mv.flags.is_double_pawn_push {
            let ep_rank = if stm == Color::White { Rank::new(2) } else { Rank::new(5) };
            Some(Square::new(mv.from.file(), ep_rank))
        } else {
            None
        };

        // Update Zobrist hash (will be implemented when zobrist module is complete)
        self.zobrist_hash = self.compute_zobrist_hash();

        // Save state to history
        self.move_history.push(final_move_state);

        // Switch side to move
        self.change_side_to_move();

        // Invalidate cache
        self.invalidate_cache();

        Ok(())
    }

    /// Undo the last move on the board.
    /// Restores the previous position from move history.
    fn unmake_move(&mut self, _mv: Move) -> Result<()> {
        let move_state = self.move_history.pop()
            .ok_or_else(|| crate::BoardError::InvalidMove {
                reason: "No move to unmake".to_string()
            })?;

        // Switch side back
        self.change_side_to_move();
        let stm = self.game_state.side_to_move;

        // Restore piece at original square
        let piece_at_dest = self.remove_piece(move_state.mv_to)
            .ok_or_else(|| crate::BoardError::InvalidMove {
                reason: format!("No piece at destination {}", move_state.mv_to)
            })?;

        // Determine original piece (before promotion)
        let original_piece = if move_state.promotion.is_some() {
            Piece::Pawn
        } else {
            piece_at_dest.0
        };

        self.place_piece(move_state.mv_from, original_piece, stm);

        // Restore captured piece if any
        if let Some((captured_piece, captured_color)) = move_state.captured_piece {
            // For en passant, restore pawn at correct square
            let capture_square = if original_piece == Piece::Pawn &&
                move_state.mv_from.file() != move_state.mv_to.file() &&
                piece_at_dest.0 == Piece::Pawn {
                // This was likely en passant
                let ep_rank = if stm == Color::White { Rank::new(4) } else { Rank::new(3) };
                Square::new(move_state.mv_to.file(), ep_rank)
            } else {
                move_state.mv_to
            };

            self.place_piece(capture_square, captured_piece, captured_color);
        }

        // Restore castling rook if this was a castling move
        // Check if this was a king move of 2 squares
        if original_piece == Piece::King {
            let file_diff = (move_state.mv_to.file() as i8) - (move_state.mv_from.file() as i8);
            if file_diff.abs() == 2 {
                // This was castling - restore rook
                let (rook_current, rook_original) = if file_diff > 0 {
                    // Kingside
                    (
                        Square::new(File::F, move_state.mv_from.rank()),
                        Square::new(File::H, move_state.mv_from.rank()),
                    )
                } else {
                    // Queenside
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

        // Restore game state
        self.game_state.en_passant_square = move_state.old_en_passant;
        self.game_state.castling_rights = move_state.old_castling_rights;
        self.game_state.halfmove_clock = move_state.old_halfmove_clock;
        self.zobrist_hash = move_state.old_zobrist_hash;

        // Restore fullmove number (decrement if we just unmade a Black move)
        if stm == Color::Black {
            self.game_state.fullmove_number = self.game_state.fullmove_number.saturating_sub(1);
        }

        // Invalidate cache
        self.invalidate_cache();

        Ok(())
    }
}

impl Board {
    /// Update castling rights based on a move.
    /// Removes rights when king or rooks move.
    fn update_castling_rights_after_move(&mut self, from: Square, to: Square) {
        // Check if king moved
        if let Some((piece, color)) = self.piece_at(to) {
            if piece == Piece::King {
                // King moved - remove all castling rights for this color
                self.game_state.castling_rights[color as usize].short = None;
                self.game_state.castling_rights[color as usize].long = None;
            }
        }

        // Check if rook moved from initial square
        for color in [Color::White, Color::Black] {
            let back_rank = if color == Color::White { Rank::new(0) } else { Rank::new(7) };

            // Check kingside rook
            if let Some(kingside_file) = self.game_state.castling_rights[color as usize].short {
                if from == Square::new(kingside_file, back_rank) {
                    self.game_state.castling_rights[color as usize].short = None;
                }
            }

            // Check queenside rook
            if let Some(queenside_file) = self.game_state.castling_rights[color as usize].long {
                if from == Square::new(queenside_file, back_rank) {
                    self.game_state.castling_rights[color as usize].long = None;
                }
            }

            // Check if rook was captured (remove castling rights for that rook)
            if let Some(kingside_file) = self.game_state.castling_rights[color as usize].short {
                if to == Square::new(kingside_file, back_rank) {
                    self.game_state.castling_rights[color as usize].short = None;
                }
            }

            if let Some(queenside_file) = self.game_state.castling_rights[color as usize].long {
                if to == Square::new(queenside_file, back_rank) {
                    self.game_state.castling_rights[color as usize].long = None;
                }
            }
        }
    }
}
