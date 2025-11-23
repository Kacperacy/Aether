//! Forsyth-Edwards Notation (FEN) parsing and generation for chess positions.
//!
//! FEN is a standard notation for describing chess positions using ASCII characters.
//! A FEN record consists of six space-separated fields:
//! 1. Piece placement (from rank 8 to rank 1)
//! 2. Side to move (w/b)
//! 3. Castling availability (KQkq or -)
//! 4. En passant target square (algebraic notation or -)
//! 5. Halfmove clock (moves since last pawn move or capture)
//! 6. Fullmove number (increments after Black's move)

use crate::error::BoardError::FenParsingError;
use crate::error::FenError;
use crate::error::FenError::{
    InvalidEmptySquareCount, InvalidPieceCharacter, InvalidRankSquares, TooManySquaresInRank,
};
use crate::query::BoardQuery;
use crate::{Board, BoardBuilder, Result};
use aether_core::{CastlingRights, Color, File, Piece, Rank, Square};
use std::str::FromStr;

/// Standard starting position FEN
pub const STARTING_POSITION_FEN: &str = "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1";

/// Parse a FEN string and create a Board
pub fn parse_fen(fen: &str) -> Result<Board> {
    let fen_parser = FenParser::new(fen)?;
    fen_parser.build_board()
}

/// Generate a FEN string from a Board
pub fn board_to_fen(board: &Board) -> String {
    FenGenerator::new(board).generate()
}

/// FEN parser that processes each field individually
struct FenParser<'a> {
    fields: Vec<&'a str>,
}

impl<'a> FenParser<'a> {
    /// Create a new FEN parser from a FEN string
    pub fn new(fen: &'a str) -> Result<Self> {
        let trimmed = fen.trim();
        if trimmed.is_empty() {
            return Err(FenParsingError(FenError::EmptyFen));
        }

        let fields: Vec<&str> = trimmed.split_whitespace().collect();

        // FEN must have at least the board field, others can be defaulted
        if fields.is_empty() {
            return Err(FenParsingError(FenError::EmptyFields));
        }

        // Ensure we have exactly 6 fields, padding with defaults if necessary
        let mut complete_fields = fields;
        while complete_fields.len() < 6 {
            match complete_fields.len() {
                1 => complete_fields.push("w"), // Default: white to move
                2 => complete_fields.push("-"), // Default: no castling rights
                3 => complete_fields.push("-"), // Default: no en passant
                4 => complete_fields.push("0"), // Default: halfmove clock = 0
                5 => complete_fields.push("1"), // Default: fullmove number = 1
                _ => break,
            }
        }

        if complete_fields.len() > 6 {
            return Err(FenParsingError(FenError::TooManyFields));
        }

        Ok(FenParser {
            fields: complete_fields,
        })
    }

    /// Build a board from the parsed FEN fields
    pub fn build_board(self) -> Result<Board> {
        let mut builder = BoardBuilder::new();

        // Parse piece placement (field 0)
        self.parse_piece_placement(&mut builder)?;

        // Parse side to move (field 1)
        let side_to_move = self.parse_side_to_move()?;
        builder.set_side_to_move(side_to_move);

        // Parse castling rights (field 2)
        let (white_castling, black_castling) = self.parse_castling_rights()?;
        builder
            .set_castling_rights(Color::White, white_castling)
            .set_castling_rights(Color::Black, black_castling);

        // Parse en passant square (field 3)
        let en_passant = self.parse_en_passant(side_to_move)?;
        builder.set_en_passant(en_passant)?;

        // Parse halfmove clock (field 4) and fullmove number (field 5)
        let halfmove_clock = self.parse_halfmove_clock()?;
        let fullmove_number = self.parse_fullmove_number()?;

        // Build base board and update game state
        let mut board = builder.build()?;
        board.game_state.halfmove_clock = halfmove_clock;
        board.game_state.fullmove_number = fullmove_number;

        Ok(board)
    }

