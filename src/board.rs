use crate::bitboard::Bitboard;

pub struct Board {
    pub white_pawns: Bitboard,
    pub white_knights: Bitboard,
    pub white_bishops: Bitboard,
    pub white_rooks: Bitboard,
    pub white_queens: Bitboard,
    pub white_king: Bitboard,

    pub black_pawns: Bitboard,
    pub black_knights: Bitboard,
    pub black_bishops: Bitboard,
    pub black_rooks: Bitboard,
    pub black_queens: Bitboard,
    pub black_king: Bitboard,

    pub turn: Color,
    pub castling_rights: CastlingRights,
    pub en_passant_square: Option<usize>,
    pub halfmove_clock: u8,
    pub fullmove_number: u16,
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
        let white_pawns =
            Bitboard(0b0000000000000000000000000000000000000000000000001111111100000000);
        let white_knights =
            Bitboard(0b0000000000000000000000000000000000000000000000000000000001000010);
        let white_bishops =
            Bitboard(0b0000000000000000000000000000000000000000000000000000000000100100);
        let white_rooks =
            Bitboard(0b0000000000000000000000000000000000000000000000000000000010000001);
        let white_queens =
            Bitboard(0b0000000000000000000000000000000000000000000000000000000000001000);
        let white_king =
            Bitboard(0b0000000000000000000000000000000000000000000000000000000000010000);

        let black_pawns =
            Bitboard(0b0000000011111111000000000000000000000000000000000000000000000000);
        let black_knights =
            Bitboard(0b0100001000000000000000000000000000000000000000000000000000000000);
        let black_bishops =
            Bitboard(0b0010010000000000000000000000000000000000000000000000000000000000);
        let black_rooks =
            Bitboard(0b1000000100000000000000000000000000000000000000000000000000000000);
        let black_queens =
            Bitboard(0b0000100000000000000000000000000000000000000000000000000000000000);
        let black_king =
            Bitboard(0b0001000000000000000000000000000000000000000000000000000000000000);

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
            white_pawns,
            white_knights,
            white_bishops,
            white_rooks,
            white_queens,
            white_king,
            black_pawns,
            black_knights,
            black_bishops,
            black_rooks,
            black_queens,
            black_king,
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
            (Color::White, Piece::Pawn) => self.white_pawns,
            (Color::White, Piece::Knight) => self.white_knights,
            (Color::White, Piece::Bishop) => self.white_bishops,
            (Color::White, Piece::Rook) => self.white_rooks,
            (Color::White, Piece::Queen) => self.white_queens,
            (Color::White, Piece::King) => self.white_king,
            (Color::Black, Piece::Pawn) => self.black_pawns,
            (Color::Black, Piece::Knight) => self.black_knights,
            (Color::Black, Piece::Bishop) => self.black_bishops,
            (Color::Black, Piece::Rook) => self.black_rooks,
            (Color::Black, Piece::Queen) => self.black_queens,
            (Color::Black, Piece::King) => self.black_king,
        }
    }

    /// Places a piece on the board at the specified square index
    fn place_piece(&mut self, color: Color, piece: Piece, index: usize) {
        match (color, piece) {
            (Color::White, Piece::Pawn) => self.white_pawns.set_bit(index),
            (Color::White, Piece::Knight) => self.white_knights.set_bit(index),
            (Color::White, Piece::Bishop) => self.white_bishops.set_bit(index),
            (Color::White, Piece::Rook) => self.white_rooks.set_bit(index),
            (Color::White, Piece::Queen) => self.white_queens.set_bit(index),
            (Color::White, Piece::King) => self.white_king.set_bit(index),
            (Color::Black, Piece::Pawn) => self.black_pawns.set_bit(index),
            (Color::Black, Piece::Knight) => self.black_knights.set_bit(index),
            (Color::Black, Piece::Bishop) => self.black_bishops.set_bit(index),
            (Color::Black, Piece::Rook) => self.black_rooks.set_bit(index),
            (Color::Black, Piece::Queen) => self.black_queens.set_bit(index),
            (Color::Black, Piece::King) => self.black_king.set_bit(index),
        };
    }
}
