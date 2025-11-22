use crate::MoveGen;
use aether_types::{
    ALL_SQUARES, BitBoard, Color, Move, MoveFlags, PROMOTION_PIECES, Piece, Square, bishop_attacks,
    is_promotion_rank, is_square_attacked, king_attacks, knight_attacks, pawn_attacks, pawn_moves,
    queen_attacks, rook_attacks,
};
use board::{Board, BoardQuery};

#[derive(Debug, Default, Clone, Copy)]
pub struct Generator;

impl Generator {
    #[inline]
    pub fn new() -> Self {
        Self
    }

    /// Build occupancy bitboards for move generation
    #[inline]
    fn occupancies<T: BoardQuery>(&self, board: &T, side: Color) -> (BitBoard, BitBoard, BitBoard) {
        let mut all = BitBoard::EMPTY;
        let mut own = BitBoard::EMPTY;
        let mut opp = BitBoard::EMPTY;

        for sq in ALL_SQUARES {
            if let Some((_, color)) = board.piece_at(sq) {
                let bb = BitBoard::from_square(sq);
                all |= bb;
                if color == side {
                    own |= bb;
                } else {
                    opp |= bb;
                }
            }
        }

        (all, own, opp)
    }

    /// Pushes a move into the move list
    #[inline(always)]
    fn push(
        moves: &mut Vec<Move>,
        from: Square,
        to: Square,
        piece: Piece,
        capture: Option<Piece>,
        flags: MoveFlags,
        promotion: Option<Piece>,
    ) {
        let mut mv = Move::new(from, to).with_piece(piece).with_flags(flags);

        if let Some(cap) = capture {
            mv = mv.with_capture(cap);
        }

        if let Some(prom) = promotion {
            mv = mv.with_promotion(prom);
        }

        moves.push(mv);
    }

