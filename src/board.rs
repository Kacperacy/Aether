mod attacks_generation;
mod move_generation;
mod util;

use crate::bitboard::Bitboard;
use crate::constants::*;

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

impl ToString for Piece {
    fn to_string(&self) -> String {
        match self {
            Piece::Pawn => "p",
            Piece::Knight => "n",
            Piece::Bishop => "b",
            Piece::Rook => "r",
            Piece::Queen => "q",
            Piece::King => "k",
        }
        .to_string()
    }
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
    pub capture: Option<Piece>,
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
            moves: Vec::new(),
        }
    }

    pub fn init() -> Self {
        let mut board = Board::new();
        board.set_fen(STARTING_POSITION);
        board
    }

    fn reset(&mut self) {
        self.white_occupancy = Bitboard::new();
        self.white_attacks = Bitboard::new();
        self.white_pieces = Pieces {
            pawns: Bitboard::new(),
            knights: Bitboard::new(),
            bishops: Bitboard::new(),
            rooks: Bitboard::new(),
            queens: Bitboard::new(),
            king: Bitboard::new(),
        };
        self.black_occupancy = Bitboard::new();
        self.black_attacks = Bitboard::new();
        self.black_pieces = Pieces {
            pawns: Bitboard::new(),
            knights: Bitboard::new(),
            bishops: Bitboard::new(),
            rooks: Bitboard::new(),
            queens: Bitboard::new(),
            king: Bitboard::new(),
        };
        self.turn = Color::White;
        self.castling_rights = CastlingRights {
            white_king_side: true,
            white_queen_side: true,
            black_king_side: true,
            black_queen_side: true,
        };
        self.en_passant_square = None;
        self.halfmove_clock = 0;
        self.fullmove_number = 1;
        self.moves = Vec::new();
    }

    pub fn set_fen(&mut self, fen: &str) {
        self.reset();
        let parts: Vec<&str> = fen.split_whitespace().collect();
        let mut row = 7;
        let mut col = 0;

        for c in parts[0].chars() {
            match c {
                '/' => {
                    row -= 1;
                    col = 0;
                }
                '1'..='8' => {
                    let offset = c.to_digit(10).unwrap() as usize;
                    col += offset;
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

                    self.add_piece(color, piece, row * BOARD_WIDTH + col);
                    col += 1;
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

    fn add_piece(&mut self, color: Color, piece: Piece, index: usize) {
        let bb = Bitboard::from_index(index);

        match color {
            Color::White => {
                self.white_occupancy.set_bit(index);
                // TODO: self.white_attacks = self.white_attacks.or(&bb);
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
                // TODO: self.black_attacks = self.black_attacks.or(&bb);
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
                // TODO: self.white_attacks = self.white_attacks.xor(&bb);
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
                // TODO: self.black_attacks = self.black_attacks.xor(&bb);
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

    pub fn print(&self) {
        println!();
        println!("  A B C D E F G H");
        for row in (0..BOARD_WIDTH).rev() {
            print!("{} ", row + 1);
            for col in 0..BOARD_WIDTH {
                let index = row * BOARD_WIDTH + col;
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
        if mv.en_passant {
            let direction = match mv.color {
                Color::White => MOVE_UP,
                Color::Black => MOVE_DOWN,
            };
            self.remove_piece(mv.color, Piece::Pawn, mv.from);
            self.add_piece(mv.color, Piece::Pawn, mv.to);
            self.remove_piece(
                match mv.color {
                    Color::White => Color::Black,
                    Color::Black => Color::White,
                },
                Piece::Pawn,
                (mv.to as i32 - direction) as usize,
            );
        } else if mv.castling {
            match mv.to {
                2 => {
                    self.remove_piece(mv.color, Piece::King, mv.from);
                    self.add_piece(mv.color, Piece::King, mv.to);
                    self.remove_piece(mv.color, Piece::Rook, 0);
                    self.add_piece(mv.color, Piece::Rook, 3);
                }
                6 => {
                    self.remove_piece(mv.color, Piece::King, mv.from);
                    self.add_piece(mv.color, Piece::King, mv.to);
                    self.remove_piece(mv.color, Piece::Rook, 7);
                    self.add_piece(mv.color, Piece::Rook, 5);
                }
                58 => {
                    self.remove_piece(mv.color, Piece::King, mv.from);
                    self.add_piece(mv.color, Piece::King, mv.to);
                    self.remove_piece(mv.color, Piece::Rook, 56);
                    self.add_piece(mv.color, Piece::Rook, 59);
                }
                62 => {
                    self.remove_piece(mv.color, Piece::King, mv.from);
                    self.add_piece(mv.color, Piece::King, mv.to);
                    self.remove_piece(mv.color, Piece::Rook, 63);
                    self.add_piece(mv.color, Piece::Rook, 61);
                }
                _ => panic!("Invalid castling move"),
            }
        } else if let Some(promotion) = mv.promotion {
            self.remove_piece(mv.color, Piece::Pawn, mv.from);
            self.add_piece(mv.color, promotion, mv.to);
        } else if let Some(capture) = mv.capture {
            self.remove_piece(mv.color, capture, mv.to);
            self.remove_piece(mv.color, mv.piece, mv.from);
            self.add_piece(mv.color, mv.piece, mv.to);
        } else {
            self.remove_piece(mv.color, mv.piece, mv.from);
            self.add_piece(mv.color, mv.piece, mv.to);
        }

        if self.en_passant_square.is_some() {
            self.en_passant_square = None;
        }

        if mv.piece == Piece::Pawn && (mv.to as i32 - mv.from as i32).abs() == 16 {
            let direction = match mv.color {
                Color::White => MOVE_UP,
                Color::Black => MOVE_DOWN,
            };
            self.en_passant_square = Some((mv.to as i32 - direction) as usize);
        }

        self.turn = match self.turn {
            Color::White => Color::Black,
            Color::Black => Color::White,
        };

        self.moves.push(*mv);

        self.halfmove_clock += 1;
        if mv.piece == Piece::Pawn {
            self.halfmove_clock = 0;
        }

        if self.turn == Color::Black {
            self.fullmove_number += 1;
        }
    }
}
