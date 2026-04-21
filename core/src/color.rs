use std::fmt::Display;
use std::ops::Not;

use crate::Rank;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[repr(u8)]
pub enum Color {
    White = 0,
    Black = 1,
}

pub const ALL_COLORS: [Color; Color::NUM] = [Color::White, Color::Black];

impl Display for Color {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.as_char())
    }
}

impl Not for Color {
    type Output = Self;

    #[inline]
    fn not(self) -> Self::Output {
        self.opponent()
    }
}

impl Color {
    pub const NUM: usize = 2;

    pub const fn as_char(self) -> char {
        match self {
            Self::White => 'w',
            Self::Black => 'b',
        }
    }

    pub const fn from_char(c: char) -> Option<Self> {
        match c {
            'w' => Some(Self::White),
            'b' => Some(Self::Black),
            _ => None,
        }
    }

    #[inline]
    pub const fn opponent(self) -> Self {
        match self {
            Self::White => Self::Black,
            Self::Black => Self::White,
        }
    }

    pub const fn pawn_start_rank(self) -> Rank {
        match self {
            Self::White => Rank::TWO,
            Self::Black => Rank::SEVEN,
        }
    }

    pub const fn pawn_promotion_rank(self) -> Rank {
        match self {
            Self::White => Rank::EIGHT,
            Self::Black => Rank::ONE,
        }
    }

    pub const fn forward_direction(self) -> i8 {
        match self {
            Self::White => 1,
            Self::Black => -1,
        }
    }

    pub const fn back_rank(self) -> Rank {
        match self {
            Self::White => Rank::ONE,
            Self::Black => Rank::EIGHT,
        }
    }

    pub const fn en_passant_rank(self) -> Rank {
        match self {
            Self::White => Rank::SIX,
            Self::Black => Rank::THREE,
        }
    }
}
