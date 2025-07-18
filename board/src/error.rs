use aether_types::{Color, Square};
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

    #[error("FEN parsing error: {message}")]
    FenParsingError { message: String },
}

pub type Result<T> = std::result::Result<T, BoardError>;
