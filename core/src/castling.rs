use crate::CoreError::InvalidCastling;
use crate::{Color, CoreError, Result, Square};
use std::fmt::Display;
use std::ops::*;
use std::str::FromStr;

/// The fixed square geometry of one castling move.
///
/// Shared so that applying a castle (moving the rook) and generating one
/// (checking vacancy and king safety) read from the same source of truth.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CastlingPath {
    pub king_from: Square,
    pub king_to: Square,
    pub rook_from: Square,
    pub rook_to: Square,
    /// Squares between king and rook that must be unoccupied.
    pub vacancy: &'static [Square],
    /// Squares the king stands on or crosses; none may be attacked.
    pub king_safety: &'static [Square],
}

impl CastlingPath {
    pub const WHITE_KINGSIDE: Self = Self {
        king_from: Square::E1,
        king_to: Square::G1,
        rook_from: Square::H1,
        rook_to: Square::F1,
        vacancy: &[Square::F1, Square::G1],
        king_safety: &[Square::E1, Square::F1, Square::G1],
    };

    pub const WHITE_QUEENSIDE: Self = Self {
        king_from: Square::E1,
        king_to: Square::C1,
        rook_from: Square::A1,
        rook_to: Square::D1,
        vacancy: &[Square::D1, Square::C1, Square::B1],
        king_safety: &[Square::E1, Square::D1, Square::C1],
    };

    pub const BLACK_KINGSIDE: Self = Self {
        king_from: Square::E8,
        king_to: Square::G8,
        rook_from: Square::H8,
        rook_to: Square::F8,
        vacancy: &[Square::F8, Square::G8],
        king_safety: &[Square::E8, Square::F8, Square::G8],
    };

    pub const BLACK_QUEENSIDE: Self = Self {
        king_from: Square::E8,
        king_to: Square::C8,
        rook_from: Square::A8,
        rook_to: Square::D8,
        vacancy: &[Square::D8, Square::C8, Square::B8],
        king_safety: &[Square::E8, Square::D8, Square::C8],
    };

    #[inline(always)]
    #[must_use]
    pub const fn kingside(color: Color) -> Self {
        match color {
            Color::White => Self::WHITE_KINGSIDE,
            Color::Black => Self::BLACK_KINGSIDE,
        }
    }

    #[inline(always)]
    #[must_use]
    pub const fn queenside(color: Color) -> Self {
        match color {
            Color::White => Self::WHITE_QUEENSIDE,
            Color::Black => Self::BLACK_QUEENSIDE,
        }
    }

    /// The path `color` takes to reach `king_to`, if that is a castling
    /// destination at all.
    #[inline(always)]
    #[must_use]
    pub const fn for_king_destination(color: Color, king_to: Square) -> Option<Self> {
        let ks = Self::kingside(color);
        let qs = Self::queenside(color);

        if king_to.to_index() == ks.king_to.to_index() {
            Some(ks)
        } else if king_to.to_index() == qs.king_to.to_index() {
            Some(qs)
        } else {
            None
        }
    }
}

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
