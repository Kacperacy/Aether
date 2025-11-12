use crate::MoveGen;
use crate::attacks::attackers_to_square_with_occ;
use crate::magic::{get_bishop_attacks, get_queen_attacks, get_rook_attacks};
use crate::pieces::{get_king_moves, get_knight_moves, get_pawn_attacks, get_pawn_moves};
use aether_types::{ALL_SQUARES, BitBoard, Color, Move, MoveFlags, Piece, Square};
use board::BoardQuery;

/// Minimal move generator (incremental implementation).
#[derive(Debug, Default, Clone, Copy)]
pub struct Generator;

impl Generator {
    pub fn new() -> Self {
        Self
    }

    #[inline]
    fn build_occupancies<T: BoardQuery>(
        &self,
        board: &T,
        stm: Color,
    ) -> (BitBoard, BitBoard, BitBoard) {
        let mut occ = BitBoard::EMPTY;
        let mut own = BitBoard::EMPTY;
        let mut opp = BitBoard::EMPTY;
        for sq in ALL_SQUARES {
            if let Some((_, c)) = board.piece_at(sq) {
                let bb = BitBoard::from_square(sq);
                occ |= bb;
                if c == stm {
                    own |= bb;
                } else {
                    opp |= bb;
                }
            }
        }
        (occ, own, opp)
    }

    #[inline]
    fn push_move(
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
        if let Some(p) = promotion {
            mv = mv.with_promotion(p);
        }
        moves.push(mv);
    }

    #[inline]
    fn gen_pawn_moves<T: BoardQuery>(
        &self,
        board: &T,
        from: Square,
        stm: Color,
        occ: BitBoard,
        opp: BitBoard,
        moves: &mut Vec<Move>,
    ) {
        use Piece::*;
        // Quiet pushes (including double)
        let mut pushes = get_pawn_moves(from, stm, occ);
        while let Some(to) = pushes.next() {
            let is_promo = crate::pieces::pawn::is_promotion_rank(to, stm);
            let delta_rank = (to.rank() as i8) - (from.rank() as i8);
            let is_double = (stm == Color::White && delta_rank == 2)
                || (stm == Color::Black && delta_rank == -2);
            let flags = MoveFlags {
                is_castle: false,
                is_en_passant: false,
                is_double_pawn_push: is_double,
            };
            if is_promo {
                for promo in [Knight, Bishop, Rook, Queen] {
                    Self::push_move(moves, from, to, Pawn, None, flags, Some(promo));
                }
            } else {
                Self::push_move(moves, from, to, Pawn, None, flags, None);
            }
        }

        // Normal captures
        let mut attacks = get_pawn_attacks(from, stm) & opp;
        let capture_flags = MoveFlags {
            is_castle: false,
            is_en_passant: false,
            is_double_pawn_push: false,
        };
        while let Some(to) = attacks.next() {
            let cap_piece = board.piece_at(to).map(|(p, _)| p);
            let is_promo = crate::pieces::pawn::is_promotion_rank(to, stm);
            if is_promo {
                for promo in [Knight, Bishop, Rook, Queen] {
                    Self::push_move(moves, from, to, Pawn, cap_piece, capture_flags, Some(promo));
                }
            } else {
                Self::push_move(moves, from, to, Pawn, cap_piece, capture_flags, None);
            }
        }

        // En passant
        if let Some(ep) = board.en_passant_square() {
            // EP target must be one of pawn attack squares from 'from'. Also EP square is empty.
            if get_pawn_attacks(from, stm).has(ep) {
                let flags = MoveFlags {
                    is_castle: false,
                    is_en_passant: true,
                    is_double_pawn_push: false,
                };
                // Captured piece is always a pawn
                Self::push_move(moves, from, ep, Pawn, Some(Pawn), flags, None);
            }
        }
    }

    #[inline]
    fn gen_knight_moves<T: BoardQuery>(
        &self,
        board: &T,
        from: Square,
        occ: BitBoard,
        own: BitBoard,
        moves: &mut Vec<Move>,
    ) {
        let mut targets = get_knight_moves(from) & !own;
        let flags = MoveFlags {
            is_castle: false,
            is_en_passant: false,
            is_double_pawn_push: false,
        };
        while let Some(to) = targets.next() {
            let cap_piece = if occ.has(to) {
                board.piece_at(to).map(|(p, _)| p)
            } else {
                None
            };
            Self::push_move(moves, from, to, Piece::Knight, cap_piece, flags, None);
        }
    }

