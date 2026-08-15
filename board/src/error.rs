use aether_core::{Color, Rank, Square};
use thiserror::Error;

#[derive(Debug, Error)]
pub enum BoardError {
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

    #[error("No move to unmake")]
    NoMoveToUnmake,

    #[error("FEN parsing error: {0}")]
    FenParsingError(FenError),

    #[error("Invalid castling destination square {square:?} for {color:?}")]
    InvalidCastlingDestination { square: Square, color: Color },
}

#[derive(Debug, Error)]
pub enum FenError {
    #[error("Empty fen string")]
    EmptyFen,

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

    #[error("Invalid castling rights: {rights}")]
    InvalidCastlingRights { rights: String },

    #[error("Invalid en_passant square: {en_passant_str}")]
    InvalidEnPassantSquare { en_passant_str: String },

    #[error("En passant square {square} is not on expected rank {rank}")]
    InvalidEnPassantRank { square: Square, rank: Rank },

    #[error("Invalid halfmove clock: {clock}")]
    InvalidHalfmoveClock { clock: String },

    #[error("Invalid fullmove number: {number}")]
    InvalidFullmoveNumber { number: String },
}
