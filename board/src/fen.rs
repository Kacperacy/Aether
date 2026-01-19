use crate::error::BoardError::FenParsingError;
use crate::error::FenError;
use crate::{Board, BoardBuilder, Result};
use aether_core::{CastlingRights, Color, File, Piece, Rank, Square};
use std::fmt::{self, Display, Formatter};
use std::str::FromStr;

pub const STARTING_POSITION_FEN: &str = "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1";

const DEFAULT_FEN_FIELDS: [&str; 6] = ["", "w", "-", "-", "0", "1"];

impl FromStr for Board {
    type Err = crate::BoardError;

    fn from_str(fen: &str) -> Result<Self> {
        FenParser::new(fen)?.build_board()
    }
}

impl Display for Board {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "{}", FenGenerator::new(self).generate())
    }
}

struct FenParser<'a> {
    fields: [&'a str; 6],
}

impl<'a> FenParser<'a> {
    fn new(fen: &'a str) -> Result<Self> {
        let trimmed = fen.trim();
        if trimmed.is_empty() {
            return Err(FenParsingError(FenError::EmptyFen));
        }

        let parts: Vec<&str> = trimmed.split_whitespace().collect();

        if parts.len() > 6 {
            return Err(FenParsingError(FenError::TooManyFields));
        }

        let mut fields = DEFAULT_FEN_FIELDS;
        for (i, part) in parts.into_iter().enumerate() {
            fields[i] = part;
        }

        Ok(FenParser { fields })
    }

    fn build_board(self) -> Result<Board> {
        let mut builder = BoardBuilder::new();

        self.parse_piece_placement(&mut builder)?;

        let side_to_move = self.parse_side_to_move()?;
        builder.set_side_to_move(side_to_move);

        let (white_castling, black_castling) = self.parse_castling_rights()?;
        builder
            .set_castling_rights(Color::White, white_castling)
            .set_castling_rights(Color::Black, black_castling);

        let en_passant = self.parse_en_passant(side_to_move)?;
        builder.set_en_passant(en_passant)?;

        let halfmove_clock = self.parse_halfmove_clock()?;
        let fullmove_number = self.parse_fullmove_number()?;

        builder
            .set_halfmove_clock(halfmove_clock)
            .set_fullmove_number(fullmove_number);

        builder.build()
    }

    fn parse_piece_placement(&self, builder: &mut BoardBuilder) -> Result<()> {
        let placement = self.fields[0];
        let ranks: Vec<&str> = placement.split('/').collect();

        if ranks.len() != Rank::NUM {
            return Err(FenParsingError(FenError::WrongAmountOfRanks {
                amount: ranks.len(),
            }));
        }

        for (rank_index, rank_str) in ranks.iter().enumerate() {
            let rank = Rank::from_index((Rank::NUM - 1 - rank_index) as i8);
            let mut file_index: usize = 0;

            for ch in rank_str.chars() {
                if file_index >= File::NUM {
                    return Err(FenParsingError(FenError::TooManySquaresInRank {
                        rank: Rank::from_index(rank_index as i8),
                    }));
                }

                if let Some(empty_count) = ch.to_digit(10) {
                    let empty_count = empty_count as usize;
                    if !(1..=File::NUM).contains(&empty_count) {
                        return Err(FenParsingError(FenError::InvalidEmptySquareCount {
                            count: empty_count,
                        }));
                    }
                    file_index += empty_count;
                } else {
                    let (piece, color) = parse_piece_char(ch)?;
                    let file = File::from_index(file_index as i8);
                    let square = Square::new(file, rank);

                    builder.place_piece(square, piece, color)?;
                    file_index += 1;
                }
            }

            if file_index != File::NUM {
                return Err(FenParsingError(FenError::InvalidRankSquares {
                    rank: Rank::from_index(rank_index as i8),
                    amount: file_index,
                }));
            }
        }

        Ok(())
    }

    fn parse_side_to_move(&self) -> Result<Color> {
        match self.fields[1] {
            "w" => Ok(Color::White),
            "b" => Ok(Color::Black),
            _ => Err(FenParsingError(FenError::InvalidSideToMove {
                side: self.fields[1].to_string(),
            })),
        }
    }

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
                _ => return Err(FenParsingError(FenError::InvalidCastlingRights { ch })),
            }
        }

        Ok((white_rights, black_rights))
    }

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

        let square = Square::from_str(en_passant_str).map_err(|_| {
            FenParsingError(FenError::InvalidEnPassantSquare {
                en_passant_str: en_passant_str.to_string(),
            })
        })?;

        let expected_rank = side_to_move.en_passant_rank();
        if square.rank() != expected_rank {
            return Err(FenParsingError(FenError::InvalidEnPassantRank {
                square,
                rank: expected_rank,
            }));
        }

        Ok(Some(square))
    }

    fn parse_halfmove_clock(&self) -> Result<u16> {
        self.fields[4].parse::<u16>().map_err(|_| {
            FenParsingError(FenError::InvalidHalfmoveClock {
                clock: self.fields[4].to_string(),
            })
        })
    }

    fn parse_fullmove_number(&self) -> Result<u16> {
        let fullmove = self.fields[5].parse::<u16>().map_err(|_| {
            FenParsingError(FenError::InvalidFullmoveNumber {
                number: self.fields[5].to_string(),
            })
        })?;

        Ok(fullmove.max(1))
    }
}

