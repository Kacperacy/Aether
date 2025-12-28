use crate::MoveGen;
use aether_core::{
    ALL_PIECES, BitBoard, Color, Move, MoveFlags, PROMOTION_PIECES, Piece, Square, bishop_attacks,
    is_promotion_rank, is_square_attacked, king_attacks, knight_attacks, pawn_attacks, pawn_moves,
    queen_attacks, rook_attacks,
};
use board::BoardQuery;

#[derive(Debug, Default, Clone, Copy)]
pub struct Generator;

impl Generator {
    #[inline]
    pub fn new() -> Self {
        Self
    }

    /// Builds occupancy bitboards for move generation
    #[inline]
    fn occupancies<T: BoardQuery>(&self, board: &T, side: Color) -> (BitBoard, BitBoard, BitBoard) {
        let own = board.occupied_by(side);
        let opponent = board.occupied_by(side.opponent());
        let all = own | opponent;
        (all, own, opponent)
    }

    /// Appends a move to the move list
    #[inline(always)]
    fn push_move(
        moves: &mut Vec<Move>,
        from: Square,
        to: Square,
        piece: Piece,
        capture: Option<Piece>,
        flags: MoveFlags,
        promotion: Option<Piece>,
    ) {
        let mut chess_move = Move::new(from, to, piece).with_flags(flags);

        if let Some(captured_piece) = capture {
            chess_move = chess_move.with_capture(captured_piece);
        }

        if let Some(promotion_piece) = promotion {
            chess_move = chess_move.with_promotion(promotion_piece);
        }

        moves.push(chess_move);
    }

    /// Generates standard piece moves from an attack bitboard
    /// Handles captures automatically by checking occupancy
    #[inline]
    fn generate_piece_moves<T: BoardQuery>(
        &self,
        board: &T,
        from: Square,
        piece: Piece,
        targets: BitBoard,
        occupied: BitBoard,
        moves: &mut Vec<Move>,
    ) {
        let flags = MoveFlags::default();

        for to in targets.iter() {
            let capture = if occupied.has(to) {
                board.piece_at(to).map(|(p, _)| p)
            } else {
                None
            };
            Self::push_move(moves, from, to, piece, capture, flags, None);
        }
    }

    /// Generates all pawn moves including pushes, captures, promotions, and en passant
    fn generate_pawn_moves<T: BoardQuery>(
        &self,
        board: &T,
        from: Square,
        side: Color,
        occupied: BitBoard,
        opponent_pieces: BitBoard,
        moves: &mut Vec<Move>,
    ) {
        let normal_flags = MoveFlags::default();
        let double_push_flags = MoveFlags {
            is_double_pawn_push: true,
            ..MoveFlags::default()
        };

        // Generate pawn pushes
        let push_targets = pawn_moves(from, side, occupied);
        for to in push_targets.iter() {
            let is_promotion = is_promotion_rank(to, side);
            let is_double_push = to.rank().to_index().abs_diff(from.rank().to_index()) == 2;
            let flags = if is_double_push {
                double_push_flags
            } else {
                normal_flags
            };

            if is_promotion {
                for &promo_piece in &PROMOTION_PIECES {
                    Self::push_move(moves, from, to, Piece::Pawn, None, flags, Some(promo_piece));
                }
            } else {
                Self::push_move(moves, from, to, Piece::Pawn, None, flags, None);
            }
        }

        // Generate pawn captures
        let capture_targets = pawn_attacks(from, side) & opponent_pieces;
        for to in capture_targets.iter() {
            let captured = board.piece_at(to).map(|(p, _)| p);
            let is_promotion = is_promotion_rank(to, side);

            if is_promotion {
                for &promo_piece in &PROMOTION_PIECES {
                    Self::push_move(
                        moves,
                        from,
                        to,
                        Piece::Pawn,
                        captured,
                        normal_flags,
                        Some(promo_piece),
                    );
                }
            } else if captured.is_some() {
                Self::push_move(moves, from, to, Piece::Pawn, captured, normal_flags, None);
            }
        }

        // Generate en passant captures
        if let Some(ep_square) = board.en_passant_square() {
            if pawn_attacks(from, side).has(ep_square) {
                let ep_flags = MoveFlags {
                    is_en_passant: true,
                    ..MoveFlags::default()
                };
                Self::push_move(
                    moves,
                    from,
                    ep_square,
                    Piece::Pawn,
                    Some(Piece::Pawn),
                    ep_flags,
                    None,
                );
            }
        }
    }