    /// Parse the piece placement field (rank 8 to rank 1)
    fn parse_piece_placement(&self, builder: &mut BoardBuilder) -> Result<()> {
        let placement = self.fields[0];
        let ranks: Vec<&str> = placement.split('/').collect();

        if ranks.len() != 8 {
            return Err(FenParsingError(FenError::WrongAmountOfRanks {
                amount: ranks.len(),
            }));
        }

        for (rank_index, rank_str) in ranks.iter().enumerate() {
            let rank = Rank::from_index(7 - rank_index as i8); // FEN starts from rank 8
            let mut file_index = 0;

            for ch in rank_str.chars() {
                if file_index >= 8 {
                    return Err(FenParsingError(TooManySquaresInRank {
                        rank: Rank::from_index(rank_index as i8),
                    }));
                }

                if ch.is_ascii_digit() {
                    // Empty squares
                    let empty_count = ch.to_digit(10).unwrap() as i8;
                    if !(1..=8).contains(&empty_count) {
                        return Err(FenParsingError(InvalidEmptySquareCount {
                            count: empty_count as usize,
                        }));
                    }
                    file_index += empty_count;
                } else {
                    // Piece
                    let (piece, color) = self.parse_piece_char(ch)?;
                    let file = File::from_index(file_index);
                    let square = Square::new(file, rank);

                    builder.place_piece(square, piece, color)?;
                    file_index += 1;
                }
            }

            if file_index != 8 {
                return Err(FenParsingError(InvalidRankSquares {
                    rank: Rank::from_index(rank_index as i8),
                    amount: file_index as usize,
                }));
            }
        }

        Ok(())
    }

    /// Parse a piece character into piece type and color
    fn parse_piece_char(&self, ch: char) -> Result<(Piece, Color)> {
        let (piece, color) = match ch {
            'P' => (Piece::Pawn, Color::White),
            'N' => (Piece::Knight, Color::White),
            'B' => (Piece::Bishop, Color::White),
            'R' => (Piece::Rook, Color::White),
            'Q' => (Piece::Queen, Color::White),
            'K' => (Piece::King, Color::White),
            'p' => (Piece::Pawn, Color::Black),
            'n' => (Piece::Knight, Color::Black),
            'b' => (Piece::Bishop, Color::Black),
            'r' => (Piece::Rook, Color::Black),
            'q' => (Piece::Queen, Color::Black),
            'k' => (Piece::King, Color::Black),
            _ => return Err(FenParsingError(InvalidPieceCharacter { ch })),
        };
        Ok((piece, color))
    }

    /// Parse side to move field
    fn parse_side_to_move(&self) -> Result<Color> {
        match self.fields[1] {
            "w" => Ok(Color::White),
            "b" => Ok(Color::Black),
            _ => Err(FenParsingError(FenError::InvalidSideToMove {
                side: self.fields[1].to_string(),
            })),
        }
    }

    /// Parse castling rights field
    fn parse_castling_rights(&self) -> Result<(CastlingRights, CastlingRights)> {
        let castling_str = self.fields[2];

        if castling_str == "-" {
            return Ok((CastlingRights::EMPTY, CastlingRights::EMPTY));
        }

        let mut white_rights = CastlingRights::EMPTY;
        let mut black_rights = CastlingRights::EMPTY;

        for ch in castling_str.chars() {
            match ch {
                'K' => white_rights.short = Some(File::H),
                'Q' => white_rights.long = Some(File::A),
                'k' => black_rights.short = Some(File::H),
                'q' => black_rights.long = Some(File::A),
                // Support Chess960 castling rights
                'A'..='H' => white_rights.long = Some(File::from_str(&ch.to_string()).unwrap()),
                'a'..='h' => black_rights.long = Some(File::from_str(&ch.to_string()).unwrap()),
                _ => return Err(FenParsingError(FenError::InvalidCastlingRights { ch })),
            }
        }

        Ok((white_rights, black_rights))
    }

