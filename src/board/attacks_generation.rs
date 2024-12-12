use crate::bitboard::Bitboard;
use crate::board::{Board, Color, Piece};
use crate::constants::*;

impl Board {
    pub fn print_attacks(&self, attacks: &Bitboard) {
        for i in 0..BOARD_SIZE {
            if i % BOARD_WIDTH == 0 {
                println!();
            }

            if attacks.is_set(i) {
                print!("X ");
            } else {
                print!(". ");
            }
        }

        println!();
    }
    pub fn generate_pawn_attacks(&self) -> Bitboard {
        let mut attacks = Bitboard::new();
        let pawns = match self.turn {
            Color::White => self.white_pieces.pawns,
            Color::Black => self.black_pieces.pawns,
        };

        let direction = match self.turn {
            Color::White => MOVE_UP,
            Color::Black => MOVE_DOWN,
        };

        for i in 0..BOARD_SIZE {
            if !pawns.is_set(i) {
                continue;
            }

            let from = i;
            let left = from as i32 + direction + MOVE_LEFT;
            let right = from as i32 + direction + MOVE_RIGHT;

            if Board::is_index_in_bounds(left) {
                if (left % BOARD_WIDTH as i32 - (from % BOARD_WIDTH) as i32).abs() > 1 {
                    continue;
                }
                attacks.set_bit(left as usize);
            }
            if Board::is_index_in_bounds(right) {
                if (right % BOARD_WIDTH as i32 - (from % BOARD_WIDTH) as i32).abs() > 1 {
                    continue;
                }
                attacks.set_bit(right as usize);
            }
        }

        attacks
    }

    pub fn generate_knight_attacks(&self) -> Bitboard {
        let knights = match self.turn {
            Color::White => self.white_pieces.knights,
            Color::Black => self.black_pieces.knights,
        };

        let mut attacks = Bitboard::new();

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

                attacks.set_bit(to as usize);
            }
        }

        attacks
    }

    pub fn generate_slider_attacks(&self, directions: &[i32], pieces: Bitboard) -> Bitboard {
        let mut attacks = Bitboard::new();

        for i in 0..BOARD_SIZE {
            if !pieces.is_set(i) {
                continue;
            }

            let from = i;

            for direction in directions.iter() {
                let mut to = from as i32 + direction;
                while Board::is_index_in_bounds(to) {
                    attacks.set_bit(to as usize);

                    if to as usize % BOARD_WIDTH == 0 || to as usize % BOARD_WIDTH == 7 {
                        break;
                    }

                    to += direction;
                }
            }
        }

        attacks
    }

    pub fn generate_bishop_attacks(&self) -> Bitboard {
        let bishops = match self.turn {
            Color::White => self.white_pieces.bishops,
            Color::Black => self.black_pieces.bishops,
        };

        self.generate_slider_attacks(&BISHOP_DIRECTIONS, bishops)
    }

    pub fn generate_rook_attacks(&self) -> Bitboard {
        let rooks = match self.turn {
            Color::White => self.white_pieces.rooks,
            Color::Black => self.black_pieces.rooks,
        };

        self.generate_slider_attacks(&ROOK_DIRECTIONS, rooks)
    }

    pub fn generate_queen_attacks(&self) -> Bitboard {
        let queens = match self.turn {
            Color::White => self.white_pieces.queens,
            Color::Black => self.black_pieces.queens,
        };

        self.generate_slider_attacks(&QUEEN_DIRECTIONS, queens)
    }

    pub fn generate_king_attacks(&self) -> Bitboard {
        let king = match self.turn {
            Color::White => self.white_pieces.king,
            Color::Black => self.black_pieces.king,
        };

        let mut attacks = Bitboard::new();

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

                attacks.set_bit(to as usize);
            }
        }

        attacks
    }

    pub fn update_attacks(&mut self, piece: Piece) {
        let attacks = match piece {
            Piece::Pawn => self.generate_pawn_attacks(),
            Piece::Knight => self.generate_knight_attacks(),
            Piece::Bishop => self.generate_bishop_attacks(),
            Piece::Rook => self.generate_rook_attacks(),
            Piece::Queen => self.generate_queen_attacks(),
            Piece::King => self.generate_king_attacks(),
        };

        match self.turn {
            Color::White => match piece {
                Piece::Pawn => self.white_attacks.pawns = attacks,
                Piece::Knight => self.white_attacks.knights = attacks,
                Piece::Bishop => self.white_attacks.bishops = attacks,
                Piece::Rook => self.white_attacks.rooks = attacks,
                Piece::Queen => self.white_attacks.queens = attacks,
                Piece::King => self.white_attacks.king = attacks,
            },
            Color::Black => match piece {
                Piece::Pawn => self.black_attacks.pawns = attacks,
                Piece::Knight => self.black_attacks.knights = attacks,
                Piece::Bishop => self.black_attacks.bishops = attacks,
                Piece::Rook => self.black_attacks.rooks = attacks,
                Piece::Queen => self.black_attacks.queens = attacks,
                Piece::King => self.black_attacks.king = attacks,
            },
        };
    }
}
