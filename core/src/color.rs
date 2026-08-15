use crate::CoreError::InvalidColor;
use crate::{CoreError, Rank, Result};
use std::fmt::Display;
use std::ops::Not;
use std::str::FromStr;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
#[repr(u8)]
pub enum Color {
    White = 0,
    Black = 1,
}

impl Color {
    pub const NUM: usize = 2;

    pub const ALL: [Self; Self::NUM] = [Self::White, Self::Black];

    #[inline(always)]
    pub const fn from_index(index: u8) -> Self {
        debug_assert!(index < 2, "Color index out of bounds");
        match index {
            0 => Self::White,
            _ => Self::Black,
        }
    }

    #[inline]
    pub const fn as_char(self) -> char {
        match self {
            Self::White => 'w',
            Self::Black => 'b',
        }
    }

    #[inline]
    pub const fn from_char(c: char) -> Option<Self> {
        match c {
            'w' => Some(Self::White),
            'b' => Some(Self::Black),
            _ => None,
        }
    }

    #[inline(always)]
    pub const fn opponent(self) -> Self {
        match self {
            Self::White => Self::Black,
            Self::Black => Self::White,
        }
    }

    #[inline(always)]
    pub const fn pawn_start_rank(self) -> Rank {
        match self {
            Self::White => Rank::TWO,
            Self::Black => Rank::SEVEN,
        }
    }

    #[inline(always)]
    pub const fn pawn_promotion_rank(self) -> Rank {
        match self {
            Self::White => Rank::EIGHT,
            Self::Black => Rank::ONE,
        }
    }

    #[inline(always)]
    pub const fn forward_direction(self) -> i8 {
        match self {
            Self::White => 1,
            Self::Black => -1,
        }
    }

    #[inline(always)]
    pub const fn back_rank(self) -> Rank {
        match self {
            Self::White => Rank::ONE,
            Self::Black => Rank::EIGHT,
        }
    }

    #[inline(always)]
    pub const fn en_passant_rank(self) -> Rank {
        match self {
            Self::White => Rank::SIX,
            Self::Black => Rank::THREE,
        }
    }
}

impl FromStr for Color {
    type Err = CoreError;

    fn from_str(s: &str) -> Result<Self> {
        let chars: Vec<char> = s.chars().collect();
        if chars.len() == 1
            && let Some(c) = Self::from_char(chars[0])
        {
            return Ok(c);
        }

        Err(InvalidColor {
            color: s.to_string(),
        })
    }
}

impl Display for Color {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.as_char())
    }
}

impl Not for Color {
    type Output = Self;

    #[inline(always)]
    fn not(self) -> Self::Output {
        self.opponent()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_color_creation_and_index() {
        assert_eq!(Color::White as usize, 0);
        assert_eq!(Color::Black as usize, 1);
        assert_eq!(Color::from_index(0), Color::White);
        assert_eq!(Color::from_index(1), Color::Black);
    }

    #[test]
    fn test_as_char_and_display() {
        assert_eq!(Color::White.as_char(), 'w');
        assert_eq!(Color::Black.as_char(), 'b');
        assert_eq!(format!("{}", Color::White), "w");
    }

    #[test]
    fn test_from_str() {
        assert_eq!(Color::from_str("w").unwrap(), Color::White);
        assert_eq!(Color::from_str("b").unwrap(), Color::Black);

        assert!(Color::from_str("white").is_err());
        assert!(Color::from_str("W").is_err());
        assert!(Color::from_str("").is_err());
    }

    #[test]
    fn test_opponent_and_not() {
        assert_eq!(Color::White.opponent(), Color::Black);
        assert_eq!(Color::Black.opponent(), Color::White);
        assert_eq!(!Color::White, Color::Black);
        assert_eq!(Color::White.opponent(), Color::Black);
    }

    #[test]
    fn test_directions_and_ranks() {
        assert_eq!(Color::White.forward_direction(), 1);
        assert_eq!(Color::Black.forward_direction(), -1);

        assert_eq!(Color::White.pawn_start_rank(), Rank::TWO);
        assert_eq!(Color::Black.pawn_start_rank(), Rank::SEVEN);

        assert_eq!(Color::White.back_rank(), Rank::ONE);
        assert_eq!(Color::Black.back_rank(), Rank::EIGHT);
    }
}
