use crate::bitboard::Bitboard;
use crate::constans::STARTING_POSITION;
use std::ptr::null;

pub struct Board {
    pub white_occupancy: Bitboard,
    pub white_attacks: Bitboard,
    pub white_pieces: Pieces,

    pub black_occupancy: Bitboard,
    pub black_attacks: Bitboard,
    pub black_pieces: Pieces,

    pub turn: Color,
    pub castling_rights: CastlingRights,
    pub en_passant_square: Option<usize>,
    pub halfmove_clock: u8,
    pub fullmove_number: u16,

    pub moves: Vec<Move>,
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

#[derive(Debug, Copy, Clone, PartialEq)]
pub struct Move {
    pub from: usize,
    pub to: usize,
    pub piece: Piece,
    pub color: Color,
    pub en_passant: bool,
    pub castling: bool,
    pub promotion: Option<Piece>,
}

impl Default for Board {
    fn default() -> Self {
        Self::new()
    }
}

impl Board {
    pub fn new() -> Self {
        Board {
            white_occupancy: Bitboard::new(),
            white_attacks: Bitboard::new(),
            white_pieces: Pieces {
                pawns: Bitboard::new(),
                knights: Bitboard::new(),
                bishops: Bitboard::new(),
                rooks: Bitboard::new(),
                queens: Bitboard::new(),
                king: Bitboard::new(),
            },
            black_occupancy: Bitboard::new(),
            black_attacks: Bitboard::new(),
            black_pieces: Pieces {
                pawns: Bitboard::new(),
                knights: Bitboard::new(),
                bishops: Bitboard::new(),
                rooks: Bitboard::new(),
                queens: Bitboard::new(),
                king: Bitboard::new(),
            },
            turn: Color::White,
            castling_rights: CastlingRights {
                white_king_side: true,
                white_queen_side: true,
                black_king_side: true,
                black_queen_side: true,
            },
            en_passant_square: None,
            halfmove_clock: 0,
            fullmove_number: 1,
        }
    }

    pub fn init() -> Self {
        let mut board = Board::new();
        board.set_fen(STARTING_POSITION);
        board
    }

    pub fn set_fen(&mut self, fen: &str) {
        let parts: Vec<&str> = fen.split_whitespace().collect();
        let mut rank = 7;
        let mut file = 0;

        for c in parts[0].chars() {
            match c {
                '/' => {
                    rank -= 1;
                    file = 0;
                }
                '1'..='8' => {
                    let offset = c.to_digit(10).unwrap() as usize;
                    file += offset;
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
                        _ => panic!("Invalid FEN"),
                    };

                    self.add_piece(color, piece, rank, file);
                    file += 1;
                }
            }
        }

        self.turn = match parts[1] {
            "w" => Color::White,
            "b" => Color::Black,
            _ => panic!("Invalid FEN"),
        };

        self.castling_rights = CastlingRights {
            white_king_side: parts[2].contains('K'),
            white_queen_side: parts[2].contains('Q'),
            black_king_side: parts[2].contains('k'),
            black_queen_side: parts[2].contains('q'),
        };

        self.en_passant_square = match parts[3] {
            "-" => None,
            s => Some(Board::square_to_index(s)),
        };

        self.halfmove_clock = parts[4].parse().unwrap();
        self.fullmove_number = parts[5].parse().unwrap();
    }

    fn add_piece(&mut self, color: Color, piece: Piece, rank: usize, file: usize) {
        let index = rank * 8 + file;
        let bb = Bitboard::from_index(index);

        match color {
            Color::White => {
                self.white_occupancy.set_bit(index);
                // self.white_attacks = self.white_attacks.or(&bb);
                match piece {
                    Piece::Pawn => self.white_pieces.pawns = self.white_pieces.pawns.or(&bb),
                    Piece::Knight => self.white_pieces.knights = self.white_pieces.knights.or(&bb),
                    Piece::Bishop => self.white_pieces.bishops = self.white_pieces.bishops.or(&bb),
                    Piece::Rook => self.white_pieces.rooks = self.white_pieces.rooks.or(&bb),
                    Piece::Queen => self.white_pieces.queens = self.white_pieces.queens.or(&bb),
                    Piece::King => self.white_pieces.king = self.white_pieces.king.or(&bb),
                }
            }
            Color::Black => {
                self.black_occupancy.set_bit(index);
                // self.black_attacks = self.black_attacks.or(&bb);
                match piece {
                    Piece::Pawn => self.black_pieces.pawns = self.black_pieces.pawns.or(&bb),
                    Piece::Knight => self.black_pieces.knights = self.black_pieces.knights.or(&bb),
                    Piece::Bishop => self.black_pieces.bishops = self.black_pieces.bishops.or(&bb),
                    Piece::Rook => self.black_pieces.rooks = self.black_pieces.rooks.or(&bb),
                    Piece::Queen => self.black_pieces.queens = self.black_pieces.queens.or(&bb),
                    Piece::King => self.black_pieces.king = self.black_pieces.king.or(&bb),
                }
            }
        }
    }

