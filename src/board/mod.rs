mod attacks_generation;
mod move_generation;
mod utils;
mod zobrist;

use crate::bitboard::Bitboard;
use crate::board::zobrist::ZOBRIST;
use crate::constants::*;
use std::fmt::Display;

pub struct Board {
    pub all_occupancy: Bitboard,
    pub occupancy: [Bitboard; 2],
    pub attacks: [[Bitboard; 6]; 2],
    pub pieces: [[Bitboard; 6]; 2],

    pub turn: Color,
    pub ply: u32,
    pub game_state: GameState,

    pub moves: Vec<Move>,
    pub zobrist_history: Vec<u64>,
    pub fen_history: Vec<String>,
}

pub struct GameState {
    pub captured_piece: Option<Piece>,
    pub en_passant_square: Option<usize>,
    pub castling_rights: u8,
    pub fifty_move_ply_count: u8,
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
            all_occupancy: Bitboard::new(),
            occupancy: [Bitboard::new(); 2],
            attacks: [[Bitboard::new(); 6]; 2],
            pieces: [[Bitboard::new(); 6]; 2],
            turn: Color::White,
            game_state: GameState {
                captured_piece: None,
                en_passant_square: None,
                castling_rights: CASTLING_RIGHTS[0] | CASTLING_RIGHTS[1],
                fifty_move_ply_count: 0,
                current_zobrist: 0,
            },
            ply: 0,
            moves: Vec::new(),
            zobrist_history: Vec::new(),
            fen_history: Vec::new(),
        }
    }

    pub fn init() -> Self {
        let mut board = Board::new();
        board.set_fen(STARTING_POSITION);
        board
    }

    pub fn reset(&mut self) {
        self.occupancy = [Bitboard::new(); 2];
        self.attacks = [[Bitboard::new(); 6]; 2];
        self.pieces = [[Bitboard::new(); 6]; 2];
        self.turn = Color::White;
        self.game_state = GameState {
            captured_piece: None,
            en_passant_square: None,
            castling_rights: CASTLING_RIGHTS[0] | CASTLING_RIGHTS[1],
            fifty_move_ply_count: 0,
            current_zobrist: 0,
        };
        self.ply = 0;
        self.moves = Vec::new();
        self.zobrist_history = Vec::new();
        self.fen_history = Vec::new();
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

        if parts[2].contains('K') {
            self.game_state.castling_rights |= CASTLING_WHITE_KING;
        }
        if parts[2].contains('Q') {
            self.game_state.castling_rights |= CASTLING_WHITE_QUEEN;
        }
        if parts[2].contains('k') {
            self.game_state.castling_rights |= CASTLING_BLACK_KING;
        }
        if parts[2].contains('q') {
            self.game_state.castling_rights |= CASTLING_BLACK_QUEEN;
        }

        self.game_state.en_passant_square = match parts[3] {
            "-" => None,
            s => Some(Board::square_to_index(s)),
        };

        self.game_state.fifty_move_ply_count = parts[4].parse().unwrap();
        self.ply = (parts[5].parse::<u32>().unwrap() - 1) * 2
            + if self.turn == Color::Black { 1 } else { 0 };

        self.game_state.current_zobrist = ZOBRIST.hash(&self);
    }

    pub fn add_piece(&mut self, color: Color, piece: Piece, index: usize) {
        let bb = Bitboard::from_index(index);

        self.occupancy[color as usize] = self.occupancy[color as usize].or(&bb);
        self.pieces[color as usize][piece as usize] =
            self.pieces[color as usize][piece as usize].or(&bb);
    }

    pub fn remove_piece(&mut self, color: Color, piece: Piece, index: usize) {
        let bb = Bitboard::from_index(index);

        self.occupancy[color as usize] = self.occupancy[color as usize].and(&bb.not());
        self.pieces[color as usize][piece as usize] =
            self.pieces[color as usize][piece as usize].and(&bb.not());
    }

    pub fn move_piece(&mut self, color: Color, piece: Piece, from: usize, to: usize) {
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

    pub fn is_empty_between(&self, from: usize, to: usize) -> bool {
        let direction = (to as i32 - from as i32).signum();
        let mut index = from as i32 + direction;

        while index != to as i32 {
            if self.occupancy[Color::White as usize].is_set(index as usize)
                || self.occupancy[Color::Black as usize].is_set(index as usize)
            {
                return false;
            }
            index += direction;
        }

        true
    }

    pub fn can_castle(&self, color: Color, is_king_side: bool) -> bool {
        let index = match color {
            Color::White => 0,
            Color::Black => 2,
        } + if is_king_side { 0 } else { 1 };

        let mask = 1 << index;
        let king_square = CASTLING_RIGHTS_SQUARES[index][0];
        let rook_square = CASTLING_ROOKS[index];

        if self.game_state.castling_rights & mask == 0 {
            return false;
        }

        self.is_empty_between(king_square, rook_square)
    }

    fn update_zobrist(&mut self, mv: &Move, square: usize) {
        self.game_state.current_zobrist ^= ZOBRIST.pieces
            [mv.piece as usize + if mv.color == Color::Black { 0 } else { 6 }][square];
    }

    pub fn make_move(&mut self, mv: &Move) {
        let mut new_zobrist = self.game_state.current_zobrist;
        let mut new_castling_rights = self.game_state.castling_rights;
        let mut new_en_passant_square = None;

        self.move_piece(mv.color, mv.piece, mv.from, mv.to);

        // handle capture
        if mv.capture.is_some() {
            let mut capture_square = mv.to as i32;

            // handle en passant capture
            if mv.en_passant {
                capture_square -= match mv.color {
                    Color::White => MOVE_DOWN,
                    Color::Black => MOVE_UP,
                };
            }

            self.remove_piece(
                mv.color.opposite(),
                mv.capture.unwrap(),
                capture_square as usize,
            );

            self.update_zobrist(mv, capture_square as usize);
        }

        // handle castling
        if mv.piece == Piece::King {
            new_castling_rights &= !CASTLING_RIGHTS[mv.color as usize];

            if mv.castling {
                let (rook_from, rook_to) = match mv.to {
                    2 => (0, 3),
                    6 => (7, 5),
                    _ => panic!("Invalid castling move"),
                };

                self.move_piece(mv.color, Piece::Rook, rook_from, rook_to);
                self.update_zobrist(mv, rook_from);
                self.update_zobrist(mv, rook_to);
            }
        }

        // handle promotion
        if mv.promotion.is_some() {
            self.remove_piece(mv.color, Piece::Pawn, mv.to);
            self.add_piece(mv.color, mv.promotion.unwrap(), mv.to);
            self.update_zobrist(mv, mv.to);
        }

        // update en passant square
        if mv.piece == Piece::Pawn && (mv.to as i32 - mv.from as i32).abs() == 16 {
            let direction = match mv.color {
                Color::White => MOVE_UP,
                Color::Black => MOVE_DOWN,
            };
            new_en_passant_square = Some((mv.to as i32 - direction) as usize);
            new_zobrist ^= ZOBRIST.en_passant[new_en_passant_square.unwrap() % 8];
        }

        // update castling rights
        if new_castling_rights != 0 {
            if mv.from == CASTLING_ROOKS[0] || mv.to == CASTLING_ROOKS[0] {
                new_castling_rights &= !CASTLING_WHITE_QUEEN;
            }
            if mv.from == CASTLING_ROOKS[1] || mv.to == CASTLING_ROOKS[1] {
                new_castling_rights &= !CASTLING_WHITE_KING;
            }
            if mv.from == CASTLING_ROOKS[2] || mv.to == CASTLING_ROOKS[2] {
                new_castling_rights &= !CASTLING_BLACK_QUEEN;
            }
            if mv.from == CASTLING_ROOKS[3] || mv.to == CASTLING_ROOKS[3] {
                new_castling_rights &= !CASTLING_BLACK_KING;
            }
        }

        // update zobrist
        let piece_index = mv.piece as usize + if mv.color == Color::Black { 0 } else { 6 };
        new_zobrist ^= ZOBRIST.side;
        new_zobrist ^= ZOBRIST.pieces[piece_index][mv.from];
        new_zobrist ^= ZOBRIST.pieces[piece_index][mv.to];
        new_zobrist ^= ZOBRIST.en_passant[self.game_state.en_passant_square.unwrap_or(0) % 8];

        if new_castling_rights != self.game_state.castling_rights {
            new_zobrist ^= ZOBRIST.castling_rights[self.game_state.castling_rights as usize];
            new_zobrist ^= ZOBRIST.castling_rights[new_castling_rights as usize];
        }

        // Update turn and move counters
        self.turn = self.turn.opposite();

        self.ply += 1;
        let mut new_fifty_move_ply_count = self.game_state.fifty_move_ply_count + 1;
        if mv.piece == Piece::Pawn || mv.capture.is_some() {
            new_fifty_move_ply_count = 0;
        }
    }
}