    #[inline]
    fn gen_slider_moves<T: BoardQuery>(
        &self,
        board: &T,
        from: Square,
        piece: Piece,
        occ: BitBoard,
        own: BitBoard,
        moves: &mut Vec<Move>,
    ) {
        let attacks = match piece {
            Piece::Bishop => get_bishop_attacks(from, occ),
            Piece::Rook => get_rook_attacks(from, occ),
            Piece::Queen => get_queen_attacks(from, occ),
            _ => BitBoard::EMPTY,
        } & !own;
        let mut targets = attacks;
        let flags = MoveFlags {
            is_castle: false,
            is_en_passant: false,
            is_double_pawn_push: false,
        };
        while let Some(to) = targets.next() {
            let cap_piece = if occ.has(to) {
                board.piece_at(to).map(|(p, _)| p)
            } else {
                None
            };
            Self::push_move(moves, from, to, piece, cap_piece, flags, None);
        }
    }

    #[inline]
    fn gen_king_moves<T: BoardQuery>(
        &self,
        board: &T,
        from: Square,
        occ: BitBoard,
        own: BitBoard,
        moves: &mut Vec<Move>,
    ) {
        let mut targets = get_king_moves(from) & !own;
        let normal_flags = MoveFlags {
            is_castle: false,
            is_en_passant: false,
            is_double_pawn_push: false,
        };
        while let Some(to) = targets.next() {
            let cap_piece = if occ.has(to) {
                board.piece_at(to).map(|(p, _)| p)
            } else {
                None
            };
            Self::push_move(moves, from, to, Piece::King, cap_piece, normal_flags, None);
        }

        // Castling
        if let Some((_, color)) = board.piece_at(from) {
            let opp = color.opponent();
            let castle_flags = MoveFlags {
                is_castle: true,
                is_en_passant: false,
                is_double_pawn_push: false,
            };
            // Short castle (king side)
            if board.can_castle_short(color) {
                let (e, f, g) = match color {
                    Color::White => (Square::E1, Square::F1, Square::G1),
                    Color::Black => (Square::E8, Square::F8, Square::G8),
                };
                if from == e
                    && !board.is_square_occupied(f)
                    && !board.is_square_occupied(g)
                    && !board.is_square_attacked(e, opp)
                    && !board.is_square_attacked(f, opp)
                    && !board.is_square_attacked(g, opp)
                {
                    Self::push_move(moves, from, g, Piece::King, None, castle_flags, None);
                }
            }

            // Long castle (queen side)
            if board.can_castle_long(color) {
                let (e, d, c, b) = match color {
                    Color::White => (Square::E1, Square::D1, Square::C1, Square::B1),
                    Color::Black => (Square::E8, Square::D8, Square::C8, Square::B8),
                };
                if from == e
                    && !board.is_square_occupied(d)
                    && !board.is_square_occupied(c)
                    && !board.is_square_occupied(b)
                    && !board.is_square_attacked(e, opp)
                    && !board.is_square_attacked(d, opp)
                    && !board.is_square_attacked(c, opp)
                {
                    Self::push_move(moves, from, c, Piece::King, None, castle_flags, None);
                }
            }
        }
    }
}

impl<T: BoardQuery> MoveGen<T> for Generator {
    fn pseudo_legal(&self, board: &T, moves: &mut Vec<Move>) {
        moves.clear();
        let stm = board.side_to_move();
        let (occ, own, opp) = self.build_occupancies(board, stm);

        for sq in ALL_SQUARES {
            if let Some((piece, color)) = board.piece_at(sq) {
                if color != stm {
                    continue;
                }
                match piece {
                    Piece::Pawn => self.gen_pawn_moves(board, sq, stm, occ, opp, moves),
                    Piece::Knight => self.gen_knight_moves(board, sq, occ, own, moves),
                    Piece::Bishop | Piece::Rook | Piece::Queen => {
                        self.gen_slider_moves(board, sq, piece, occ, own, moves)
                    }
                    Piece::King => self.gen_king_moves(board, sq, occ, own, moves),
                }
            }
        }
    }