fn parse_piece_char(ch: char) -> Result<(Piece, Color)> {
    let piece =
        Piece::from_char(ch).ok_or(FenParsingError(FenError::InvalidPieceCharacter { ch }))?;
    let color = if ch.is_ascii_uppercase() {
        Color::White
    } else {
        Color::Black
    };
    Ok((piece, color))
}

struct FenGenerator<'a> {
    board: &'a Board,
}

impl<'a> FenGenerator<'a> {
    fn new(board: &'a Board) -> Self {
        FenGenerator { board }
    }

    fn generate(&self) -> String {
        format!(
            "{} {} {} {} {} {}",
            self.generate_piece_placement(),
            self.generate_side_to_move(),
            self.generate_castling_rights(),
            self.generate_en_passant(),
            self.board.halfmove_clock(),
            self.board.fullmove_number()
        )
    }

    fn generate_piece_placement(&self) -> String {
        let mut placement = String::with_capacity(71); // max FEN piece placement length

        for rank_index in (0..Rank::NUM).rev() {
            if rank_index < Rank::NUM - 1 {
                placement.push('/');
            }

            let rank = Rank::from_index(rank_index as i8);
            let mut empty_count = 0;

            for file_index in 0..File::NUM {
                let file = File::from_index(file_index as i8);
                let square = Square::new(file, rank);

                if let Some((piece, color)) = self.board.piece_at(square) {
                    if empty_count > 0 {
                        placement.push(char::from_digit(empty_count, 10).unwrap());
                        empty_count = 0;
                    }

                    let piece_char = if color == Color::White {
                        piece.as_char().to_ascii_uppercase()
                    } else {
                        piece.as_char()
                    };
                    placement.push(piece_char);
                } else {
                    empty_count += 1;
                }
            }

            if empty_count > 0 {
                placement.push(char::from_digit(empty_count, 10).unwrap());
            }
        }

        placement
    }

    fn generate_side_to_move(&self) -> char {
        match self.board.side_to_move() {
            Color::White => 'w',
            Color::Black => 'b',
        }
    }

    fn generate_castling_rights(&self) -> String {
        let white_rights = self.board.castling_rights(Color::White);
        let black_rights = self.board.castling_rights(Color::Black);

        let mut castling = String::with_capacity(4);

        if white_rights.short.is_some() {
            castling.push('K');
        }
        if white_rights.long.is_some() {
            castling.push('Q');
        }
        if black_rights.short.is_some() {
            castling.push('k');
        }
        if black_rights.long.is_some() {
            castling.push('q');
        }

        if castling.is_empty() {
            castling.push('-');
        }

        castling
    }

    fn generate_en_passant(&self) -> String {
        self.board
            .en_passant_square()
            .map_or_else(|| "-".to_string(), |sq| sq.to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_starting_position() {
        let board: Board = STARTING_POSITION_FEN
            .parse()
            .expect("Failed to parse starting position");

        assert_eq!(board.side_to_move(), Color::White);
        assert!(board.can_castle_short(Color::White));
        assert!(board.can_castle_long(Color::White));
        assert!(board.can_castle_short(Color::Black));
        assert!(board.can_castle_long(Color::Black));
        assert!(board.en_passant_square().is_none());
        assert_eq!(board.halfmove_clock(), 0);
        assert_eq!(board.fullmove_number(), 1);
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
            let board: Board = fen
                .parse()
                .unwrap_or_else(|_| panic!("Failed to parse FEN: {}", fen));
            let generated_fen = board.to_string();

            assert_eq!(
                *fen, generated_fen,
                "FEN roundtrip mismatch:\n  input:  {}\n  output: {}",
                fen, generated_fen
            );

            let reparsed_board: Board = generated_fen
                .parse()
                .expect("Failed to parse generated FEN");

            assert_eq!(board.side_to_move(), reparsed_board.side_to_move());
            assert_eq!(board.halfmove_clock(), reparsed_board.halfmove_clock());
            assert_eq!(board.fullmove_number(), reparsed_board.fullmove_number());
            assert_eq!(
                board.en_passant_square(),
                reparsed_board.en_passant_square()
            );
        }
    }

    #[test]
    fn test_invalid_fen_errors() {
        let invalid_fens = [
            "",
            "rnbqkbnr/pppppppp/8/8/8/8/8 w KQkq - 0 1",
            "rnbqkbnr/pppppppp/8/8/8/8/8/8/8 w KQkq - 0 1",
            "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR x KQkq - 0 1",
            "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq e2 0 1",
        ];

        for fen in &invalid_fens {
            assert!(
                fen.parse::<Board>().is_err(),
                "Expected error for FEN: {}",
                fen
            );
        }
    }

    #[test]
    fn test_display_trait() {
        let board: Board = STARTING_POSITION_FEN.parse().unwrap();
        let displayed = format!("{}", board);
        assert_eq!(displayed, STARTING_POSITION_FEN);
    }

    #[test]
    fn test_partial_fen_with_defaults() {
        let partial_fen = "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR";
        let board: Board = partial_fen.parse().expect("Failed to parse partial FEN");

        assert_eq!(board.side_to_move(), Color::White);
        assert!(board.en_passant_square().is_none());
        assert_eq!(board.halfmove_clock(), 0);
        assert_eq!(board.fullmove_number(), 1);
    }
}
