mod attacks_generation;
mod move_generation;
mod utils;
mod zobrist;

use crate::bitboard::Bitboard;
use crate::board::zobrist::ZOBRIST;
use crate::constants::*;
use std::fmt::Display;

pub struct Board {
    pub occupancy: [Bitboard; 2],
    pub attacks: [[Bitboard; 6]; 2],
    pub pieces: [[Bitboard; 6]; 2],

    pub turn: Color,
    pub castling_rights: CastlingRights,
    pub en_passant_square: Option<usize>,
    pub halfmove_clock: u8,
    pub fullmove_number: u16,

    pub moves: Vec<Move>,

    pub current_zobrist: u64,
}

#[derive(Debug, Copy, Clone, PartialEq)]
pub enum Color {
    White = 0,
    Black = 1,
}

impl Color {
    pub fn opposite(&self) -> Color {
        match self {
            Color::White => Color::Black,
            Color::Black => Color::White,
        }
    }
}

#[derive(Debug, Copy, Clone, PartialEq)]
pub enum Piece {
    Pawn = 0,
    Knight = 1,
    Bishop = 2,
    Rook = 3,
    Queen = 4,
    King = 5,
}

impl Display for Piece {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let str = match self {
            Piece::Pawn => "p",
            Piece::Knight => "n",
            Piece::Bishop => "b",
            Piece::Rook => "r",
            Piece::Queen => "q",
            Piece::King => "k",
        }
        .to_string();
        write!(f, "{}", str)
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
            occupancy: [Bitboard::new(); 2],
            attacks: [[Bitboard::new(); 6]; 2],
            pieces: [[Bitboard::new(); 6]; 2],
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
            current_zobrist: 0,
        }
    }

    pub fn init() -> Self {
        let mut board = Board::new();
        board.set_fen(STARTING_POSITION);
        board
    }

    fn reset(&mut self) {
        self.occupancy = [Bitboard::new(); 2];
        self.attacks = [[Bitboard::new(); 6]; 2];
        self.pieces = [[Bitboard::new(); 6]; 2];
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
        self.current_zobrist = 0;
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

        self.current_zobrist = ZOBRIST.hash(&self);
    }

    fn add_piece(&mut self, color: Color, piece: Piece, index: usize) {
        let bb = Bitboard::from_index(index);

        self.occupancy[color as usize] = self.occupancy[color as usize].or(&bb);
        self.pieces[color as usize][piece as usize] =
            self.pieces[color as usize][piece as usize].or(&bb);
    }

    fn remove_piece(&mut self, color: Color, piece: Piece, index: usize) {
        let bb = Bitboard::from_index(index);

        self.occupancy[color as usize] = self.occupancy[color as usize].and(&bb.not());
        self.pieces[color as usize][piece as usize] =
            self.pieces[color as usize][piece as usize].and(&bb.not());
    }

    fn move_piece(&mut self, color: Color, piece: Piece, from: usize, to: usize) {
        self.remove_piece(color, piece, from);
        self.add_piece(color, piece, to);
    }

    pub fn print(&self) {
        println!();
        println!("  A B C D E F G H");
        for row in (0..BOARD_WIDTH).rev() {
            print!("{} ", row + 1);
            for col in 0..BOARD_WIDTH {
                let index = row * BOARD_WIDTH + col;

                let piece = match self.piece_at(index) {
                    Some(p) => p.piece.to_string(),
                    None => ".".to_string(),
                };

                print!("{} ", piece);
            }
            println!();
        }
        println!();
    }

    pub fn make_move(&mut self, mv: &Move) {
        self.move_piece(mv.color, mv.piece, mv.from, mv.to);

        if mv.capture.is_some() {
            let mut direction = 0;

            if mv.en_passant {
                direction = match mv.color {
                    Color::White => MOVE_DOWN,
                    Color::Black => MOVE_UP,
                };
            }

            self.remove_piece(
                mv.color.opposite(),
                mv.capture.unwrap(),
                (mv.to as i32 - direction) as usize,
            );
        }

        // Update en passant square
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

        if mv.piece == Piece::King {
            match mv.color {
                Color::White => {
                    self.castling_rights.white_king_side = false;
                    self.castling_rights.white_queen_side = false;
                }
                Color::Black => {
                    self.castling_rights.black_king_side = false;
                    self.castling_rights.black_queen_side = false;
                }
            }
        } else if mv.piece == Piece::Rook {
            match mv.color {
                Color::White => {
                    if mv.from == 0 {
                        self.castling_rights.white_queen_side = false;
                    } else if mv.from == 7 {
                        self.castling_rights.white_king_side = false;
                    }
                }
                Color::Black => {
                    if mv.from == 56 {
                        self.castling_rights.black_queen_side = false;
                    } else if mv.from == 63 {
                        self.castling_rights.black_king_side = false;
                    }
                }
            }
        }

        // Update turn and move counters
        self.moves.push(*mv);

        self.halfmove_clock += 1;
        if mv.piece == Piece::Pawn || mv.capture.is_some() {
            self.halfmove_clock = 0;
        }

        if self.turn == Color::Black {
            self.fullmove_number += 1;
        }

        self.turn = self.turn.opposite();
    }
}