    fn legal(&self, board: &T, moves: &mut Vec<Move>) {
        // Generate pseudo-legal moves then filter by king safety using a fast local simulation.
        self.pseudo_legal(board, moves);
        let stm = board.side_to_move();
        let map = PieceMap::from_board(board);
        moves.retain(|m| is_move_legal_in_map(&map, stm, m));
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

#[derive(Clone, Copy, Debug)]
struct PieceMap {
    pieces: [[BitBoard; 6]; 2],
    color_occ: [BitBoard; 2],
    occ: BitBoard,
}

impl PieceMap {
    fn from_board<T: BoardQuery>(board: &T) -> Self {
        let mut pieces = [[BitBoard::EMPTY; 6]; 2];
        for sq in ALL_SQUARES {
            if let Some((p, c)) = board.piece_at(sq) {
                pieces[c as usize][p as usize] |= BitBoard::from_square(sq);
            }
        }
        let w = pieces[Color::White as usize];
        let b = pieces[Color::Black as usize];
        let color_occ = [
            w[0] | w[1] | w[2] | w[3] | w[4] | w[5],
            b[0] | b[1] | b[2] | b[3] | b[4] | b[5],
        ];
        let occ = color_occ[0] | color_occ[1];
        Self {
            pieces,
            color_occ,
            occ,
        }
    }
}

#[inline]
fn attackers_to_square_in_map(map: &PieceMap, sq: Square, color: Color) -> BitBoard {
    let their = &map.pieces[color as usize];
    attackers_to_square_with_occ(sq, color, map.occ, their)
}

#[inline]
fn is_king_attacked_in_map(map: &PieceMap, color: Color) -> bool {
    if let Some(king_sq) = map.pieces[color as usize][Piece::King as usize].to_square() {
        let opp = color.opponent();
        !attackers_to_square_in_map(map, king_sq, opp).is_empty()
    } else {
        false
    }
}

fn simulate_move_on_map(mut map: PieceMap, stm: Color, mv: &Move) -> PieceMap {
    let us = stm as usize;
    let them = stm.opponent() as usize;

    let from_bb = BitBoard::from_square(mv.from);
    let to_bb = BitBoard::from_square(mv.to);

    // Remove moving piece from origin
    map.pieces[us][mv.piece as usize] &= !from_bb;
    map.color_occ[us] &= !from_bb;
    map.occ &= !from_bb;

    // Handle captures
    if mv.flags.is_en_passant {
        // Captured pawn is behind the target square relative to mover
        if let Some(captured_sq) = mv.to.down(stm) {
            let cap_bb = BitBoard::from_square(captured_sq);
            map.pieces[them][Piece::Pawn as usize] &= !cap_bb;
            map.color_occ[them] &= !cap_bb;
            map.occ &= !cap_bb;
        }
    } else if let Some(cap) = mv.capture {
        map.pieces[them][cap as usize] &= !to_bb;
        // Opponent piece disappears from 'to' square (king move will occupy it next).
        map.color_occ[them] &= !to_bb;
        // Do not clear map.occ at 'to' here, since the moving piece will occupy it below.
    }

    // Place piece on destination (promotion or normal)
    if let Some(promo) = mv.promotion {
        map.pieces[us][promo as usize] |= to_bb;
    } else {
        map.pieces[us][mv.piece as usize] |= to_bb;
    }
    map.color_occ[us] |= to_bb;
    map.occ |= to_bb;

    // Handle castling rook move
    if mv.flags.is_castle {
        let (rook_from, rook_to) = match (stm, mv.to) {
            (Color::White, Square::G1) => (Square::H1, Square::F1),
            (Color::White, Square::C1) => (Square::A1, Square::D1),
            (Color::Black, Square::G8) => (Square::H8, Square::F8),
            (Color::Black, Square::C8) => (Square::A8, Square::D8),
            _ => (mv.to, mv.to), // no-op fallback
        };
        let rf = BitBoard::from_square(rook_from);
        let rt = BitBoard::from_square(rook_to);
        map.pieces[us][Piece::Rook as usize] &= !rf;
        map.pieces[us][Piece::Rook as usize] |= rt;
        map.color_occ[us] &= !rf;
        map.color_occ[us] |= rt;
        map.occ &= !rf;
        map.occ |= rt;
    }

    map
}

#[inline]
fn is_move_legal_in_map(orig_map: &PieceMap, stm: Color, mv: &Move) -> bool {
    let next_map = simulate_move_on_map(*orig_map, stm, mv);
    !is_king_attacked_in_map(&next_map, stm)
}
