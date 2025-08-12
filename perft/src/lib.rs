//! Perft crate
//!
//! Responsibilities:
//! - Provide correctness and performance tests for move generation.
//! - Offer simple perft counting utilities and states that implement `BoardQuery`.
//! - Used in development and CI to validate `movegen` implementations.
//!
//! This crate should remain test-focused and avoid engine/search coupling beyond
//! consuming the public APIs of `aether-types`, `board`, and `movegen`.

use aether_types::{BitBoard, BoardQuery, Color, Move, MoveGen, Piece, Square};
use movegen::{Generator, attacks::attackers_to_square_with_occ};

#[derive(Clone, Debug)]
pub struct PerftState {
    pieces: [[BitBoard; 6]; 2],
    color_occ: [BitBoard; 2],
    occ: BitBoard,
    side: Color,
    ep: Option<Square>,
    castle_short: [bool; 2],
    castle_long: [bool; 2],
}

impl PerftState {
    pub fn from_board<T: BoardQuery>(board: &T) -> Self {
        let mut pieces = [[BitBoard::EMPTY; 6]; 2];
        for &sq in Square::all().iter() {
            if let Some((p, c)) = board.piece_at(sq) {
                pieces[c as usize][p as usize] |= BitBoard::from_square(sq);
            }
        }
        let mut s = Self {
            pieces,
            color_occ: [BitBoard::EMPTY; 2],
            occ: BitBoard::EMPTY,
            side: board.side_to_move(),
            ep: board.en_passant_square(),
            castle_short: [
                board.can_castle_short(Color::White),
                board.can_castle_short(Color::Black),
            ],
            castle_long: [
                board.can_castle_long(Color::White),
                board.can_castle_long(Color::Black),
            ],
        };
        s.recompute_occupancies();
        s
    }

    fn recompute_occupancies(&mut self) {
        let w = self.pieces[0];
        let b = self.pieces[1];
        self.color_occ[0] = w[0] | w[1] | w[2] | w[3] | w[4] | w[5];
        self.color_occ[1] = b[0] | b[1] | b[2] | b[3] | b[4] | b[5];
        self.occ = self.color_occ[0] | self.color_occ[1];
    }

    #[allow(dead_code)]
    fn remove_piece_at(&mut self, sq: Square) -> Option<(Piece, Color)> {
        let bb = BitBoard::from_square(sq);
        for color in [Color::White, Color::Black] {
            for piece in Piece::all() {
                let c = color as usize;
                let p = piece as usize;
                if !(self.pieces[c][p] & bb).is_empty() {
                    self.pieces[c][p] &= !bb;
                    return Some((piece, color));
                }
            }
        }
        None
    }

    #[allow(dead_code)]
    fn place_piece(&mut self, sq: Square, piece: Piece, color: Color) {
        self.pieces[color as usize][piece as usize] |= BitBoard::from_square(sq);
    }

