mod attacks_generation;
mod move_generation;
mod utils;
mod zobrist;

use crate::bitboard::Bitboard;
use crate::board::zobrist::ZOBRIST;
use crate::constants::*;
use std::fmt::Display;

pub struct Board {
    pub colors: [Bitboard; 2],
    pub pieces: [Bitboard; 6],

    pub turn: Color,
    pub ply: u32,
    pub game_state: GameState,

    pub moves: Vec<Move>,
    pub zobrist_history: Vec<u64>,
    pub fen_history: Vec<String>,
    pub game_state_history: Vec<GameState>,
}

#[derive(Debug, Copy, Clone)]
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
            colors: [Bitboard::new(); 2],
            pieces: [Bitboard::new(); 6],
            turn: Color::White,
            game_state: GameState {
                captured_piece: None,
                en_passant_square: None,
                castling_rights: CASTLING_RIGHTS[0] | CASTLING_RIGHTS[1],
                fifty_move_ply_count: 0,
                current_zobrist: 0,
            },
            ply: 1,
            moves: Vec::new(),
            zobrist_history: Vec::new(),
            fen_history: Vec::new(),
            game_state_history: vec![GameState {
                captured_piece: None,
                en_passant_square: None,
                castling_rights: CASTLING_RIGHTS[0] | CASTLING_RIGHTS[1],
                fifty_move_ply_count: 0,
                current_zobrist: 0,
            }],
        }
    }

    pub fn init() -> Self {
        let mut board = Board::new();
        board.set_fen(STARTING_POSITION);
        board
    }

    pub fn reset(&mut self) {
        self.colors = [Bitboard::new(); 2];
        self.pieces = [Bitboard::new(); 6];
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
        self.game_state_history = vec![self.game_state];
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

        self.game_state.castling_rights = 0;
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

    pub fn to_fen(&self) -> String {
        let mut fen = String::new();

        for row in (0..BOARD_WIDTH).rev() {
            let mut empty = 0;
            for col in 0..BOARD_WIDTH {
                let index = row * BOARD_WIDTH + col;

                if self.piece_at(index).is_none() {
                    empty += 1;
                } else {
                    if empty > 0 {
                        fen.push_str(&empty.to_string());
                        empty = 0;
                    }

                    let piece = self.piece_at(index).unwrap();
                    let c = match piece.color {
                        Color::White => piece
                            .piece
                            .to_string()
                            .to_uppercase()
                            .chars()
                            .next()
                            .unwrap(),
                        Color::Black => piece.piece.to_string().chars().next().unwrap(),
                    };
                    fen.push(c);
                }
            }

            if empty > 0 {
                fen.push_str(&empty.to_string());
            }

            if row > 0 {
                fen.push('/');
            }
        }

        fen.push(' ');
        fen.push_str(match self.turn {
            Color::White => "w",
            Color::Black => "b",
        });

        fen.push(' ');
        if self.game_state.castling_rights == 0 {
            fen.push('-');
        } else {
            if self.game_state.castling_rights & CASTLING_WHITE_KING != 0 {
                fen.push('K');
            }
            if self.game_state.castling_rights & CASTLING_WHITE_QUEEN != 0 {
                fen.push('Q');
            }
            if self.game_state.castling_rights & CASTLING_BLACK_KING != 0 {
                fen.push('k');
            }
            if self.game_state.castling_rights & CASTLING_BLACK_QUEEN != 0 {
                fen.push('q');
            }
        }

        fen.push(' ');
        match self.game_state.en_passant_square {
            Some(square) => fen.push_str(&Board::index_to_square(square)),
            None => fen.push('-'),
        }

        fen.push(' ');
        fen.push_str(&self.game_state.fifty_move_ply_count.to_string());

        fen.push(' ');
        fen.push_str(&((self.ply / 2) + 1).to_string());

        fen
    }

    pub fn add_piece(&mut self, color: Color, piece: Piece, index: usize) {
        let bb = Bitboard::from_index(index);

        self.colors[color as usize] = self.colors[color as usize].or(&bb);
        self.pieces[piece as usize] = self.pieces[color as usize].or(&bb);
    }

    pub fn remove_piece(&mut self, color: Color, piece: Piece, index: usize) {
        let bb = Bitboard::from_index(index);

        self.colors[color as usize] = self.colors[color as usize].xor(&bb);
        self.pieces[piece as usize] = self.pieces[color as usize].xor(&bb);
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
        let occupancy = self.colors[Color::White as usize].or(&self.colors[Color::Black as usize]);

        while index != to as i32 {
            if occupancy.is_set(index as usize) {
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

        if self.game_state.castling_rights & mask == 0 {
            return false;
        }

        let king_square = CASTLING_RIGHTS_SQUARES[index][0];
        let rook_square = CASTLING_ROOKS[index];

        self.is_empty_between(king_square, rook_square)
    }

    // TODO: CHECK STATUS
    // pub fn check_status(&mut self) -> bool {
    // }

    // TODO: CALCULATE IS CHECK
    // pub fn calculate_is_check(&self) -> bool {
    //     let king_square = self.pieces[self.turn as usize][Piece::King as usize].lsb();
    //     let enemy_orthogonal = self.pieces[self.turn.opposite() as usize][Piece::Rook as usize]
    //         .or(&self.pieces[self.turn.opposite() as usize][Piece::Queen as usize]);
    //     let enemy_diagonal = self.pieces[self.turn.opposite() as usize][Piece::Bishop as usize]
    //         .or(&self.pieces[self.turn.opposite() as usize][Piece::Queen as usize]);
    //
    //     false
    // }

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

        self.turn = self.turn.opposite();

        self.ply += 1;
        let mut new_fifty_move_ply_count = self.game_state.fifty_move_ply_count + 1;
        if mv.piece == Piece::Pawn || mv.capture.is_some() {
            new_fifty_move_ply_count = 0;
        }

        let new_game_state = GameState {
            captured_piece: mv.capture,
            en_passant_square: new_en_passant_square,
            castling_rights: new_castling_rights,
            fifty_move_ply_count: new_fifty_move_ply_count,
            current_zobrist: new_zobrist,
        };

        self.game_state = new_game_state;
        self.game_state_history.push(new_game_state);
        self.zobrist_history.push(new_zobrist);
        self.fen_history.push(self.to_fen());
        self.moves.push(*mv);
    }

    pub fn undo_move(&mut self, mv: &Move) {
        self.turn = self.turn.opposite();
        let last_move = self.moves.pop().unwrap();

        if last_move != *mv {
            panic!("Invalid move");
        }

        if mv.promotion == Some(Piece::Pawn) {
            self.remove_piece(mv.color, mv.promotion.unwrap(), mv.to);
            self.add_piece(mv.color, Piece::Pawn, mv.to);
        }

        self.move_piece(mv.color, mv.piece, mv.to, mv.from);

        if mv.capture.is_some() {
            let mut capture_square = mv.to as i32;

            if mv.en_passant {
                capture_square -= match mv.color {
                    Color::White => MOVE_DOWN,
                    Color::Black => MOVE_UP,
                };
            }

            self.add_piece(
                mv.color.opposite(),
                mv.capture.unwrap(),
                capture_square as usize,
            );
        }

        if mv.piece == Piece::King {
            if mv.castling {
                let (rook_from, rook_to) = match mv.to {
                    2 => (0, 3),
                    6 => (7, 5),
                    _ => panic!("Invalid castling move"),
                };

                self.move_piece(mv.color, Piece::Rook, rook_to, rook_from);
            }
        }

        self.game_state = self.game_state_history.last().unwrap().clone();
        self.zobrist_history.pop();
        self.fen_history.pop();
        self.ply -= 1;
    }
}
