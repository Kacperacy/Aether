use aether_types::{Color, Rank, Square};
use thiserror::Error;

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

    #[error("Invalid move: {0}")]
    ChessMoveError(#[from] MoveError),

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

#[derive(Debug, Error)]
pub enum MoveError {
    #[error("Invalid move format: {move_str}")]
    InvalidMoveFormat { move_str: String },

    #[error("Illegal move: {move_str}")]
    IllegalMove { move_str: String },

    #[error("No piece at source square: {square}")]
    NoPieceAtSource { square: Square },

    #[error("Destination square occupied by own piece: {square}")]
    DestinationOccupiedByOwnPiece { square: Square },

    #[error("Move puts own king in check: {move_str}")]
    MovePutsKingInCheck { move_str: String },

    #[error("Piece mismatch for move: expected {expected:?}, found {found:?}")]
    PieceMismatch { expected: String, found: String },

    #[error("Invalid rook for castling at square: {square}")]
    InvalidRookForCastling { square: Square },

    #[error("No rook found for castling at square: {square}")]
    NoRookForCastling { square: Square },

    #[error("No move to unmake")]
    NoMoveToUnmake,
}