    fn apply_move(&mut self, mv: &Move) {
        let us = self.side as usize;
        let them = self.side.opponent() as usize;
        let from_bb = BitBoard::from_square(mv.from);
        let to_bb = BitBoard::from_square(mv.to);

        // Remove moving piece from origin (incremental occupancy updates)
        self.pieces[us][mv.piece as usize] &= !from_bb;
        self.color_occ[us] &= !from_bb;
        self.occ &= !from_bb;

        // Handle captures
        if mv.flags.is_en_passant {
            // Captured pawn is behind the target square relative to mover
            if let Some(captured_sq) = mv.to.down(self.side) {
                let cap_bb = BitBoard::from_square(captured_sq);
                self.pieces[them][Piece::Pawn as usize] &= !cap_bb;
                self.color_occ[them] &= !cap_bb;
                self.occ &= !cap_bb;
            }
        } else if let Some(cap) = mv.capture {
            self.pieces[them][cap as usize] &= !to_bb;
            // Opponent piece disappears from 'to' square
            self.color_occ[them] &= !to_bb;
            // Do not clear self.occ at 'to' here, the moving piece will occupy it below
        }

        // Place piece on destination (promotion or normal)
        if let Some(promo) = mv.promotion {
            self.pieces[us][promo as usize] |= to_bb;
        } else {
            self.pieces[us][mv.piece as usize] |= to_bb;
        }
        self.color_occ[us] |= to_bb;
        self.occ |= to_bb;

        // Handle castling rook move
        if mv.flags.is_castle {
            let (rook_from, rook_to) = match (self.side, mv.to) {
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
            self.color_occ[us] &= !rf;
            self.color_occ[us] |= rt;
            self.occ &= !rf;
            self.occ |= rt;
        }

        // Update en passant square
        if mv.flags.is_double_pawn_push {
            // The EP target square is the square jumped over
            if let Some(ep_sq) = mv.from.up(self.side) {
                self.ep = Some(ep_sq);
            } else {
                self.ep = None;
            }
        } else {
            self.ep = None;
        }

        // Update castling rights
        // Own king moved: lose both rights
        if mv.piece == Piece::King {
            self.castle_short[us] = false;
            self.castle_long[us] = false;
        }
        // Own rook moved from original squares
        if mv.piece == Piece::Rook {
            match (self.side, mv.from) {
                (Color::White, Square::H1) => self.castle_short[us] = false,
                (Color::White, Square::A1) => self.castle_long[us] = false,
                (Color::Black, Square::H8) => self.castle_short[us] = false,
                (Color::Black, Square::A8) => self.castle_long[us] = false,
                _ => {}
            }
        }
        // If captured opponent rook on its original squares, update their rights
        if !mv.flags.is_en_passant {
            if let Some(cap) = mv.capture {
                if cap == Piece::Rook {
                    match (self.side, mv.to) {
                        (Color::White, Square::H8) => self.castle_short[them] = false,
                        (Color::White, Square::A8) => self.castle_long[them] = false,
                        (Color::Black, Square::H1) => self.castle_short[them] = false,
                        (Color::Black, Square::A1) => self.castle_long[them] = false,
                        _ => {}
                    }
                }
            }
        }

        // Switch side
        self.side = self.side.opponent();
    }
}

impl BoardQuery for PerftState {
    fn piece_at(&self, sq: Square) -> Option<(Piece, Color)> {
        if !self.occ.has(sq) {
            return None;
        }
        let color = if self.color_occ[Color::White as usize].has(sq) {
            Color::White
        } else {
            Color::Black
        };
        for piece in Piece::all() {
            if self.pieces[color as usize][piece as usize].has(sq) {
                return Some((piece, color));
            }
        }
        None
    }

    fn is_square_occupied(&self, square: Square) -> bool {
        self.occ.has(square)
    }

    fn is_square_attacked(&self, square: Square, by_color: Color) -> bool {
        let occ = self.occ;
        let their = &self.pieces[by_color as usize];
        !attackers_to_square_with_occ(square, by_color, occ, their).is_empty()
    }

    fn get_king_square(&self, color: Color) -> Option<Square> {
        self.pieces[color as usize][Piece::King as usize].to_square()
    }

    fn can_castle_short(&self, color: Color) -> bool {
        self.castle_short[color as usize]
    }

    fn can_castle_long(&self, color: Color) -> bool {
        self.castle_long[color as usize]
    }

    fn en_passant_square(&self) -> Option<Square> {
        self.ep
    }

    fn side_to_move(&self) -> Color {
        self.side
    }
}

pub fn perft_count<T: BoardQuery>(board: &T, depth: u32) -> u64 {
    let state = PerftState::from_board(board);
    perft_count_state(&state, depth)
}

fn perft_count_state(state: &PerftState, depth: u32) -> u64 {
    if depth == 0 {
        return 1;
    }
    let generator = Generator::new();
    let mut moves: Vec<Move> = Vec::with_capacity(256);
    generator.legal(state, &mut moves);
    if depth == 1 {
        return moves.len() as u64;
    }
    let mut nodes = 0u64;
    for mv in &moves {
        let mut next = state.clone();
        next.apply_move(mv);
        nodes += perft_count_state(&next, depth - 1);
    }
    nodes
}