    /// Generates knight moves
    fn generate_knight_moves<T: BoardQuery>(
        &self,
        board: &T,
        from: Square,
        occupied: BitBoard,
        own_pieces: BitBoard,
        moves: &mut Vec<Move>,
    ) {
        let targets = knight_attacks(from) & !own_pieces;
        self.generate_piece_moves(board, from, Piece::Knight, targets, occupied, moves);
    }

    /// Generates sliding piece moves (bishop, rook, queen)
    fn generate_slider_moves<T: BoardQuery>(
        &self,
        board: &T,
        from: Square,
        piece: Piece,
        occupied: BitBoard,
        own_pieces: BitBoard,
        moves: &mut Vec<Move>,
    ) {
        let attacks = match piece {
            Piece::Bishop => bishop_attacks(from, occupied),
            Piece::Rook => rook_attacks(from, occupied),
            Piece::Queen => queen_attacks(from, occupied),
            _ => return,
        };
        let targets = attacks & !own_pieces;
        self.generate_piece_moves(board, from, piece, targets, occupied, moves);
    }

    /// Generates king moves including normal moves (castling handled separately)
    fn generate_king_moves<T: BoardQuery>(
        &self,
        board: &T,
        from: Square,
        occupied: BitBoard,
        own_pieces: BitBoard,
        moves: &mut Vec<Move>,
    ) {
        let targets = king_attacks(from) & !own_pieces;
        self.generate_piece_moves(board, from, Piece::King, targets, occupied, moves);

        if let Some((_, side)) = board.piece_at(from) {
            self.generate_castling_moves(board, from, side, moves);
        }
    }

    /// Generates castling moves if legal
    fn generate_castling_moves<T: BoardQuery>(
        &self,
        board: &T,
        king_square: Square,
        side: Color,
        moves: &mut Vec<Move>,
    ) {
        let opponent = side.opponent();
        let castle_flags = MoveFlags {
            is_castle: true,
            ..MoveFlags::default()
        };

        // Kingside castling
        if board.can_castle_short(side) {
            let (king_start, f_square, g_square) = match side {
                Color::White => (Square::E1, Square::F1, Square::G1),
                Color::Black => (Square::E8, Square::F8, Square::G8),
            };

            let path_clear =
                !board.is_square_occupied(f_square) && !board.is_square_occupied(g_square);
            let path_safe = !board.is_square_attacked(king_start, opponent)
                && !board.is_square_attacked(f_square, opponent)
                && !board.is_square_attacked(g_square, opponent);

            if king_square == king_start && path_clear && path_safe {
                Self::push_move(
                    moves,
                    king_start,
                    g_square,
                    Piece::King,
                    None,
                    castle_flags,
                    None,
                );
            }
        }

        // Queenside castling
        if board.can_castle_long(side) {
            let (king_start, d_square, c_square, b_square) = match side {
                Color::White => (Square::E1, Square::D1, Square::C1, Square::B1),
                Color::Black => (Square::E8, Square::D8, Square::C8, Square::B8),
            };

            let path_clear = !board.is_square_occupied(d_square)
                && !board.is_square_occupied(c_square)
                && !board.is_square_occupied(b_square);
            let path_safe = !board.is_square_attacked(king_start, opponent)
                && !board.is_square_attacked(d_square, opponent)
                && !board.is_square_attacked(c_square, opponent);

            if king_square == king_start && path_clear && path_safe {
                Self::push_move(
                    moves,
                    king_start,
                    c_square,
                    Piece::King,
                    None,
                    castle_flags,
                    None,
                );
            }
        }
    }
}

