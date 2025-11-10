use std::fmt::Display;
use std::ops::Not;

/// Represents the color/side in chess.
///
/// Can be either White or Black. Implements `Not` trait for easy color switching.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[repr(u8)]
pub enum Color {
    /// White pieces
    White = 0,
    /// Black pieces
    Black = 1,
}

impl Display for Color {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.as_char())
    }
}

impl Not for Color {
    type Output = Self;

    fn not(self) -> Self {
        match self {
            Self::White => Self::Black,
            Self::Black => Self::White,
        }
    }
}

impl Color {
    /// Number of colors (2: White and Black)
    pub const NUM: usize = 2;

    /// Converts color to a single character ('w' or 'b').
    pub const fn as_char(self) -> char {
        match self {
            Self::White => 'w',
            Self::Black => 'b',
        }
    }

    /// Creates a color from a character ('w' for White, 'b' for Black).
    pub const fn from_char(c: char) -> Option<Self> {
        match c {
            'w' => Some(Self::White),
            'b' => Some(Self::Black),
            _ => None,
        }
    }

    /// Returns an array containing both colors.
    pub const fn all() -> [Self; 2] {
        [Self::White, Self::Black]
    }

    /// Returns the opposite color.
    ///
    /// White returns Black, Black returns White.
    pub const fn opponent(self) -> Self {
        match self {
            Self::White => Self::Black,
            Self::Black => Self::White,
        }
    }

    /// Returns the starting rank for pawns of this color.
    ///
    /// White pawns start on rank 2, Black pawns on rank 7.
    pub const fn pawn_start_rank(self) -> crate::Rank {
        match self {
            Self::White => crate::Rank::Two,
            Self::Black => crate::Rank::Seven,
        }
    }

    /// Returns the promotion rank for pawns of this color.
    ///
    /// White pawns promote on rank 8, Black pawns on rank 1.
    pub const fn pawn_promotion_rank(self) -> crate::Rank {
        match self {
            Self::White => crate::Rank::Eight,
            Self::Black => crate::Rank::One,
        }
    }

    /// Returns the forward direction for this color (+1 for White, -1 for Black).
    pub const fn forward_direction(self) -> i8 {
        match self {
            Self::White => 1,
            Self::Black => -1,
        }
    }

    /// Returns the back rank for this color.
    ///
    /// White's back rank is rank 1, Black's is rank 8.
    pub const fn back_rank(self) -> crate::Rank {
        match self {
            Self::White => crate::Rank::One,
            Self::Black => crate::Rank::Eight,
        }
    }
}
