use crate::bitboard::Bitboard;
use crate::board::{Board, Color, Move, Piece};
use crate::constants::*;

impl Board {
    pub fn generate_possible_moves(&self) -> Vec<Move> {
        let mut moves = Vec::new();

        // 50 moves draw
        if self.halfmove_clock >= 100 {
            return moves;
        }

        moves.extend(&self.generate_pawn_moves());
        moves.extend(&self.generate_bishop_moves());
        moves.extend(&self.generate_knight_moves());
        moves.extend(&self.generate_rook_moves());
        moves.extend(&self.generate_queen_moves());
        moves.extend(&self.generate_king_moves());

        println!("Possible {:?} moves:", moves.len());
        moves.iter().for_each(|m: &Move| {
            let mut move_str = Board::index_to_square(m.from) + &Board::index_to_square(m.to);
            if let Some(promotion) = m.promotion {
                move_str.push_str(&promotion.to_string());
            }
            print!("{:?} ", move_str);
        });

        moves
    }

    pub fn generate_pawn_moves(&self) -> Vec<Move> {
        let mut moves = Vec::new();
        let pawns = self.pieces[self.turn as usize][Piece::Pawn as usize];

        // TODO: Validate check

        for i in 0..BOARD_SIZE {
            if !pawns.is_set(i) {
                continue;
            }

            let direction = match self.turn {
                Color::White => MOVE_UP,
                Color::Black => MOVE_DOWN,
            };

            let from = i;
            let possible_to = i as i32 + direction;

            if !Board::is_index_in_bounds(possible_to) {
                continue;
            }

            let to = possible_to as usize;
            let left = (to as i32 + MOVE_LEFT) as usize;
            let right = (to as i32 + MOVE_RIGHT) as usize;

            // DOUBLE PUSH
            if (ROW_2.is_set(from) && self.turn == Color::White)
                || (ROW_7.is_set(from) && self.turn == Color::Black)
            {
                let double = to as i32 + direction;
                if self.is_square_empty(to) && self.is_square_empty(double as usize) {
                    moves.push(Move {
                        from,
                        to: double as usize,
                        piece: Piece::Pawn,
                        color: self.turn,
                        en_passant: false,
                        castling: false,
                        promotion: None,
                        capture: None,
                    });
                }
            }

            // EN PASSANT
            if let Some(ep) = self.en_passant_square {
                if left == ep {
                    moves.push(Move {
                        from,
                        to: left,
                        piece: Piece::Pawn,
                        color: self.turn,
                        en_passant: true,
                        castling: false,
                        promotion: None,
                        capture: Some(Piece::Pawn),
                    });
                }
                if right == ep {
                    moves.push(Move {
                        from,
                        to: right,
                        piece: Piece::Pawn,
                        color: self.turn,
                        en_passant: true,
                        castling: false,
                        promotion: None,
                        capture: Some(Piece::Pawn),
                    });
                }
            }

            // CAPTURES
            if self.is_enemy(left) {
                if let Some(piece_at) = self.piece_at(left) {
                    moves.push(Move {
                        from,
                        to: left,
                        piece: Piece::Pawn,
                        color: self.turn,
                        en_passant: false,
                        castling: false,
                        promotion: None,
                        capture: Some(piece_at.piece),
                    });
                }
            }
            if self.is_enemy(right) {
                if let Some(piece_at) = self.piece_at(right) {
                    moves.push(Move {
                        from,
                        to: right,
                        piece: Piece::Pawn,
                        color: self.turn,
                        en_passant: false,
                        castling: false,
                        promotion: None,
                        capture: Some(piece_at.piece),
                    });
                }
            }

            // PROMOTION
            if (self.turn == Color::White && ROW_7.is_set(from) && self.is_square_empty(to))
                || (self.turn == Color::Black && ROW_2.is_set(from) && self.is_square_empty(to))
            {
                moves.push(Move {
                    from,
                    to,
                    piece: Piece::Pawn,
                    color: self.turn,
                    en_passant: false,
                    castling: false,
                    promotion: Some(Piece::Queen),
                    capture: None,
                });
                moves.push(Move {
                    from,
                    to,
                    piece: Piece::Pawn,
                    color: self.turn,
                    en_passant: false,
                    castling: false,
                    promotion: Some(Piece::Rook),
                    capture: None,
                });
                moves.push(Move {
                    from,
                    to,
                    piece: Piece::Pawn,
                    color: self.turn,
                    en_passant: false,
                    castling: false,
                    promotion: Some(Piece::Bishop),
                    capture: None,
                });
                moves.push(Move {
                    from,
                    to,
                    piece: Piece::Pawn,
                    color: self.turn,
                    en_passant: false,
                    castling: false,
                    promotion: Some(Piece::Knight),
                    capture: None,
                });
            }

            // NORMAL PUSH
            if self.is_square_empty(to) {
                moves.push(Move {
                    from,
                    to,
                    piece: Piece::Pawn,
                    color: self.turn,
                    en_passant: false,
                    castling: false,
                    promotion: None,
                    capture: None,
                });
            }
        }

        moves
    }

