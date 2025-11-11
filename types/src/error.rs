use std::error::Error;
use std::fmt::{Display, Formatter};

#[derive(Debug, Clone, PartialEq)]
pub enum TypeError {
    InvalidFile(String),
    InvalidRank(String),
    InvalidSquare(String),
    InvalidPiece(String),
    InvalidFileIndex(u8),
    InvalidRankIndex(u8),
    InvalidSquareIndex(u8),
}

impl Display for TypeError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            TypeError::InvalidFile(file) => write!(f, "Invalid file: {} (expected a-h)", file),
            TypeError::InvalidRank(rank) => write!(f, "Invalid rank: {} (expected 1-8)", rank),
            TypeError::InvalidSquare(square) => {
                write!(f, "Invalid square: {} (expected format like 'e4')", square)
            }
            TypeError::InvalidPiece(piece) => write!(
                f,
                "Invalid piece: {} (expected p/n/b/r/q/k/P/N/B/R/Q/K)",
                piece
            ),
            TypeError::InvalidFileIndex(index) => {
                write!(f, "Invalid file index: {} (expected 0-7)", index)
            }
            TypeError::InvalidRankIndex(index) => {
                write!(f, "Invalid rank index: {} (expected 0-7)", index)
            }
            TypeError::InvalidSquareIndex(index) => {
                write!(f, "Invalid square index: {} (expected 0-63)", index)
            }
        }
    }
}

impl Error for TypeError {}

pub type Result<T> = std::result::Result<T, TypeError>;
