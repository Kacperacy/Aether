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
        let white_pieces = Pieces {
            pawns: Bitboard(0b0000000000000000000000000000000000000000000000001111111100000000),
            knights: Bitboard(0b0000000000000000000000000000000000000000000000000000000001000010),
            bishops: Bitboard(0b0000000000000000000000000000000000000000000000000000000000100100),
            rooks: Bitboard(0b0000000000000000000000000000000000000000000000000000000010000001),
            queens: Bitboard(0b0000000000000000000000000000000000000000000000000000000000001000),
            king: Bitboard(0b0000000000000000000000000000000000000000000000000000000000010000),
        };

        let white_occupancy = Bitboard::new().or(&white_pieces.pawns).or(&white_pieces.knights).or(&white_pieces.bishops)
            .or(&white_pieces.rooks).or(&white_pieces.queens).or(&white_pieces.king);

        let black_pieces = Pieces {
            pawns: Bitboard(0b0000000011111111000000000000000000000000000000000000000000000000),
            knights: Bitboard(0b0100001000000000000000000000000000000000000000000000000000000000),
            bishops: Bitboard(0b0010010000000000000000000000000000000000000000000000000000000000),
            rooks: Bitboard(0b1000000100000000000000000000000000000000000000000000000000000000),
            queens: Bitboard(0b0000100000000000000000000000000000000000000000000000000000000000),
            king: Bitboard(0b0001000000000000000000000000000000000000000000000000000000000000),
        };

        let black_occupancy = Bitboard::new().or(&black_pieces.pawns).or(&black_pieces.knights).or(&black_pieces.bishops)
            .or(&black_pieces.rooks).or(&black_pieces.queens).or(&black_pieces.king);

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
    pub fn square_to_index(square: &str) -> usize {
        let file = square.chars().nth(0).unwrap() as usize - 'a' as usize;
        let rank = square.chars().nth(1).unwrap().to_digit(10).unwrap() as usize - 1;

        rank * 8 + file
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
    pub fn place_piece(&mut self, color: Color, piece: Piece, index: usize) {
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
    pub fn is_pawn_starting_position(color: Color, position: usize) -> bool {
        match color {
            Color::White => (8..16).contains(&position),
            Color::Black => (48..56).contains(&position),
        }
    }

    /// Check is square is empty
    pub fn is_square_empty(index: usize, occupancy: Bitboard) -> bool {
        !occupancy.is_set(index)
    }

    /// Chceck if enemy piece is on square
    pub fn is_square_enemy(&self, color: Color, position: usize) -> bool {
        match color {
            Color::White => self.black_occupancy.is_set(position),
            Color::Black => self.white_occupancy.is_set(position),
        }
    }

    pub fn is_check(&self) -> Option<Color> {
        let (king_position, opponent_occupancy, opponent_pieces) = match self.turn {
            Color::White => (
                self.white_pieces.king,
                self.black_occupancy,
                &self.black_pieces
            ),
            Color::Black => (
                self.black_pieces.king,
                self.white_occupancy,
                &self.white_pieces
            )
        };

        let attacks = self.generate_pawn_attacks(self.turn, opponent_pieces.pawns)
            .or(&self.generate_knight_attacks(opponent_pieces.knights))
            .or(&self.generate_bishop_attacks(opponent_pieces.bishops, opponent_occupancy))
            .or(&self.generate_rook_attacks(opponent_pieces.rooks, opponent_occupancy))
            .or(&self.generate_queen_attacks(opponent_pieces.queens, opponent_occupancy))
            .or(&self.generate_king_attacks(opponent_pieces.king));

        if attacks.is_set(king_position.first_set_bit().unwrap()) {
            return Some(self.turn);
        }

        None
    }

    pub fn generate_pawn_attacks(&self, color: Color, pawns: Bitboard) -> Bitboard {
        match color {
            Color::White => {
                let attacks_left = pawns.left_shift(7) & !Bitboard(0x8080808080808080);
                let attacks_right = pawns.left_shift(9) & !Bitboard(0x0101010101010101);
                attacks_left.or(&attacks_right)
            }
            Color::Black => {
                let attacks_left = pawns.right_shift(9) & !Bitboard(0x8080808080808080);
                let attacks_right = pawns.right_shift(7) & !Bitboard(0x0101010101010101);
                attacks_left.or(&attacks_right)
            }
        }
    }

    pub fn generate_knight_attacks(&self, knights: Bitboard) -> Bitboard {
        let mut attacks = Bitboard::new();
        let knight_positions = knights;

        for i in 0..64 {
            if knight_positions.is_set(i) {
                let rank = i / 8;
                let file = i % 8;

                let knight_moves = [
                    (i.wrapping_add(17), rank + 2 <= 7 && file + 1 <= 7),
                    (i.wrapping_add(15), rank + 2 <= 7 && file >= 1),
                    (i.wrapping_add(10), rank + 1 <= 7 && file + 2 <= 7),
                    (i.wrapping_add(6), rank + 1 <= 7 && file >= 2),
                    (i.wrapping_sub(17), rank >= 2 && file >= 1),
                    (i.wrapping_sub(15), rank >= 2 && file + 1 <= 7),
                    (i.wrapping_sub(10), rank >= 1 && file >= 2),
                    (i.wrapping_sub(6), rank >= 1 && file + 2 <= 7),
                ];

                for &(move_index, valid) in &knight_moves {
                    if valid && move_index < 64 {
                        attacks.set_bit(move_index);
                    }
                }
            }
        }

        attacks
    }

    pub fn generate_bishop_attacks(&self, bishops: Bitboard, occupancy: Bitboard) -> Bitboard {
        let mut attacks = Bitboard::new();
        let bishop_positions = bishops;

        for i in 0..64 {
            if bishop_positions.is_set(i) {
                attacks = attacks.or(&self.generate_diagonal_attacks(i, occupancy));
            }
        }

        attacks
    }

    fn generate_diagonal_attacks(&self, index: usize, occupancy: Bitboard) -> Bitboard {
        let mut attacks = Bitboard::new();
        let directions = [9, 7, -9, -7];

        for &direction in &directions {
            let mut current_index = index as isize;

            loop {
                current_index += direction;

                if current_index < 0 || current_index >= 64 {
                    break;
                }

                if (direction == 9 || direction == -7) && current_index % 8 == 0 {
                    break;
                }
                if (direction == 7 || direction == -9) && current_index % 8 == 7 {
                    break;
                }

                let current_index_usize = current_index as usize;
                attacks.set_bit(current_index_usize);

                if occupancy.is_set(current_index_usize) {
                    break;
                }
            }
        }

        attacks
    }

    pub fn generate_rook_attacks(&self, rooks: Bitboard, occupancy: Bitboard) -> Bitboard {
        let mut attacks = Bitboard::new();
        let rook_positions = rooks;

        for i in 0..64 {
            if rook_positions.is_set(i) {
                attacks = attacks.or(&self.generate_straight_attacks(i, occupancy));
            }
        }

        attacks
    }

    fn generate_straight_attacks(&self, index: usize, occupancy: Bitboard) -> Bitboard {
        let mut attacks = Bitboard::new();
        let directions = [8, -8, 1, -1];

        for &direction in &directions {
            let mut current_index = index as isize;

            loop {
                current_index += direction;

                if current_index < 0 || current_index >= 64 {
                    break;
                }

                if (direction == 1 && current_index % 8 == 0) || (direction == -1 && current_index % 8 == 7) {
                    break;
                }

                let current_index_usize = current_index as usize;
                attacks.set_bit(current_index_usize);

                if occupancy.is_set(current_index_usize) {
                    break;
                }
            }
        }

        attacks
    }

    pub fn generate_queen_attacks(&self, queens: Bitboard, occupancy: Bitboard) -> Bitboard {
        let mut attacks = Bitboard::new();
        let queen_positions = queens;

        for i in 0..64 {
            if queen_positions.is_set(i) {
                attacks = attacks.or(&self.generate_diagonal_attacks(i, occupancy));
                attacks = attacks.or(&self.generate_straight_attacks(i, occupancy));
            }
        }

        attacks
    }

    pub fn generate_king_attacks(&self, king: Bitboard) -> Bitboard {
        let mut attacks = Bitboard::new();
        let king_index = king.first_set_bit().unwrap();

        let king_moves = [
            king_index.wrapping_add(8), king_index.wrapping_sub(8),
            king_index.wrapping_add(1), king_index.wrapping_sub(1),
            king_index.wrapping_add(9), king_index.wrapping_sub(9),
            king_index.wrapping_add(7), king_index.wrapping_sub(7),
        ];

        for &move_index in &king_moves {
            if move_index < 64 {
                attacks.set_bit(move_index);
            }
        }

        attacks
    }

    pub fn generate_all_moves(&self) -> Vec<(usize, usize)> {
        let mut moves = Vec::new();
        let (pieces, occupancy, opponent_occupancy) = match self.turn {
            Color::White => (&self.white_pieces, self.white_occupancy, self.black_occupancy),
            Color::Black => (&self.black_pieces, self.black_occupancy, self.white_occupancy),
        };

        moves.extend(self.generate_pawn_moves(pieces.pawns, occupancy, opponent_occupancy));

        moves.extend(self.generate_knight_moves(pieces.knights, occupancy, opponent_occupancy));

        moves.extend(self.generate_bishop_moves(pieces.bishops, occupancy, opponent_occupancy));

        moves.extend(self.generate_rook_moves(pieces.rooks, occupancy, opponent_occupancy));

        moves.extend(self.generate_queen_moves(pieces.queens, occupancy, opponent_occupancy));

        moves.extend(self.generate_king_moves(pieces.king, occupancy, opponent_occupancy));

        moves
    }

    fn generate_pawn_moves(&self, pawns: Bitboard, occupancy: Bitboard, opponent_occupancy: Bitboard) -> Vec<(usize, usize)> {
        // Implement pawn move generation logic
        vec![]
    }

    fn generate_knight_moves(&self, knights: Bitboard, occupancy: Bitboard, opponent_occupancy: Bitboard) -> Vec<(usize, usize)> {
        // Implement knight move generation logic
        vec![]
    }

    fn generate_bishop_moves(&self, bishops: Bitboard, occupancy: Bitboard, opponent_occupancy: Bitboard) -> Vec<(usize, usize)> {
        // Implement bishop move generation logic
        vec![]
    }

    fn generate_rook_moves(&self, rooks: Bitboard, occupancy: Bitboard, opponent_occupancy: Bitboard) -> Vec<(usize, usize)> {
        // Implement rook move generation logic
        vec![]
    }

    fn generate_queen_moves(&self, queens: Bitboard, occupancy: Bitboard, opponent_occupancy: Bitboard) -> Vec<(usize, usize)> {
        // Implement queen move generation logic
        vec![]
    }

    fn generate_king_moves(&self, king: Bitboard, occupancy: Bitboard, opponent_occupancy: Bitboard) -> Vec<(usize, usize)> {
        // Implement king move generation logic
        vec![]
    }
}