    /// Parse en passant target square
    fn parse_en_passant(&self, side_to_move: Color) -> Result<Option<Square>> {
        let en_passant_str = self.fields[3];

        if en_passant_str == "-" {
            return Ok(None);
        }

        if en_passant_str.len() != 2 {
            return Err(FenParsingError(FenError::InvalidEnPassantSquare {
                en_passant_str: en_passant_str.to_string(),
            }));
        }

        match Square::from_str(en_passant_str) {
            Ok(square) => {
                // Validate en passant square is on correct rank
                let expected_rank = match side_to_move {
                    Color::White => Rank::Six,
                    Color::Black => Rank::Three,
                };

                if square.rank() != expected_rank {
                    return Err(FenParsingError(FenError::InvalidEnPassantRank {
                        square,
                        rank: expected_rank,
                    }));
                }

                Ok(Some(square))
            }
            Err(_) => Err(FenParsingError(FenError::InvalidEnPassantSquare {
                en_passant_str: en_passant_str.to_string(),
            })),
        }
    }

    /// Parse halfmove clock
    fn parse_halfmove_clock(&self) -> Result<u16> {
        self.fields[4].parse::<u16>().map_err(|_| {
            FenParsingError(FenError::InvalidHalfmoveClock {
                clock: self.fields[4].to_string(),
            })
        })
    }

    /// Parse fullmove number
    fn parse_fullmove_number(&self) -> Result<u16> {
        let fullmove = self.fields[5].parse::<u16>().map_err(|_| {
            FenParsingError(FenError::InvalidFullmoveNumber {
                number: self.fields[5].to_string(),
            })
        })?;

        // Ensure fullmove number is at least 1
        if fullmove == 0 { Ok(1) } else { Ok(fullmove) }
    }
}

/// FEN generator that creates FEN strings from board positions
struct FenGenerator<'a> {
    board: &'a Board,
}

impl<'a> FenGenerator<'a> {
    /// Create a new FEN generator for a board
    pub fn new(board: &'a Board) -> Self {
        FenGenerator { board }
    }

    /// Generate a complete FEN string
    pub fn generate(&self) -> String {
        let placement = self.generate_piece_placement();
        let side_to_move = self.generate_side_to_move();
        let castling = self.generate_castling_rights();
        let en_passant = self.generate_en_passant();
        let halfmove = self.generate_halfmove_clock();
        let fullmove = self.generate_fullmove_number();

        format!("{placement} {side_to_move} {castling} {en_passant} {halfmove} {fullmove}")
    }

    /// Generate piece placement field
    fn generate_piece_placement(&self) -> String {
        let mut placement = String::new();

        for rank_index in 0..8 {
            let rank = Rank::from_index(7 - rank_index); // Start from rank 8
            let mut empty_count = 0;

            for file_index in 0..8 {
                let file = File::from_index(file_index);
                let square = Square::new(file, rank);

                if let Some((piece, color)) = self.board.piece_at(square) {
                    // Place any accumulated empty squares
                    if empty_count > 0 {
                        placement.push_str(&empty_count.to_string());
                        empty_count = 0;
                    }

                    // Add piece character
                    let piece_char = match (piece, color) {
                        (Piece::Pawn, Color::White) => 'P',
                        (Piece::Knight, Color::White) => 'N',
                        (Piece::Bishop, Color::White) => 'B',
                        (Piece::Rook, Color::White) => 'R',
                        (Piece::Queen, Color::White) => 'Q',
                        (Piece::King, Color::White) => 'K',
                        (Piece::Pawn, Color::Black) => 'p',
                        (Piece::Knight, Color::Black) => 'n',
                        (Piece::Bishop, Color::Black) => 'b',
                        (Piece::Rook, Color::Black) => 'r',
                        (Piece::Queen, Color::Black) => 'q',
                        (Piece::King, Color::Black) => 'k',
                    };
                    placement.push(piece_char);
                } else {
                    // Empty square
                    empty_count += 1;
                }
            }

            // Add any remaining empty squares for this rank
            if empty_count > 0 {
                placement.push_str(&empty_count.to_string());
            }

            // Add rank separator (except for last rank)
            if rank_index < 7 {
                placement.push('/');
            }
        }

        placement
    }

