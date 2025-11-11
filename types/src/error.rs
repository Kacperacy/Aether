use crate::{Color, Rank, Square};
use std::fmt::Display;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum TypeError {
    #[error("Invalid file: {file:?} (expected a-h)")]
    InvalidFile { file: String },

    #[error("Invalid rank: {rank:?} (expected 1-8)")]
    InvalidRank { rank: String },

    #[error("Invalid square: {square:?} (expected format like 'e4')")]
    InvalidSquare { square: String },

    #[error("Invalid piece: {piece:?} (expected p/n/b/r/q/k/P/N/B/R/Q/K)")]
    InvalidPiece { piece: String },

    #[error("Invalid file index: {file_index:?} (expected 0-7)")]
    InvalidFileIndex { file_index: u8 },

    #[error("Invalid rank index: {rank_index:?} (expected 0-7)")]
    InvalidRankIndex { rank_index: u8 },

    #[error("Invalid square index: {square_index:?} (expected 0-63)")]
    InvalidSquareIndex { square_index: u8 },
}

pub type TypeResult<T> = Result<T, TypeError>;

#[derive(Debug, Error)]
pub enum BoardError {
    #[error("Invalid piece placement: {piece:?} at {square:?}")]
    InvalidPiecePlacement { piece: String, square: Square },

    #[error("King not found for {color:?}")]
    KingNotFound { color: Color },

    #[error("Multiple kings found for {color:?}")]
    MultipleKings { color: Color },

    #[error("Invalid castling rights: {reason}")]
    InvalidCastlingRights { reason: String },

    #[error("Overlapping pieces at {square:?}")]
    OverlappingPieces { square: Square },

    #[error("Invalid en passant square: {square:?}")]
    InvalidEnPassantSquare { square: Square },

    #[error("FEN parsing error: {0}")]
    FenParsingError(#[from] FenError),
}

#[derive(Debug, Error)]
pub enum FenError {
    #[error("Empty fen string")]
    EmptyFen,

    #[error("FEN must contain at least piece placement field")]
    EmptyFields,

    #[error("FEN contains too many fields")]
    TooManyFields,

    #[error("Expected 8 ranks, found {amount}")]
    WrongAmountOfRanks { amount: usize },

    #[error("Too many squares in rank {rank}")]
    TooManySquaresInRank { rank: Rank },

    #[error("Invalid empty square count: {count}")]
    InvalidEmptySquareCount { count: usize },

    #[error("Rank {rank} has {amount} squares, expected 8")]
    InvalidRankSquares { rank: Rank, amount: usize },

    #[error("Invalid piece character: {ch}")]
    InvalidPieceCharacter { ch: char },

    #[error("Invalid side to move: {side}")]
    InvalidSideToMove { side: String },

    #[error("Invalid castling right: {ch}")]
    InvalidCastlingRights { ch: char },

    #[error("Invalid en_passant square: {en_passant_str}")]
    InvalidEnPassantSquare { en_passant_str: String },

    #[error("En passant square {square} is not on expected rank {rank}")]
    InvalidEnPassantRank { square: Square, rank: Rank },

    #[error("Invalid halfmove clock: {clock}")]
    InvalidHalfmoveClock { clock: String },

    #[error("Invalid fullmove number: {number}")]
    InvalidFullmoveNumber { number: String },
}

pub type BoardResult<T> = Result<T, BoardError>;
