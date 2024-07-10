use std::ops::RangeBounds;

use crate::bitboard::Bitboard;

pub struct Board {
    pub white_occupancy: Bitboard,
    pub white_pieces: Pieces,

    pub black_occupancy: Bitboard,
    pub black_pieces: Pieces,

    pub turn: Color,
    pub castling_rights: CastlingRights,
    pub en_passant_square: Option<usize>,
    pub halfmove_clock: u8,
    pub fullmove_number: u16,
}

pub struct Pieces {
    pub pawns: Bitboard,
    pub knights: Bitboard,
    pub bishops: Bitboard,
    pub rooks: Bitboard,
    pub queens: Bitboard,
    pub king: Bitboard,
}

#[derive(Debug, Copy, Clone, PartialEq)]
pub enum Color {
    White,
    Black,
}

#[derive(Debug, Copy, Clone, PartialEq)]
pub enum Piece {
    Pawn,
    Knight,
    Bishop,
    Rook,
    Queen,
    King,
}

#[derive(Debug, Copy, Clone, PartialEq)]
pub struct CastlingRights {
    pub white_king_side: bool,
    pub white_queen_side: bool,
    pub black_king_side: bool,
    pub black_queen_side: bool,
}

impl Board {
    /// Creates a new chessboard with default values
    pub fn new() -> Self {
        let white_occupancy = Bitboard::new();

        let white_pieces = Pieces {
            pawns: Bitboard(0b0000000000000000000000000000000000000000000000001111111100000000),
            knights: Bitboard(0b0000000000000000000000000000000000000000000000000000000001000010),
            bishops: Bitboard(0b0000000000000000000000000000000000000000000000000000000000100100),
            rooks: Bitboard(0b0000000000000000000000000000000000000000000000000000000010000001),
            queens: Bitboard(0b0000000000000000000000000000000000000000000000000000000000001000),
            king: Bitboard(0b0000000000000000000000000000000000000000000000000000000000010000),
        };

        let black_occupancy = Bitboard::new();

        let black_pieces = Pieces {
            pawns: Bitboard(0b0000000011111111000000000000000000000000000000000000000000000000),
            knights: Bitboard(0b0100001000000000000000000000000000000000000000000000000000000000),
            bishops: Bitboard(0b0010010000000000000000000000000000000000000000000000000000000000),
            rooks: Bitboard(0b1000000100000000000000000000000000000000000000000000000000000000),
            queens: Bitboard(0b0000100000000000000000000000000000000000000000000000000000000000),
            king: Bitboard(0b0001000000000000000000000000000000000000000000000000000000000000),
        };

        let turn = Color::White;

        let castling_rights = CastlingRights {
            white_king_side: true,
            white_queen_side: true,
            black_king_side: true,
            black_queen_side: true,
        };

        let en_passant_square = None;
        let halfmove_clock = 0;
        let fullmove_number = 1;

        Board {
            white_pieces,
            white_occupancy,
            black_pieces,
            black_occupancy,
            turn,
            castling_rights,
            en_passant_square,
            halfmove_clock,
            fullmove_number,
        }
    }

    /// Creates a chessboard from a FEN string
    pub fn from_fen(fen: &str) -> Option<Self> {
        let mut board = Board::new();
        let mut squares = fen.split_whitespace();

        let piece_placement = squares.next()?;
        let mut rank = 7;
        let mut file = 0;
        for c in piece_placement.chars() {
            match c {
                '/' => {
                    if file != 8 {
                        return None;
                    }
                    rank -= 1;
                    file = 0;
                }
                '1'..='8' => {
                    let empty_squares = c.to_digit(10).unwrap() as usize;
                    file += empty_squares;
                }
                _ => {
                    let color = if c.is_uppercase() {
                        Color::White
                    } else {
                        Color::Black
                    };
                    let piece = match c.to_ascii_lowercase() {
                        'p' => Piece::Pawn,
                        'n' => Piece::Knight,
                        'b' => Piece::Bishop,
                        'r' => Piece::Rook,
                        'q' => Piece::Queen,
                        'k' => Piece::King,
                        _ => return None,
                    };
                    board.place_piece(color, piece, rank * 8 + file);
                    file += 1;
                }
            }
        }

        let turn = squares.next()?;
        board.turn = match turn {
            "w" => Color::White,
            "b" => Color::Black,
            _ => return None,
        };

        let castling_rights = squares.next()?;
        board.castling_rights = CastlingRights {
            white_king_side: castling_rights.contains('K'),
            white_queen_side: castling_rights.contains('Q'),
            black_king_side: castling_rights.contains('k'),
            black_queen_side: castling_rights.contains('q'),
        };

        let en_passant_square = squares.next()?;
        board.en_passant_square = match en_passant_square {
            "-" => None,
            square => Some(Board::square_to_index(square)),
        };

        let halfmove_clock = squares.next()?.parse().ok()?;
        board.halfmove_clock = halfmove_clock;

        let fullmove_number = squares.next()?.parse().ok()?;
        board.fullmove_number = fullmove_number;

        Some(board)
    }

