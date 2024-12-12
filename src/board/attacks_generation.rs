use crate::bitboard::Bitboard;
use crate::board::{Board, Color};
use crate::constants::*;

impl Board {
    fn generate_pawn_attacks(&self) -> Bitboard {
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

    fn generate_knight_attacks(&self) -> Bitboard {
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

    fn generate_slider_attacks(&self, directions: &[i32], pieces: Bitboard) -> Bitboard {
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

                    to += direction;

                    if to as usize % BOARD_WIDTH == 0 || to as usize % BOARD_WIDTH == 7 {
                        break;
                    }
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
}
