use crate::Board;
use aether_types::{BitBoard, CastlingRights, Color, File, Piece, Rank, Square};

impl Board {
    pub fn from_fen(fen: &str) -> Result<Self, &'static str> {
        let mut board = Self::new();

        board.pieces = [[BitBoard::default(); 6]; 2];

        let sections: Vec<&str> = fen.split_whitespace().collect();
        if sections.len() < 6 {
            return Err("Invalid FEN: not enough sections");
        }

        let placement = sections[0];
        let mut file = File::A;
        let mut rank = Rank::Eight;

        for c in placement.chars() {
            match c {
                '/' => {
                    if let Some(next_rank) = rank.offset(-1) {
                        rank = next_rank;
                        file = File::A;
                    } else {
                        return Err("Invalid FEN: too many ranks");
                    }
                }
                '1'..='8' => {
                    let skip = c.to_digit(10).unwrap() as i8;
                    for _ in 0..skip {
                        if file as i8 > 7 {
                            return Err("Invalid FEN: too many files in a rank");
                        }
                        if let Some(next_file) = file.offset(1) {
                            file = next_file;
                        }
                    }
                }
                'P' => {
                    board.place_piece(Square::new(file, rank), Piece::Pawn, Color::White);
                    if let Some(next_file) = file.offset(1) {
                        file = next_file;
                    }
                }
                'N' => {
                    board.place_piece(Square::new(file, rank), Piece::Knight, Color::White);
                    if let Some(next_file) = file.offset(1) {
                        file = next_file;
                    }
                }
                'B' => {
                    board.place_piece(Square::new(file, rank), Piece::Bishop, Color::White);
                    if let Some(next_file) = file.offset(1) {
                        file = next_file;
                    }
                }
                'R' => {
                    board.place_piece(Square::new(file, rank), Piece::Rook, Color::White);
                    if let Some(next_file) = file.offset(1) {
                        file = next_file;
                    }
                }
                'Q' => {
                    board.place_piece(Square::new(file, rank), Piece::Queen, Color::White);
                    if let Some(next_file) = file.offset(1) {
                        file = next_file;
                    }
                }
                'K' => {
                    board.place_piece(Square::new(file, rank), Piece::King, Color::White);
                    if let Some(next_file) = file.offset(1) {
                        file = next_file;
                    }
                }
                'p' => {
                    board.place_piece(Square::new(file, rank), Piece::Pawn, Color::Black);
                    if let Some(next_file) = file.offset(1) {
                        file = next_file;
                    }
                }
                'n' => {
                    board.place_piece(Square::new(file, rank), Piece::Knight, Color::Black);
                    if let Some(next_file) = file.offset(1) {
                        file = next_file;
                    }
                }
                'b' => {
                    board.place_piece(Square::new(file, rank), Piece::Bishop, Color::Black);
                    if let Some(next_file) = file.offset(1) {
                        file = next_file;
                    }
                }
                'r' => {
                    board.place_piece(Square::new(file, rank), Piece::Rook, Color::Black);
                    if let Some(next_file) = file.offset(1) {
                        file = next_file;
                    }
                }
                'q' => {
                    board.place_piece(Square::new(file, rank), Piece::Queen, Color::Black);
                    if let Some(next_file) = file.offset(1) {
                        file = next_file;
                    }
                }
                'k' => {
                    board.place_piece(Square::new(file, rank), Piece::King, Color::Black);
                    if let Some(next_file) = file.offset(1) {
                        file = next_file;
                    }
                }
                _ => return Err("Invalid FEN: unknown piece character"),
            }
        }

        match sections[1] {
            "w" => board.side_to_move = Color::White,
            "b" => board.side_to_move = Color::Black,
            _ => return Err("Invalid FEN: side to move must be 'w' or 'b'"),
        }

        let castling = sections[2];
        board.castling_rights = [CastlingRights::EMPTY; 2];

        if castling != "-" {
            for c in castling.chars() {
                match c {
                    // TODO: Update fields in the future
                    'K' => board.castling_rights[Color::White as usize].short = Some(File::H),
                    'Q' => board.castling_rights[Color::White as usize].long = Some(File::A),
                    'k' => board.castling_rights[Color::Black as usize].short = Some(File::H),
                    'q' => board.castling_rights[Color::Black as usize].long = Some(File::A),
                    _ => return Err("Invalid FEN: unknown castling character"),
                }
            }
        }

        if sections[3] != "-" {
            if let Ok(square) = Square::from_algebraic(sections[3]) {
                board.en_passant_square = Some(square);
            } else {
                return Err("Invalid FEN: invalid en passant square");
            }
        }

        if let Ok(halfmove) = sections[4].parse::<u8>() {
            board.halfmove_clock = halfmove;
        } else {
            return Err("Invalid FEN: halfmove clock must be a number");
        }

        if let Ok(fullmove) = sections[5].parse::<u16>() {
            board.fullmove_number = fullmove;
        } else {
            return Err("Invalid FEN: fullmove number must be a number");
        }

        Ok(board)
    }

    pub fn to_fen(&self) -> String {
        let mut fen = String::new();

        for rank in (Rank::One as i8..=Rank::Eight as i8).rev() {
            let mut empty = 0;
            for file in File::A as i8..=File::H as i8 {
                let square = Square::new(File::from_index(file), Rank::new(rank));
                if let Some((piece, color)) = self.piece_at(square) {
                    if empty > 0 {
                        fen.push_str(&empty.to_string());
                        empty = 0;
                    }
                    let piece_char = match piece {
                        Piece::Pawn => 'P',
                        Piece::Knight => 'N',
                        Piece::Bishop => 'B',
                        Piece::Rook => 'R',
                        Piece::Queen => 'Q',
                        Piece::King => 'K',
                    };
                    let color_char = match color {
                        Color::White => piece_char,
                        Color::Black => piece_char.to_ascii_lowercase(),
                    };
                    fen.push(color_char);
                } else {
                    empty += 1;
                }
            }
            if empty > 0 {
                fen.push_str(&empty.to_string());
            }
            if rank > Rank::One as i8 {
                fen.push('/');
            }
        }

        fen.push(' ');

        fen.push(match self.side_to_move {
            Color::White => 'w',
            Color::Black => 'b',
        });

        fen.push(' ');

        let mut castling = String::new();
        if self.can_castle_short(Color::White) {
            castling.push('K');
        }
        if self.can_castle_long(Color::White) {
            castling.push('Q');
        }
        if self.can_castle_short(Color::Black) {
            castling.push('k');
        }
        if self.can_castle_long(Color::Black) {
            castling.push('q');
        }
        if castling.is_empty() {
            fen.push('-');
        } else {
            fen.push_str(&castling);
        }

        fen.push(' ');

        fen.push_str(
            &self
                .en_passant_square
                .map_or("-".to_string(), |square| square.to_algebraic()),
        );

        fen.push(' ');

        fen.push_str(&self.halfmove_clock.to_string());

        fen.push(' ');

        fen.push_str(&self.fullmove_number.to_string());

        fen
    }
}