    /// Generate pawn moves
    fn gen_pawn_moves<T: BoardQuery>(
        &self,
        board: &T,
        from: Square,
        side: Color,
        occupied: BitBoard,
        opponent: BitBoard,
        moves: &mut Vec<Move>,
    ) {
        let normal_flags = MoveFlags::default();
        let double_flags = MoveFlags {
            is_double_pawn_push: true,
            ..MoveFlags::default()
        };

        let mut pushes = pawn_moves(from, side, occupied);
        while let Some(to) = pushes.next() {
            let is_promo = is_promotion_rank(to, side);
            let delta = (to.rank() as i8).abs_diff(from.rank() as i8);

            let flags = if delta == 2 {
                double_flags
            } else {
                normal_flags
            };

            if is_promo {
                for &promo in &PROMOTION_PIECES {
                    Self::push(moves, from, to, Piece::Pawn, None, flags, Some(promo));
                }
            } else {
                Self::push(moves, from, to, Piece::Pawn, None, flags, None);
            }
        }

        let mut attacks = pawn_attacks(from, side) & opponent;
        while let Some(to) = attacks.next() {
            let captured = board.piece_at(to).map(|(p, _)| p);
            let is_promo = is_promotion_rank(to, side);

            if is_promo {
                for &promo in &PROMOTION_PIECES {
                    Self::push(
                        moves,
                        from,
                        to,
                        Piece::Pawn,
                        captured,
                        normal_flags,
                        Some(promo),
                    );
                }
            } else {
                if captured.is_some() {
                    Self::push(moves, from, to, Piece::Pawn, captured, normal_flags, None);
                }
            }
        }

        if let Some(ep_square) = board.en_passant_square() {
            if pawn_attacks(from, side).has(ep_square) {
                let ep_flags = MoveFlags {
                    is_en_passant: true,
                    ..MoveFlags::default()
                };

                Self::push(
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

    /// Generate knight moves
    fn gen_knight_moves<T: BoardQuery>(
        &self,
        board: &T,
        from: Square,
        occupied: BitBoard,
        own: BitBoard,
        moves: &mut Vec<Move>,
    ) {
        let mut targets = knight_attacks(from) & !own;
        let flags = MoveFlags::default();

        while let Some(to) = targets.next() {
            let captured = if occupied.has(to) {
                board.piece_at(to).map(|(p, _)| p)
            } else {
                None
            };

            Self::push(moves, from, to, Piece::Knight, captured, flags, None);
        }
    }

    /// Generate slider moves (Bishop, Rook, Queen)
    fn gen_slider_moves<T: BoardQuery>(
        &self,
        board: &T,
        from: Square,
        piece: Piece,
        occupied: BitBoard,
        own: BitBoard,
        moves: &mut Vec<Move>,
    ) {
        let attacks = match piece {
            Piece::Bishop => bishop_attacks(from, occupied),
            Piece::Rook => rook_attacks(from, occupied),
            Piece::Queen => queen_attacks(from, occupied),
            _ => return,
        };

        let mut targets = attacks & !own;
        let flags = MoveFlags::default();

        while let Some(to) = targets.next() {
            let captured = if occupied.has(to) {
                board.piece_at(to).map(|(p, _)| p)
            } else {
                None
            };

            Self::push(moves, from, to, piece, captured, flags, None);
        }
    }

    /// Generate king moves
    fn gen_king_moves<T: BoardQuery>(
        &self,
        board: &T,
        from: Square,
        occupied: BitBoard,
        own: BitBoard,
        moves: &mut Vec<Move>,
    ) {
        let mut targets = king_attacks(from) & !own;
        let flags = MoveFlags::default();

        while let Some(to) = targets.next() {
            let captured = if occupied.has(to) {
                board.piece_at(to).map(|(p, _)| p)
            } else {
                None
            };

            Self::push(moves, from, to, Piece::King, captured, flags, None);
        }

        if let Some((_, side)) = board.piece_at(from) {
            self.gen_castling(board, from, side, moves);
        }
    }

    /// Generate castling moves
    fn gen_castling<T: BoardQuery>(
        &self,
        board: &T,
        king_sq: Square,
        side: Color,
        moves: &mut Vec<Move>,
    ) {
        let opponent = side.opponent();
        let castle_flags = MoveFlags {
            is_castle: true,
            ..MoveFlags::default()
        };

        if board.can_castle_short(side) {
            let (e, f, g) = match side {
                Color::White => (Square::E1, Square::F1, Square::G1),
                Color::Black => (Square::E8, Square::F8, Square::G8),
            };

            if king_sq == e
                && !board.is_square_occupied(f)
                && !board.is_square_occupied(g)
                && !board.is_square_attacked(e, opponent)
                && !board.is_square_attacked(f, opponent)
                && !board.is_square_attacked(g, opponent)
            {
                Self::push(moves, e, g, Piece::King, None, castle_flags, None);
            }
        }

        if board.can_castle_long(side) {
            let (e, d, c, b) = match side {
                Color::White => (Square::E1, Square::D1, Square::C1, Square::B1),
                Color::Black => (Square::E8, Square::D8, Square::C8, Square::B8),
            };

            if king_sq == e
                && !board.is_square_occupied(d)
                && !board.is_square_occupied(c)
                && !board.is_square_occupied(b)
                && !board.is_square_attacked(e, opponent)
                && !board.is_square_attacked(d, opponent)
                && !board.is_square_attacked(c, opponent)
            {
                Self::push(moves, e, c, Piece::King, None, castle_flags, None);
            }
        }
    }
}

impl<T: BoardQuery> MoveGen<T> for Generator {
    fn pseudo_legal(&self, board: &T, moves: &mut Vec<Move>) {
        moves.clear();
        let side = board.side_to_move();
        let (occupied, own, opponent) = self.occupancies(board, side);

        for sq in ALL_SQUARES {
            if let Some((piece, color)) = board.piece_at(sq) {
                if color != side {
                    continue;
                }

                match piece {
                    Piece::Pawn => self.gen_pawn_moves(board, sq, side, occupied, opponent, moves),
                    Piece::Knight => self.gen_knight_moves(board, sq, occupied, own, moves),
                    Piece::Bishop | Piece::Rook | Piece::Queen => {
                        self.gen_slider_moves(board, sq, piece, occupied, own, moves)
                    }
                    Piece::King => self.gen_king_moves(board, sq, occupied, own, moves),
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
        let mut pieces = [[BitBoard::EMPTY; 6]; 2];

        for sq in ALL_SQUARES {
            if let Some((piece, color)) = board.piece_at(sq) {
                pieces[color as usize][piece as usize] |= BitBoard::from_square(sq);
            }
        }

        let white_occ = pieces[Color::White as usize]
            .iter()
            .fold(BitBoard::EMPTY, |a, b| a | *b);
        let black_occ = pieces[Color::Black as usize]
            .iter()
            .fold(BitBoard::EMPTY, |a, b| a | *b);

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

    // Note: Proper tests would require a BoardQuery implementation
    // These are structural tests only

    #[test]
    fn test_generator_creation() {
        let gen1 = Generator::new();
        let _gen2 = Generator::default();
        assert!(true); // Just verify it compiles
    }

    #[test]
    fn test_piece_map_occupancy() {
        // Would need a mock board to properly test
        assert!(true);
    }
}
