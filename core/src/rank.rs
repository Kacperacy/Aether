use crate::CoreError::InvalidRank;
use crate::{BitBoard, Color, CoreError, Result};
use std::fmt::Display;
use std::str::FromStr;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct Rank(u8);

impl Rank {
    pub const NUM: usize = 8;

    pub const ONE: Rank = Rank(0);
    pub const TWO: Rank = Rank(1);
    pub const THREE: Rank = Rank(2);
    pub const FOUR: Rank = Rank(3);
    pub const FIVE: Rank = Rank(4);
    pub const SIX: Rank = Rank(5);
    pub const SEVEN: Rank = Rank(6);
    pub const EIGHT: Rank = Rank(7);

    pub const ALL: [Self; Self::NUM] = [
        Self::ONE,
        Self::TWO,
        Self::THREE,
        Self::FOUR,
        Self::FIVE,
        Self::SIX,
        Self::SEVEN,
        Self::EIGHT,
    ];

    #[inline(always)]
    pub const fn from_index(index: i8) -> Self {
        debug_assert!(index >= 0 && index < 8, "Rank index out of bounds");
        Self(index as u8)
    }

    #[inline(always)]
    pub const fn to_index(self) -> u8 {
        self.0
    }

    #[inline]
    pub const fn as_char(self) -> char {
        (b'1' + self.0) as char
    }

    pub const fn offset(self, offset: i8) -> Option<Self> {
        let new_rank = self.0 as i8 + offset;
        if new_rank < 0 || new_rank > 7 {
            None
        } else {
            Some(Self(new_rank as u8))
        }
    }

    #[inline(always)]
    pub const fn flip(self) -> Self {
        Self(7 - self.0)
    }

    #[inline(always)]
    pub const fn bitboard(self) -> BitBoard {
        BitBoard::new(0xFF_u64 << (self.0 * 8))
    }

    pub const fn relative_to(self, color: Color) -> Self {
        match color {
            Color::White => self,
            Color::Black => self.flip(),
        }
    }
}

impl FromStr for Rank {
    type Err = CoreError;

    fn from_str(s: &str) -> Result<Self> {
        let chars: Vec<char> = s.chars().collect();
        if chars.len() == 1 && chars[0] >= '1' && chars[0] <= '8' {
            Ok(Self((chars[0] as u8) - b'1'))
        } else {
            Err(InvalidRank {
                rank: s.to_string(),
            })
        }
    }
}

impl Display for Rank {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.as_char())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Color;

    #[test]
    fn test_rank_creation_and_index() {
        assert_eq!(Rank::ONE.to_index(), 0);
        assert_eq!(Rank::EIGHT.to_index(), 7);
        assert_eq!(Rank::from_index(3), Rank::FOUR);
    }

    #[test]
    fn test_as_char_and_display() {
        assert_eq!(Rank::ONE.as_char(), '1');
        assert_eq!(Rank::EIGHT.as_char(), '8');
        assert_eq!(format!("{}", Rank::FIVE), "5");
    }

    #[test]
    fn test_from_str() {
        assert_eq!(Rank::from_str("1").unwrap(), Rank::ONE);
        assert_eq!(Rank::from_str("8").unwrap(), Rank::EIGHT);

        assert!(Rank::from_str("9").is_err());
        assert!(Rank::from_str("0").is_err());
        assert!(Rank::from_str("12").is_err());
        assert!(Rank::from_str("a").is_err());
    }

    #[test]
    fn test_offset() {
        assert_eq!(Rank::FOUR.offset(1), Some(Rank::FIVE));
        assert_eq!(Rank::FOUR.offset(-1), Some(Rank::THREE));
        assert_eq!(Rank::FOUR.offset(4), Some(Rank::EIGHT));

        assert_eq!(Rank::ONE.offset(-1), None);
        assert_eq!(Rank::ONE.offset(-5), None);
        assert_eq!(Rank::EIGHT.offset(1), None);
        assert_eq!(Rank::EIGHT.offset(5), None);
    }

    #[test]
    fn test_flip() {
        assert_eq!(Rank::ONE.flip(), Rank::EIGHT);
        assert_eq!(Rank::TWO.flip(), Rank::SEVEN);
        assert_eq!(Rank::THREE.flip(), Rank::SIX);
        assert_eq!(Rank::FOUR.flip(), Rank::FIVE);
    }

    #[test]
    fn test_bitboard() {
        assert_eq!(Rank::ONE.bitboard().value(), 0x00000000000000FF);
        assert_eq!(Rank::TWO.bitboard().value(), 0x000000000000FF00);
        assert_eq!(Rank::EIGHT.bitboard().value(), 0xFF00000000000000);
    }

    #[test]
    fn test_relative_to() {
        assert_eq!(Rank::TWO.relative_to(Color::White), Rank::TWO);

        assert_eq!(Rank::TWO.relative_to(Color::Black), Rank::SEVEN);

        assert_eq!(Rank::EIGHT.relative_to(Color::Black), Rank::ONE);
    }
}
