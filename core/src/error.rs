use thiserror::Error;

#[derive(Debug, Error)]
pub enum CoreError {
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

    #[error("Invalid move: {mv:?} (expected UCI format like 'e2e4' or 'e7e8q')")]
    InvalidMove { mv: String },
}