impl<T: BoardQuery> MoveGen<T> for Generator {
    fn pseudo_legal(&self, board: &T, moves: &mut Vec<Move>) {
        moves.clear();
        moves.reserve(256);

        let side = board.side_to_move();
        let (occupied, own_pieces, opponent_pieces) = self.occupancies(board, side);

        for square in own_pieces.iter() {
            if let Some((piece, _)) = board.piece_at(square) {
                match piece {
                    Piece::Pawn => self.generate_pawn_moves(
                        board,
                        square,
                        side,
                        occupied,
                        opponent_pieces,
                        moves,
                    ),
                    Piece::Knight => {
                        self.generate_knight_moves(board, square, occupied, own_pieces, moves)
                    }
                    Piece::Bishop | Piece::Rook | Piece::Queen => self
                        .generate_slider_moves(board, square, piece, occupied, own_pieces, moves),
                    Piece::King => {
                        self.generate_king_moves(board, square, occupied, own_pieces, moves)
                    }
                }
            }
        }
    }

    fn legal(&self, board: &T, moves: &mut Vec<Move>) {
        self.pseudo_legal(board, moves);
        let side = board.side_to_move();
        let map = PieceMap::from_board(board);
        moves.retain(|mv| is_move_legal(&map, side, mv));
    }

    fn captures(&self, board: &T, moves: &mut Vec<Move>) {
        self.pseudo_legal(board, moves);
        moves.retain(|m| m.is_capture() || m.flags.is_en_passant);
    }

    fn quiet_moves(&self, board: &T, moves: &mut Vec<Move>) {
        self.pseudo_legal(board, moves);
        moves.retain(|m| !m.is_capture() && !m.flags.is_en_passant && !m.flags.is_castle);
    }

    fn checks(&self, board: &T, moves: &mut Vec<Move>) {
        let opponent = board.side_to_move().opponent();
        let king_sq = match board.get_king_square(opponent) {
            Some(sq) => sq,
            None => return, // No king (shouldn't happen)
        };

        let all_occ = board.occupied_by(board.side_to_move()) | board.occupied_by(opponent);

        // Pre-calculate check squares for each piece type
        let knight_checks = knight_attacks(king_sq);
        let bishop_checks = bishop_attacks(king_sq, all_occ);
        let rook_checks = rook_attacks(king_sq, all_occ);
        let queen_checks = bishop_checks | rook_checks;

        // Generate quiet moves and filter for checks
        let mut quiet = Vec::new();
        self.quiet_moves(board, &mut quiet);

        for mv in quiet {
            // Skip pawns - rare to give useful checks in quiescence
            if mv.piece == Piece::Pawn {
                continue;
            }

            // Check if move lands on a check square
            let is_check = match mv.piece {
                Piece::Knight => knight_checks.has(mv.to),
                Piece::Bishop => bishop_checks.has(mv.to),
                Piece::Rook => rook_checks.has(mv.to),
                Piece::Queen => queen_checks.has(mv.to),
                _ => false,
            };

            if is_check {
                moves.push(mv);
            }
        }
    }
}

/// Piece map for legality checking
#[derive(Debug, Clone, Copy)]
struct PieceMap {
    pieces: [[BitBoard; 6]; 2],
    color_occ: [BitBoard; 2],
    all_occ: BitBoard,
}

impl PieceMap {
    fn from_board<T: BoardQuery>(board: &T) -> Self {
        // Copy bitboards directly - O(12) vs O(32) piece_at lookups
        let mut pieces = [[BitBoard::EMPTY; 6]; 2];
        for &piece in &ALL_PIECES {
            pieces[Color::White as usize][piece as usize] = board.piece_bb(piece, Color::White);
            pieces[Color::Black as usize][piece as usize] = board.piece_bb(piece, Color::Black);
        }

        let white_occ = board.occupied_by(Color::White);
        let black_occ = board.occupied_by(Color::Black);

        Self {
            pieces,
            color_occ: [white_occ, black_occ],
            all_occ: white_occ | black_occ,
        }
    }