    /// Generate side to move field
    fn generate_side_to_move(&self) -> String {
        match self.board.game_state().side_to_move {
            Color::White => "w".to_string(),
            Color::Black => "b".to_string(),
        }
    }

    /// Generate castling rights field
    fn generate_castling_rights(&self) -> String {
        let white_rights = &self.board.game_state().castling_rights[Color::White as usize];
        let black_rights = &self.board.game_state().castling_rights[Color::Black as usize];

        let mut castling = String::new();

        // White castling rights
        if white_rights.short.is_some() {
            castling.push('K');
        }
        if white_rights.long.is_some() {
            castling.push('Q');
        }

        // Black castling rights
        if black_rights.short.is_some() {
            castling.push('k');
        }
        if black_rights.long.is_some() {
            castling.push('q');
        }

        if castling.is_empty() {
            "-".to_string()
        } else {
            castling
        }
    }

    /// Generate en passant field
    fn generate_en_passant(&self) -> String {
        match self.board.game_state().en_passant_square {
            Some(square) => square.to_string(),
            None => "-".to_string(),
        }
    }

    /// Generate halfmove clock field
    fn generate_halfmove_clock(&self) -> String {
        self.board.game_state().halfmove_clock.to_string()
    }

    /// Generate fullmove number field
    fn generate_fullmove_number(&self) -> String {
        self.board.game_state().fullmove_number.to_string()
    }
}

/// Extension trait for Board to add FEN functionality
pub trait FenOps {
    /// Parse a FEN string and return a Board
    fn from_fen(fen: &str) -> Result<Board>;

    /// Convert this board to a FEN string
    fn to_fen(&self) -> String;
}

impl FenOps for Board {
    fn from_fen(fen: &str) -> Result<Board> {
        parse_fen(fen)
    }

    fn to_fen(&self) -> String {
        board_to_fen(self)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_starting_position() {
        let board =
            Board::from_fen(STARTING_POSITION_FEN).expect("Failed to parse starting position");

        // Verify side to move
        assert_eq!(board.game_state().side_to_move, Color::White);

        // Verify castling rights
        assert!(board.can_castle_short(Color::White));
        assert!(board.can_castle_long(Color::White));
        assert!(board.can_castle_short(Color::Black));
        assert!(board.can_castle_long(Color::Black));

        // Verify en passant
        assert!(board.en_passant_square().is_none());

        // Verify move counters
        assert_eq!(board.game_state().halfmove_clock, 0);
        assert_eq!(board.game_state().fullmove_number, 1);
    }

    #[test]
    fn test_parse_and_generate_roundtrip() {
        let test_fens = [
            STARTING_POSITION_FEN,
            "rnbqkbnr/pppppppp/8/8/4P3/8/PPPP1PPP/RNBQKBNR b KQkq e3 0 1",
            "rnbqkb1r/pppp1ppp/5n2/4p3/2B1P3/8/PPPP1PPP/RNBQK1NR w KQkq - 4 3",
            "7k/8/8/8/8/8/8/4K2R w K - 0 1",
        ];

        for fen in &test_fens {
            let board = Board::from_fen(fen).expect(&format!("Failed to parse FEN: {}", fen));
            let generated_fen = board.to_fen();
            let reparsed_board =
                Board::from_fen(&generated_fen).expect("Failed to parse generated FEN");

            // The generated FEN should parse to an equivalent board
            assert_eq!(
                board.game_state().side_to_move,
                reparsed_board.game_state().side_to_move
            );
        }
    }

    #[test]
    fn test_invalid_fen_errors() {
        let invalid_fens = [
            "",                                                          // Empty
            "rnbqkbnr/pppppppp/8/8/8/8/8 w KQkq - 0 1",                  // Too few ranks
            "rnbqkbnr/pppppppp/8/8/8/8/8/8/8 w KQkq - 0 1",              // Too many ranks
            "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR x KQkq - 0 1",  // Invalid side to move
            "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq e2 0 1", // Invalid en passant rank
        ];

        for fen in &invalid_fens {
            assert!(
                Board::from_fen(fen).is_err(),
                "Expected error for FEN: {}",
                fen
            );
        }
    }
}
