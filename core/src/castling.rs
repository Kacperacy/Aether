use crate::CoreError::InvalidCastling;
use crate::{Color, CoreError, Result};
use std::fmt::Display;
use std::ops::*;
use std::str::FromStr;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
pub struct CastlingRights(pub u8);

impl CastlingRights {
    // Single rights
    pub const NONE: Self = Self(0);
    pub const WK: Self = Self(0b0001); // White Kingside
    pub const WQ: Self = Self(0b0010); // White Queenside
    pub const BK: Self = Self(0b0100); // Black Kingside
    pub const BQ: Self = Self(0b1000); // Black Queenside

    // Grouped masks
    pub const WHITE: Self = Self(0b0011);
    pub const BLACK: Self = Self(0b1100);
    pub const ALL: Self = Self(0b1111);

    #[inline(always)]
    pub const fn new(val: u8) -> Self {
        Self(val)
    }

    #[inline(always)]
    pub const fn kingside(color: Color) -> Self {
        match color {
            Color::White => Self::WK,
            Color::Black => Self::BK,
        }
    }

    #[inline(always)]
    pub const fn queenside(color: Color) -> Self {
        match color {
            Color::White => Self::WQ,
            Color::Black => Self::BQ,
        }
    }

    #[inline(always)]
    pub const fn for_color(color: Color) -> Self {
        match color {
            Color::White => Self::WHITE,
            Color::Black => Self::BLACK,
        }
    }

    #[inline(always)]
    pub const fn value(self) -> u8 {
        self.0
    }

    #[inline(always)]
    pub const fn is_empty(self) -> bool {
        self.0 == 0
    }

    #[inline(always)]
    pub const fn contains(self, rights: Self) -> bool {
        (self.0 & rights.0) == rights.0
    }

    #[inline(always)]
    pub const fn any(self, rights: Self) -> bool {
        (self.0 & rights.0) != 0
    }

    #[inline(always)]
    pub fn add(&mut self, rights: Self) {
        self.0 |= rights.0;
    }

    #[inline(always)]
    pub fn remove(&mut self, rights: Self) {
        self.0 &= !rights.0;
    }
}

// Operator overloads for clean bitwise math
impl BitOr for CastlingRights {
    type Output = Self;
    #[inline(always)]
    fn bitor(self, rhs: Self) -> Self::Output {
        Self(self.0 | rhs.0)
    }
}

impl BitOrAssign for CastlingRights {
    #[inline(always)]
    fn bitor_assign(&mut self, rhs: Self) {
        self.0 |= rhs.0;
    }
}

impl BitAnd for CastlingRights {
    type Output = Self;
    #[inline(always)]
    fn bitand(self, rhs: Self) -> Self::Output {
        Self(self.0 & rhs.0)
    }
}

impl BitAndAssign for CastlingRights {
    #[inline(always)]
    fn bitand_assign(&mut self, rhs: Self) {
        self.0 &= rhs.0;
    }
}

impl Not for CastlingRights {
    type Output = Self;
    #[inline(always)]
    fn not(self) -> Self::Output {
        // Negate but mask out anything above the 4th bit
        Self(!self.0 & Self::ALL.0)
    }
}

impl FromStr for CastlingRights {
    type Err = CoreError;

    fn from_str(s: &str) -> Result<Self> {
        if s == "-" {
            return Ok(Self::NONE);
        }

        let mut rights = Self::NONE;
        for c in s.chars() {
            match c {
                'K' => rights |= Self::WK,
                'Q' => rights |= Self::WQ,
                'k' => rights |= Self::BK,
                'q' => rights |= Self::BQ,
                _ => {
                    return Err(InvalidCastling {
                        rights: s.to_string(),
                    });
                }
            }
        }
        Ok(rights)
    }
}

impl Display for CastlingRights {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if self.is_empty() {
            return write!(f, "-");
        }

        let mut s = String::with_capacity(4);
        if self.contains(Self::WK) {
            s.push('K');
        }
        if self.contains(Self::WQ) {
            s.push('Q');
        }
        if self.contains(Self::BK) {
            s.push('k');
        }
        if self.contains(Self::BQ) {
            s.push('q');
        }

        write!(f, "{}", s)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_castling_creation_and_values() {
        assert_eq!(CastlingRights::NONE.value(), 0);
        assert_eq!(CastlingRights::ALL.value(), 15);
        assert_eq!(CastlingRights::WHITE.value(), 3);
    }

    #[test]
    fn test_contains_and_any() {
        let rights = CastlingRights::WK | CastlingRights::BQ;

        assert!(rights.contains(CastlingRights::WK));
        assert!(rights.contains(CastlingRights::BQ));
        assert!(!rights.contains(CastlingRights::WQ));
        assert!(!rights.contains(CastlingRights::BK));

        assert!(rights.any(CastlingRights::BLACK));
        assert!(rights.any(CastlingRights::WHITE));
    }

    #[test]
    fn test_bitwise_operations() {
        let rights = CastlingRights::WK | CastlingRights::WQ;
        assert_eq!(rights, CastlingRights::WHITE);

        let masked = rights & CastlingRights::WK;
        assert_eq!(masked, CastlingRights::WK);

        assert_eq!(!CastlingRights::WHITE, CastlingRights::BLACK);
    }

    #[test]
    fn test_from_str() {
        assert_eq!(CastlingRights::from_str("-").unwrap(), CastlingRights::NONE);
        assert_eq!(
            CastlingRights::from_str("KQkq").unwrap(),
            CastlingRights::ALL
        );
        assert_eq!(
            CastlingRights::from_str("Kq").unwrap(),
            CastlingRights::WK | CastlingRights::BQ
        );

        assert!(CastlingRights::from_str("X").is_err());
    }

    #[test]
    fn test_display() {
        assert_eq!(format!("{}", CastlingRights::NONE), "-");
        assert_eq!(format!("{}", CastlingRights::ALL), "KQkq");
        assert_eq!(format!("{}", CastlingRights::WK | CastlingRights::BK), "Kk");
    }
}