    /// Converts a square representation to an index
    fn square_to_index(square: &str) -> usize {
        let file = square.chars().nth(0).unwrap() as usize - 'a' as usize;
        let rank = square.chars().nth(1).unwrap().to_digit(10).unwrap() as usize - 1;

        8 * (7 - rank) + file
    }

    /// Gets the bitboard for a specific piece and color
    fn get_piece_bitboard(&self, color: Color, piece: Piece) -> Bitboard {
        match (color, piece) {
            (Color::White, Piece::Pawn) => self.white_pieces.pawns,
            (Color::White, Piece::Knight) => self.white_pieces.knights,
            (Color::White, Piece::Bishop) => self.white_pieces.bishops,
            (Color::White, Piece::Rook) => self.white_pieces.rooks,
            (Color::White, Piece::Queen) => self.white_pieces.queens,
            (Color::White, Piece::King) => self.white_pieces.king,
            (Color::Black, Piece::Pawn) => self.black_pieces.pawns,
            (Color::Black, Piece::Knight) => self.black_pieces.knights,
            (Color::Black, Piece::Bishop) => self.black_pieces.bishops,
            (Color::Black, Piece::Rook) => self.black_pieces.rooks,
            (Color::Black, Piece::Queen) => self.black_pieces.queens,
            (Color::Black, Piece::King) => self.black_pieces.king,
        }
    }

    /// Places a piece on the board at the specified square index
    fn place_piece(&mut self, color: Color, piece: Piece, index: usize) {
        match color {
            Color::White => self.white_occupancy.set_bit(index),
            Color::Black => self.black_occupancy.set_bit(index),
        };

        match (color, piece) {
            (Color::White, Piece::Pawn) => self.white_pieces.pawns.set_bit(index),
            (Color::White, Piece::Knight) => self.white_pieces.knights.set_bit(index),
            (Color::White, Piece::Bishop) => self.white_pieces.bishops.set_bit(index),
            (Color::White, Piece::Rook) => self.white_pieces.rooks.set_bit(index),
            (Color::White, Piece::Queen) => self.white_pieces.queens.set_bit(index),
            (Color::White, Piece::King) => self.white_pieces.king.set_bit(index),
            (Color::Black, Piece::Pawn) => self.black_pieces.pawns.set_bit(index),
            (Color::Black, Piece::Knight) => self.black_pieces.knights.set_bit(index),
            (Color::Black, Piece::Bishop) => self.black_pieces.bishops.set_bit(index),
            (Color::Black, Piece::Rook) => self.black_pieces.rooks.set_bit(index),
            (Color::Black, Piece::Queen) => self.black_pieces.queens.set_bit(index),
            (Color::Black, Piece::King) => self.black_pieces.king.set_bit(index),
        };
    }

    /// Check if pawn position is starting position
    fn is_pawn_starting_position(&self, color: Color, position: usize) -> bool {
        match color {
            Color::White => (8..16).contains(&position),
            Color::Black => (48..56).contains(&position),
        }
    }

    /// Check is square is empty
    fn is_square_empty(&self, index: usize, occupancy: Bitboard) -> bool {
        !occupancy.is_set(index)
    }

    /// Chceck if enemy piece is on square
    fn is_square_enemy(&self, color: Color, position: usize) -> bool {
        match color {
            Color::White => self.black_occupancy.is_set(position),
            Color::Black => self.white_occupancy.is_set(position),
        }
    }

    /// Generate all possible moves at the specified square index
    pub fn generate_moves(&self, index: usize) -> Vec<usize> {
        let mut moves = Vec::new();

        let occupancy = self.white_occupancy.or(&self.black_occupancy);

        self.generate_pawn_moves(&mut moves, occupancy, index);

        moves
    }

    /// Generate possible pawn moves at the specified square index
    fn generate_pawn_moves(&self, moves: &mut Vec<usize>, occupancy: Bitboard, position: usize) {
        let direction = match self.turn {
            Color::White => 8,
            Color::Black => -8,
        };

        let single_forward = position as i8 + direction;
        if !occupancy.is_set(single_forward as usize) {
            moves.push(single_forward as usize);
        }

        if self.is_pawn_starting_position(self.turn, position) {
            let double_forward = single_forward + direction;
            if self.is_square_empty(double_forward as usize, occupancy) {
                moves.push(double_forward as usize);
            }
        }

        let left_capture = single_forward - 1;
        let right_capture = single_forward + 1;
        if self.is_square_enemy(self.turn, left_capture as usize) {
            moves.push(left_capture as usize);
        }
        if self.is_square_enemy(self.turn, right_capture as usize) {
            moves.push(right_capture as usize);
        }
    }
}