    fn remove_piece(&mut self, color: Color, piece: Piece, index: usize) {
        let bb = Bitboard::from_index(index);

        match color {
            Color::White => {
                self.white_occupancy.clear_bit(index);
                // self.white_attacks = self.white_attacks.xor(&bb);
                match piece {
                    Piece::Pawn => self.white_pieces.pawns = self.white_pieces.pawns.xor(&bb),
                    Piece::Knight => self.white_pieces.knights = self.white_pieces.knights.xor(&bb),
                    Piece::Bishop => self.white_pieces.bishops = self.white_pieces.bishops.xor(&bb),
                    Piece::Rook => self.white_pieces.rooks = self.white_pieces.rooks.xor(&bb),
                    Piece::Queen => self.white_pieces.queens = self.white_pieces.queens.xor(&bb),
                    Piece::King => self.white_pieces.king = self.white_pieces.king.xor(&bb),
                }
            }
            Color::Black => {
                self.black_occupancy.clear_bit(index);
                // self.black_attacks = self.black_attacks.xor(&bb);
                match piece {
                    Piece::Pawn => self.black_pieces.pawns = self.black_pieces.pawns.xor(&bb),
                    Piece::Knight => self.black_pieces.knights = self.black_pieces.knights.xor(&bb),
                    Piece::Bishop => self.black_pieces.bishops = self.black_pieces.bishops.xor(&bb),
                    Piece::Rook => self.black_pieces.rooks = self.black_pieces.rooks.xor(&bb),
                    Piece::Queen => self.black_pieces.queens = self.black_pieces.queens.xor(&bb),
                    Piece::King => self.black_pieces.king = self.black_pieces.king.xor(&bb),
                }
            }
        }
    }

    fn square_to_index(square: &str) -> usize {
        let file = square.chars().nth(0).unwrap() as usize - 'a' as usize;
        let rank = square.chars().nth(1).unwrap() as usize - '1' as usize;
        rank * 8 + file
    }

    fn index_to_square(index: usize) -> String {
        let file = (index % 8) as u8 + b'a';
        let rank = (index / 8) as u8 + b'1';
        format!("{}{}", file as char, rank as char)
    }

    pub fn print(&self) {
        for rank in (0..8).rev() {
            for file in 0..8 {
                let index = rank * 8 + file;
                let square = Board::index_to_square(index);
                let piece = if self.white_pieces.pawns.is_set(index) {
                    'P'
                } else if self.white_pieces.knights.is_set(index) {
                    'N'
                } else if self.white_pieces.bishops.is_set(index) {
                    'B'
                } else if self.white_pieces.rooks.is_set(index) {
                    'R'
                } else if self.white_pieces.queens.is_set(index) {
                    'Q'
                } else if self.white_pieces.king.is_set(index) {
                    'K'
                } else if self.black_pieces.pawns.is_set(index) {
                    'p'
                } else if self.black_pieces.knights.is_set(index) {
                    'n'
                } else if self.black_pieces.bishops.is_set(index) {
                    'b'
                } else if self.black_pieces.rooks.is_set(index) {
                    'r'
                } else if self.black_pieces.queens.is_set(index) {
                    'q'
                } else if self.black_pieces.king.is_set(index) {
                    'k'
                } else {
                    '.'
                };
                print!("{} ", piece);
            }
            println!();
        }
        println!();
    }

    pub fn make_move(&mut self, mv: &Move) {
        if let Some(index) = self.en_passant_square {
            self.en_passant_square = None;
        }

        let from = mv.from;
        let to = mv.to;
        let piece = mv.piece;
        let color = mv.color;

        self.remove_piece(color, piece, from);
        self.add_piece(color, piece, to / 8, to % 8);

        self.turn = match self.turn {
            Color::White => Color::Black,
            Color::Black => Color::White,
        };
    }
}
