use crate::{
    magic::{get_bishop_attacks, get_queen_attacks, get_rook_attacks},
    pieces::{get_king_moves, get_knight_moves, get_pawn_attacks, get_pawn_moves},
};
use aether_types::{BitBoard, BoardQuery, Color, Move, MoveFlags, MoveGen, Piece, Square};

#[derive(Default)]
pub struct Generator;

impl Generator {
    /// Convenience: add every set bit in `bb` as a move from `from`
    #[inline(always)]
    fn push_moves<T: BoardQuery>(
        from: Square,
        mut bb: BitBoard,
        list: &mut Vec<Move>,
        piece: Piece,
        board: &T,
    ) {
        while let Some(to) = bb.pop_lsb() {
            let capture = board.piece_at(to).map(|(p, _)| p);
            list.push(Move {
                from,
                to,
                piece,
                capture,
                promotion: None,
                flags: MoveFlags {
                    is_castle: false,
                    is_en_passant: false,
                    is_double_pawn_push: false,
                },
            });
        }
    }

    fn generate_castles<T: BoardQuery>(&self, board: &T, moves: &mut Vec<Move>) {
        let side = board.side_to_move();
        let king_sq = board.get_king_square(side).unwrap();

        // squares that must be empty and not attacked for each castle
        let (short_empty, short_safe, long_empty, long_safe) = match side {
            Color::White => (
                [Square::F1, Square::G1],
                [Square::E1, Square::F1, Square::G1],
                [Square::B1, Square::C1, Square::D1],
                [Square::E1, Square::D1, Square::C1],
            ),
            Color::Black => (
                [Square::F8, Square::G8],
                [Square::E8, Square::F8, Square::G8],
                [Square::B8, Square::C8, Square::D8],
                [Square::E8, Square::D8, Square::C8],
            ),
        };

        // short castle
        if board.can_castle_short(side) {
            if short_empty.iter().all(|&s| !board.is_square_occupied(s))
                && short_safe
                    .iter()
                    .all(|&s| !board.is_square_attacked(s, side.opponent()))
            {
                moves.push(Move {
                    from: king_sq,
                    to: short_empty[1], // G-file
                    piece: Piece::King,
                    capture: None,
                    promotion: None,
                    flags: MoveFlags {
                        is_castle: true,
                        is_en_passant: false,
                        is_double_pawn_push: false,
                    },
                });
            }
        }

        // long castle
        if board.can_castle_long(side) {
            if long_empty.iter().all(|&s| !board.is_square_occupied(s))
                && long_safe
                    .iter()
                    .all(|&s| !board.is_square_attacked(s, side.opponent()))
            {
                moves.push(Move {
                    from: king_sq,
                    to: long_empty[2], // C-file
                    piece: Piece::King,
                    capture: None,
                    promotion: None,
                    flags: MoveFlags {
                        is_castle: true,
                        is_en_passant: false,
                        is_double_pawn_push: false,
                    },
                });
            }
        }
    }

    fn generate_en_passant<T: BoardQuery>(&self, board: &T, moves: &mut Vec<Move>) {
        let ep_sq = match board.en_passant_square() {
            Some(sq) => sq,
            None => return,
        };
        let side = board.side_to_move();

        // Find pawns that can capture en passant
        let candidates = get_pawn_attacks(ep_sq, side.opponent());

        // Check each adjacent square for our pawns
        for from in candidates.into_iter() {
            if let Some((piece, color)) = board.piece_at(from) {
                if piece == Piece::Pawn && color == side {
                    moves.push(Move {
                        from,
                        to: ep_sq,
                        piece: Piece::Pawn,
                        capture: Some(Piece::Pawn),
                        promotion: None,
                        flags: MoveFlags {
                            is_castle: false,
                            is_en_passant: true,
                            is_double_pawn_push: false,
                        },
                    });
                }
            }
        }
    }

    fn generate_promotions(
        &self,
        from: Square,
        to: Square,
        list: &mut Vec<Move>,
        capture: Option<Piece>,
    ) {
        use Piece::*;
        for piece in [Queen, Rook, Bishop, Knight] {
            list.push(Move {
                from,
                to,
                piece: Piece::Pawn,
                capture,
                promotion: Some(piece),
                flags: MoveFlags {
                    is_castle: false,
                    is_en_passant: false,
                    is_double_pawn_push: false,
                },
            });
        }
    }

    fn get_occupied_squares<T: BoardQuery>(&self, board: &T) -> BitBoard {
        let mut occupied = BitBoard::EMPTY;
        for square in Square::all() {
            if board.is_square_occupied(*square) {
                occupied |= square.bitboard();
            }
        }
        occupied
    }

    fn get_pieces_of_type<T: BoardQuery>(&self, board: &T, piece: Piece, color: Color) -> BitBoard {
        let mut pieces = BitBoard::EMPTY;
        for square in Square::all() {
            if let Some((p, c)) = board.piece_at(*square) {
                if p == piece && c == color {
                    pieces |= square.bitboard();
                }
            }
        }
        pieces
    }
}