    fn is_king_attacked(&self, side: Color) -> bool {
        let king_bb = self.pieces[side as usize][Piece::King as usize];
        if let Some(king_sq) = king_bb.to_square() {
            let opponent = side.opponent();
            let their_pieces = &self.pieces[opponent as usize];
            is_square_attacked(king_sq, opponent, self.all_occ, their_pieces)
        } else {
            false
        }
    }

    fn simulate_move(mut self, side: Color, mv: &Move) -> Self {
        let us = side as usize;
        let them = side.opponent() as usize;
        let from_bb = BitBoard::from_square(mv.from);
        let to_bb = BitBoard::from_square(mv.to);

        // Remove piece from origin
        self.pieces[us][mv.piece as usize] &= !from_bb;
        self.color_occ[us] &= !from_bb;
        self.all_occ &= !from_bb;

        // Handle captures
        if mv.flags.is_en_passant {
            if let Some(captured_sq) = mv.to.down(side) {
                let cap_bb = BitBoard::from_square(captured_sq);
                self.pieces[them][Piece::Pawn as usize] &= !cap_bb;
                self.color_occ[them] &= !cap_bb;
                self.all_occ &= !cap_bb;
            }
        } else if let Some(captured) = mv.capture {
            self.pieces[them][captured as usize] &= !to_bb;
            self.color_occ[them] &= !to_bb;
        }

        // Place piece on destination
        let final_piece = mv.promotion.unwrap_or(mv.piece);
        self.pieces[us][final_piece as usize] |= to_bb;
        self.color_occ[us] |= to_bb;
        self.all_occ |= to_bb;

        // Handle castling rook move
        if mv.flags.is_castle {
            let (rook_from, rook_to) = match (side, mv.to) {
                (Color::White, Square::G1) => (Square::H1, Square::F1),
                (Color::White, Square::C1) => (Square::A1, Square::D1),
                (Color::Black, Square::G8) => (Square::H8, Square::F8),
                (Color::Black, Square::C8) => (Square::A8, Square::D8),
                _ => (mv.to, mv.to),
            };

            let rf = BitBoard::from_square(rook_from);
            let rt = BitBoard::from_square(rook_to);
            self.pieces[us][Piece::Rook as usize] &= !rf;
            self.pieces[us][Piece::Rook as usize] |= rt;
            self.color_occ[us] = (self.color_occ[us] & !rf) | rt;
            self.all_occ = (self.all_occ & !rf) | rt;
        }

        self
    }
}

#[inline]
fn is_move_legal(map: &PieceMap, side: Color, mv: &Move) -> bool {
    let next_map = map.simulate_move(side, mv);
    !next_map.is_king_attacked(side)
}

#[cfg(test)]
mod tests {
    use super::*;
    use board::Board;

    #[test]
    fn test_generator_creation() {
        let gen1 = Generator::new();
        let gen2 = Generator::default();

        // Verify both generators are functional
        let board = Board::starting_position().unwrap();
        let mut moves1 = Vec::new();
        let mut moves2 = Vec::new();

        gen1.legal(&board, &mut moves1);
        gen2.legal(&board, &mut moves2);

        assert_eq!(
            moves1.len(),
            20,
            "Starting position should have 20 legal moves"
        );
        assert_eq!(
            moves2.len(),
            20,
            "Default generator should also produce 20 moves"
        );
        assert_eq!(
            moves1, moves2,
            "Both generators should produce identical moves"
        );
    }

    #[test]
    fn test_piece_map_occupancy() {
        let board = Board::starting_position().unwrap();
        let map = PieceMap::from_board(&board);

        // White should have 16 pieces
        assert_eq!(map.color_occ[Color::White as usize].len(), 16);
        // Black should have 16 pieces
        assert_eq!(map.color_occ[Color::Black as usize].len(), 16);
        // Total 32 pieces
        assert_eq!(map.all_occ.len(), 32);
    }
}
