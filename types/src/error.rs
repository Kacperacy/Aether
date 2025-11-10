//! Error types for chess type parsing and validation.
//!
//! This module provides a unified error type for all type-level operations
//! in the chess engine, including parsing of algebraic notation and validation
//! of board positions.

use std::fmt;

/// Unified error type for chess type operations.
///
/// This error type is used across all type parsing operations (File, Rank,
/// Square, Piece) to provide consistent error handling.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TypeError {
    /// Invalid file character (expected a-h)
    InvalidFile { input: String },

    /// Invalid rank character (expected 1-8)
    InvalidRank { input: String },

    /// Invalid square notation (expected format like "e4")
    InvalidSquare { input: String },

    /// Invalid piece character
    InvalidPiece { input: char },

    /// Invalid index for file (expected 0-7)
    InvalidFileIndex { index: i8 },

    /// Invalid index for rank (expected 0-7)
    InvalidRankIndex { index: i8 },

    /// Invalid index for square (expected 0-63)
    InvalidSquareIndex { index: i8 },
}

impl fmt::Display for TypeError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            TypeError::InvalidFile { input } => {
                write!(f, "Invalid file: '{}' (expected a-h)", input)
            }
            TypeError::InvalidRank { input } => {
                write!(f, "Invalid rank: '{}' (expected 1-8)", input)
            }
            TypeError::InvalidSquare { input } => {
                write!(f, "Invalid square: '{}' (expected format like 'e4')", input)
            }
            TypeError::InvalidPiece { input } => {
                write!(f, "Invalid piece: '{}' (expected p/n/b/r/q/k/P/N/B/R/Q/K)", input)
            }
            TypeError::InvalidFileIndex { index } => {
                write!(f, "Invalid file index: {} (expected 0-7)", index)
            }
            TypeError::InvalidRankIndex { index } => {
                write!(f, "Invalid rank index: {} (expected 0-7)", index)
            }
            TypeError::InvalidSquareIndex { index } => {
                write!(f, "Invalid square index: {} (expected 0-63)", index)
            }
        }
    }
}

impl std::error::Error for TypeError {}

/// Result type alias for chess type operations.
pub type Result<T> = std::result::Result<T, TypeError>;