impl<T: BoardQuery> MoveGen<T> for Generator {
    fn pseudo_legal(&self, board: &T, list: &mut Vec<Move>) {
        list.clear();
        let side = board.side_to_move();
        let occ = self.get_occupied_squares(board);

        // 1. Pawns
        let pawns = self.get_pieces_of_type(board, Piece::Pawn, side);
        for from in pawns.into_iter() {
            // pushes
            let pushes = get_pawn_moves(from, side, occ);
            let rank_promo = side.pawn_promotion_rank();

            for to in pushes.into_iter() {
                let is_double = (from.rank() == side.pawn_start_rank())
                    && (to.rank()
                        != side
                            .pawn_start_rank()
                            .offset(side.forward_direction())
                            .unwrap());

                if to.rank() == rank_promo {
                    self.generate_promotions(from, to, list, None);
                } else {
                    list.push(Move {
                        from,
                        to,
                        piece: Piece::Pawn,
                        capture: None,
                        promotion: None,
                        flags: MoveFlags {
                            is_castle: false,
                            is_en_passant: false,
                            is_double_pawn_push: is_double,
                        },
                    });
                }
            }

            // captures
            let attacks = get_pawn_attacks(from, side);
            for to in attacks.into_iter() {
                if let Some((captured_piece, captured_color)) = board.piece_at(to) {
                    if captured_color != side {
                        if to.rank() == rank_promo {
                            self.generate_promotions(from, to, list, Some(captured_piece));
                        } else {
                            list.push(Move {
                                from,
                                to,
                                piece: Piece::Pawn,
                                capture: Some(captured_piece),
                                promotion: None,
                                flags: MoveFlags {
                                    is_castle: false,
                                    is_en_passant: false,
                                    is_double_pawn_push: false,
                                },
                            });
                        }
                    }
                }
            }
        }

        // 2. Knights
        let knights = self.get_pieces_of_type(board, Piece::Knight, side);
        for from in knights.into_iter() {
            let moves_bb = get_knight_moves(from);
            for to in moves_bb.into_iter() {
                match board.piece_at(to) {
                    None => {
                        list.push(Move::new(from, to).with_piece(Piece::Knight));
                    }
                    Some((captured_piece, captured_color)) if captured_color != side => {
                        let mut mv = Move::new(from, to).with_piece(Piece::Knight);
                        mv.capture = Some(captured_piece);
                        list.push(mv);
                    }
                    _ => {} // Our own piece, skip
                }
            }
        }

        // 3. Bishops
        let bishops = self.get_pieces_of_type(board, Piece::Bishop, side);
        for from in bishops.into_iter() {
            let moves_bb = get_bishop_attacks(from, occ);
            for to in moves_bb.into_iter() {
                match board.piece_at(to) {
                    None => {
                        list.push(Move::new(from, to).with_piece(Piece::Bishop));
                    }
                    Some((captured_piece, captured_color)) if captured_color != side => {
                        let mut mv = Move::new(from, to).with_piece(Piece::Bishop);
                        mv.capture = Some(captured_piece);
                        list.push(mv);
                    }
                    _ => {} // Our own piece, skip
                }
            }
        }

        // 4. Rooks
        let rooks = self.get_pieces_of_type(board, Piece::Rook, side);
        for from in rooks.into_iter() {
            let moves_bb = get_rook_attacks(from, occ);
            for to in moves_bb.into_iter() {
                match board.piece_at(to) {
                    None => {
                        list.push(Move::new(from, to).with_piece(Piece::Rook));
                    }
                    Some((captured_piece, captured_color)) if captured_color != side => {
                        let mut mv = Move::new(from, to).with_piece(Piece::Rook);
                        mv.capture = Some(captured_piece);
                        list.push(mv);
                    }
                    _ => {} // Our own piece, skip
                }
            }
        }

        // 5. Queens
        let queens = self.get_pieces_of_type(board, Piece::Queen, side);
        for from in queens.into_iter() {
            let moves_bb = get_queen_attacks(from, occ);
            for to in moves_bb.into_iter() {
                match board.piece_at(to) {
                    None => {
                        list.push(Move::new(from, to).with_piece(Piece::Queen));
                    }
                    Some((captured_piece, captured_color)) if captured_color != side => {
                        let mut mv = Move::new(from, to).with_piece(Piece::Queen);
                        mv.capture = Some(captured_piece);
                        list.push(mv);
                    }
                    _ => {} // Our own piece, skip
                }
            }
        }

        // 6. King non-castling
        let king_sq = board.get_king_square(side).unwrap();
        let king_moves = get_king_moves(king_sq);
        for to in king_moves.into_iter() {
            match board.piece_at(to) {
                None => {
                    list.push(Move::new(king_sq, to).with_piece(Piece::King));
                }
                Some((captured_piece, captured_color)) if captured_color != side => {
                    let mut mv = Move::new(king_sq, to).with_piece(Piece::King);
                    mv.capture = Some(captured_piece);
                    list.push(mv);
                }
                _ => {} // Our own piece, skip
            }
        }

        // 7. Castling
        self.generate_castles(board, list);

        // 8. En-passant
        self.generate_en_passant(board, list);
    }

    fn legal(&self, board: &T, list: &mut Vec<Move>) {
        // Stub - would need board.make_move() and board.is_in_check()
        self.pseudo_legal(board, list);
    }

    fn captures(&self, board: &T, list: &mut Vec<Move>) {
        self.pseudo_legal(board, list);
        list.retain(|m| m.capture.is_some() || m.flags.is_en_passant);
    }

    fn quiet_moves(&self, board: &T, list: &mut Vec<Move>) {
        self.pseudo_legal(board, list);
        list.retain(|m| m.capture.is_none() && !m.flags.is_en_passant);
    }
}
