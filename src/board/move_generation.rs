use crate::bitboard::Bitboard;
use crate::board::{Board, Color, Move, Piece};
use crate::constants::*;

impl Board {
    pub fn generate_possible_moves(&self) -> Vec<Move> {
        let mut moves = Vec::new();

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
        let pawns = match self.turn {
            Color::White => self.white_pieces.pawns,
            Color::Black => self.black_pieces.pawns,
        };

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
                moves.push(Move {
                    from,
                    to: left,
                    piece: Piece::Pawn,
                    color: self.turn,
                    en_passant: false,
                    castling: false,
                    promotion: None,
                    capture: self.piece_at(left),
                });
            }
            if self.is_enemy(right) {
                moves.push(Move {
                    from,
                    to: right,
                    piece: Piece::Pawn,
                    color: self.turn,
                    en_passant: false,
                    castling: false,
                    promotion: None,
                    capture: self.piece_at(right),
                });
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
                        moves.push(Move {
                            from,
                            to: to as usize,
                            piece,
                            color: self.turn,
                            en_passant: false,
                            castling: false,
                            promotion: None,
                            capture: self.piece_at(to as usize),
                        });
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
        let bishops = match self.turn {
            Color::White => self.white_pieces.bishops,
            Color::Black => self.black_pieces.bishops,
        };

        // TODO: Validate check

        self.generate_slider_moves(&BISHOP_DIRECTIONS, bishops, Piece::Bishop)
    }

    pub fn generate_knight_moves(&self) -> Vec<Move> {
        let mut moves = Vec::new();
        let knights = match self.turn {
            Color::White => self.white_pieces.knights,
            Color::Black => self.black_pieces.knights,
        };

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
                    moves.push(Move {
                        from,
                        to: to as usize,
                        piece: Piece::Knight,
                        color: self.turn,
                        en_passant: false,
                        castling: false,
                        promotion: None,
                        capture: self.piece_at(to as usize),
                    });
                }
            }
        }

        moves
    }

    pub fn generate_rook_moves(&self) -> Vec<Move> {
        let rooks = match self.turn {
            Color::White => self.white_pieces.rooks,
            Color::Black => self.black_pieces.rooks,
        };

        // TODO: Validate check

        self.generate_slider_moves(&ROOK_DIRECTIONS, rooks, Piece::Rook)
    }

    pub fn generate_queen_moves(&self) -> Vec<Move> {
        let queens = match self.turn {
            Color::White => self.white_pieces.queens,
            Color::Black => self.black_pieces.queens,
        };

        // TODO: Validate check

        self.generate_slider_moves(&QUEEN_DIRECTIONS, queens, Piece::Queen)
    }

    pub fn generate_king_moves(&self) -> Vec<Move> {
        let mut moves = Vec::new();
        let king = match self.turn {
            Color::White => self.white_pieces.king,
            Color::Black => self.black_pieces.king,
        };

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
                    moves.push(Move {
                        from,
                        to: to as usize,
                        piece: Piece::King,
                        color: self.turn,
                        en_passant: false,
                        castling: false,
                        promotion: None,
                        capture: self.piece_at(to as usize),
                    });
                }
            }

            // CASTLING
            if self.turn == Color::White {
                if self.castling_rights.white_king_side {
                    if self.is_square_empty(61) && self.is_square_empty(62) {
                        moves.push(Move {
                            from,
                            to: 62,
                            piece: Piece::King,
                            color: self.turn,
                            en_passant: false,
                            castling: true,
                            promotion: None,
                            capture: None,
                        });
                    }
                }
                if self.castling_rights.white_queen_side {
                    if self.is_square_empty(59)
                        && self.is_square_empty(58)
                        && self.is_square_empty(57)
                    {
                        moves.push(Move {
                            from,
                            to: 58,
                            piece: Piece::King,
                            color: self.turn,
                            en_passant: false,
                            castling: true,
                            promotion: None,
                            capture: None,
                        });
                    }
                }
            } else {
                if self.castling_rights.black_king_side {
                    if self.is_square_empty(5) && self.is_square_empty(6) {
                        moves.push(Move {
                            from,
                            to: 6,
                            piece: Piece::King,
                            color: self.turn,
                            en_passant: false,
                            castling: true,
                            promotion: None,
                            capture: None,
                        });
                    }
                }
                if self.castling_rights.black_queen_side {
                    if self.is_square_empty(3) && self.is_square_empty(2) && self.is_square_empty(1)
                    {
                        moves.push(Move {
                            from,
                            to: 2,
                            piece: Piece::King,
                            color: self.turn,
                            en_passant: false,
                            castling: true,
                            promotion: None,
                            capture: None,
                        });
                    }
                }
            }
        }

        moves
    }
}
