use crate::bitboard::Bitboard;
use crate::constans::*;

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
            moves: Vec::new(),
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

                    self.add_piece(color, piece, rank * 8 + file);
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

    fn is_square_empty(&self, index: usize) -> bool {
        !self.white_occupancy.is_set(index) && !self.black_occupancy.is_set(index)
    }

    pub fn is_index_in_bounds(index: i64) -> bool {
        index >= 0 && index < 64
    }

    pub fn print(&self) {
        for rank in (0..8).rev() {
            for file in 0..8 {
                let index = rank * 8 + file;
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
                Color::White => 8,
                Color::Black => -8,
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
        } else {
            self.remove_piece(mv.color, mv.piece, mv.from);
            self.add_piece(mv.color, mv.piece, mv.to);
        }

        if self.en_passant_square.is_some() {
            self.en_passant_square = None;
        }

        if mv.en_passant {
            let direction = match mv.color {
                Color::White => 8,
                Color::Black => -8,
            };
            self.en_passant_square = Some((mv.to as i64 - direction) as usize);
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

    pub fn generate_possible_moves(self) -> Vec<Move> {
        let mut moves = Vec::new();

        moves.append(&mut self.generate_pawn_moves());

        // TODO: Generate bishop moves
        // TODO: Generate knight moves
        // TODO: Generate rook moves
        // TODO: Generate queen moves
        // TODO: Generate king moves

        moves.iter().for_each(|m| println!("{:?}", m));

        moves
    }

    pub fn generate_pawn_moves(mut self) -> Vec<Move> {
        let mut moves = Vec::new();
        let pawns = match self.turn {
            Color::White => self.white_pieces.pawns,
            Color::Black => self.black_pieces.pawns,
        };

        // TODO: Validate check

        for i in 0..64 {
            if !pawns.is_set(i) {
                continue;
            }

            let direction = match self.turn {
                Color::White => 8,
                Color::Black => -8,
            };

            let pos = i;
            let from = i;
            let possible_to = i as i64 + direction;

            if !Board::is_index_in_bounds(possible_to) {
                continue;
            }

            let to = possible_to as usize;

            // DOUBLE PUSH
            if (RANK_2.is_set(pos) && self.turn == Color::White)
                || (RANK_7.is_set(pos) && self.turn == Color::Black)
            {
                let double = to as i64 + direction;
                if self.is_square_empty(to) && self.is_square_empty(double as usize) {
                    moves.push(Move {
                        from,
                        to: double as usize,
                        piece: Piece::Pawn,
                        color: self.turn,
                        en_passant: true,
                        castling: false,
                        promotion: None,
                    });
                }
            }

            // EN PASSANT
            if let Some(ep) = self.en_passant_square {
                let left = ep - 1;
                let right = ep + 1;
                if left == ep {
                    moves.push(Move {
                        from,
                        to,
                        piece: Piece::Pawn,
                        color: self.turn,
                        en_passant: false,
                        castling: false,
                        promotion: None,
                    });
                }
                if right == ep {
                    moves.push(Move {
                        from,
                        to,
                        piece: Piece::Pawn,
                        color: self.turn,
                        en_passant: false,
                        castling: false,
                        promotion: None,
                    });
                }
            }

            // PROMOTION
            if (self.turn == Color::White && RANK_7.is_set(pos) && self.is_square_empty(to))
                || (self.turn == Color::Black && RANK_2.is_set(pos) && self.is_square_empty(to))
            {
                moves.push(Move {
                    from,
                    to: to,
                    piece: Piece::Pawn,
                    color: self.turn,
                    en_passant: false,
                    castling: false,
                    promotion: Some(Piece::Queen),
                });
                moves.push(Move {
                    from,
                    to: to,
                    piece: Piece::Pawn,
                    color: self.turn,
                    en_passant: false,
                    castling: false,
                    promotion: Some(Piece::Rook),
                });
                moves.push(Move {
                    from,
                    to: to,
                    piece: Piece::Pawn,
                    color: self.turn,
                    en_passant: false,
                    castling: false,
                    promotion: Some(Piece::Bishop),
                });
                moves.push(Move {
                    from,
                    to: to,
                    piece: Piece::Pawn,
                    color: self.turn,
                    en_passant: false,
                    castling: false,
                    promotion: Some(Piece::Knight),
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
                });
            }
        }

        moves
    }
}
