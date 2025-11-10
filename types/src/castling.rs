//! Castling rights representation.
//!
//! This module defines castling rights for a chess game, tracking
//! which files the rooks are on for castling purposes.

use crate::File;

/// Represents castling rights for one side.
///
/// Stores which files have rooks available for castling. Using `Option<File>`
/// allows for Chess960 compatibility where rooks may be on non-standard files.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct CastlingRights {
    pub short: Option<File>,
    pub long: Option<File>,
}

impl CastlingRights {
    /// No castling rights available.
    pub const EMPTY: CastlingRights = CastlingRights {
        short: None,
        long: None,
    };

    /// Returns `true` if no castling rights are available.
    pub const fn is_empty(&self) -> bool {
        self.short.is_none() && self.long.is_none()
    }
}