    fn generate_slider_moves(
        &self,
        directions: &[i32],
        pieces: Bitboard,
        piece: Piece,
    ) -> Vec<Move> {
        let mut moves = Vec::new();

        for i in 0..BOARD_SIZE {
            if !pieces.is_set(i) {
                continue;
            }

            let from = i;

            for direction in directions.iter() {
                let mut to = from as i32 + direction;
                while Board::is_index_in_bounds(to) {
                    if self.is_square_empty(to as usize) {
                        moves.push(Move {
                            from,
                            to: to as usize,
                            piece,
                            color: self.turn,
                            en_passant: false,
                            castling: false,
                            promotion: None,
                            capture: None,
                        });
                    } else if self.is_enemy(to as usize) {
                        if let Some(piece_at) = self.piece_at(to as usize) {
                            moves.push(Move {
                                from,
                                to: to as usize,
                                piece,
                                color: self.turn,
                                en_passant: false,
                                castling: false,
                                promotion: None,
                                capture: Some(piece_at.piece),
                            });
                        }
                        break;
                    } else {
                        break;
                    }

                    if to as usize % BOARD_WIDTH == 0 || to as usize % BOARD_WIDTH == 7 {
                        break;
                    }

                    to += direction;
                }
            }
        }

        moves
    }
    pub fn generate_bishop_moves(&self) -> Vec<Move> {
        let bishops = self.pieces[self.turn as usize][Piece::Bishop as usize];

        // TODO: Validate check

        self.generate_slider_moves(&BISHOP_DIRECTIONS, bishops, Piece::Bishop)
    }

    pub fn generate_knight_moves(&self) -> Vec<Move> {
        let mut moves = Vec::new();
        let knights = self.pieces[self.turn as usize][Piece::Knight as usize];

        // TODO: Validate check

        for i in 0..BOARD_SIZE {
            if !knights.is_set(i) {
                continue;
            }

            let from = i;
            for direction in KNIGHT_DIRECTIONS.iter() {
                let to = from as i32 + direction;
                if !Board::is_index_in_bounds(to) {
                    continue;
                }

                if (to % BOARD_WIDTH as i32 - (from % BOARD_WIDTH) as i32).abs() > 2 {
                    continue;
                }

                if self.is_square_empty(to as usize) {
                    moves.push(Move {
                        from,
                        to: to as usize,
                        piece: Piece::Knight,
                        color: self.turn,
                        en_passant: false,
                        castling: false,
                        promotion: None,
                        capture: None,
                    });
                } else if self.is_enemy(to as usize) {
                    if let Some(piece_at) = self.piece_at(to as usize) {
                        moves.push(Move {
                            from,
                            to: to as usize,
                            piece: Piece::Knight,
                            color: self.turn,
                            en_passant: false,
                            castling: false,
                            promotion: None,
                            capture: Some(piece_at.piece),
                        });
                    }
                }
            }
        }

        moves
    }

    pub fn generate_rook_moves(&self) -> Vec<Move> {
        let rooks = self.pieces[self.turn as usize][Piece::Rook as usize];

        // TODO: Validate check

        self.generate_slider_moves(&ROOK_DIRECTIONS, rooks, Piece::Rook)
    }

    pub fn generate_queen_moves(&self) -> Vec<Move> {
        let queens = self.pieces[self.turn as usize][Piece::Queen as usize];

        // TODO: Validate check

        self.generate_slider_moves(&QUEEN_DIRECTIONS, queens, Piece::Queen)
    }

    pub fn generate_king_moves(&self) -> Vec<Move> {
        let mut moves = Vec::new();
        let king = self.pieces[self.turn as usize][Piece::King as usize];

        // TODO: Validate check

        for i in 0..BOARD_SIZE {
            if !king.is_set(i) {
                continue;
            }

            let from = i;
            for direction in KING_DIRECTIONS.iter() {
                let to = from as i32 + direction;
                if !Board::is_index_in_bounds(to) {
                    continue;
                }

                if (to % BOARD_WIDTH as i32 - (from % BOARD_WIDTH) as i32).abs() > 1 {
                    continue;
                }

                if self.is_square_empty(to as usize) {
                    moves.push(Move {
                        from,
                        to: to as usize,
                        piece: Piece::King,
                        color: self.turn,
                        en_passant: false,
                        castling: false,
                        promotion: None,
                        capture: None,
                    });
                } else if self.is_enemy(to as usize) {
                    if let Some(piece_at) = self.piece_at(to as usize) {
                        moves.push(Move {
                            from,
                            to: to as usize,
                            piece: Piece::King,
                            color: self.turn,
                            en_passant: false,
                            castling: false,
                            promotion: None,
                            capture: Some(piece_at.piece),
                        });
                    }
                }
            }

            let castle_index = match self.turn {
                Color::White => 0,
                Color::Black => 2,
            };

            // CASTLING
            if self.can_castle(self.turn, true) {
                moves.push(Move {
                    from,
                    to: CASTLING_RIGHTS_SQUARES[castle_index][1],
                    piece: Piece::King,
                    color: self.turn,
                    en_passant: false,
                    castling: true,
                    promotion: None,
                    capture: None,
                });
            }
            if self.can_castle(self.turn, false) {
                moves.push(Move {
                    from,
                    to: CASTLING_RIGHTS_SQUARES[castle_index + 1][1],
                    piece: Piece::King,
                    color: self.turn,
                    en_passant: false,
                    castling: true,
                    promotion: None,
                    capture: None,
                });
            }
        }

        moves
    }
}
