use crate::Square;
use std::fmt;
use std::fmt::Display;
use std::ops::*;

#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Default, Hash)]
pub struct BitBoard(pub u64);

impl Display for BitBoard {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut s: String = String::new();
        s.push('\n');
        for x in 0..64 {
            if self.is_set_index(63 - x) {
                s.push_str("X ");
            } else {
                s.push_str(". ");
            }
            if x % 8 == 7 {
                s.push('\n');
            }
        }
        write!(f, "{s}")
    }
}

impl BitAnd for BitBoard {
    type Output = Self;

    fn bitand(self, rhs: Self) -> Self {
        BitBoard(self.0 & rhs.0)
    }
}

impl BitAndAssign for BitBoard {
    fn bitand_assign(&mut self, rhs: Self) {
        *self = *self & rhs;
    }
}

impl BitOr for BitBoard {
    type Output = Self;

    fn bitor(self, rhs: Self) -> Self {
        BitBoard(self.0 | rhs.0)
    }
}

impl BitOrAssign for BitBoard {
    fn bitor_assign(&mut self, rhs: Self) {
        *self = *self | rhs;
    }
}

impl BitXor for BitBoard {
    type Output = Self;

    fn bitxor(self, rhs: Self) -> Self {
        BitBoard(self.0 ^ rhs.0)
    }
}

impl BitXorAssign for BitBoard {
    fn bitxor_assign(&mut self, rhs: Self) {
        *self = *self ^ rhs;
    }
}

impl Not for BitBoard {
    type Output = Self;

    fn not(self) -> Self {
        BitBoard(!self.0)
    }
}

impl Sub for BitBoard {
    type Output = Self;

    fn sub(self, rhs: Self) -> Self {
        self & !rhs
    }
}

impl SubAssign for BitBoard {
    fn sub_assign(&mut self, rhs: Self) {
        *self = *self - rhs;
    }
}

impl BitBoard {
    pub const EMPTY: Self = Self(0);

    pub const fn new() -> Self {
        Self(0)
    }

    pub const fn value(self) -> u64 {
        self.0
    }

    #[inline(always)]
    pub const fn iter(self) -> BitBoardIter {
        BitBoardIter { bits: self.0 }
    }

    #[inline(always)]
    pub const fn count(self) -> usize {
        self.0.count_ones() as usize
    }

    pub const fn is_empty(self) -> bool {
        self.0 == Self::EMPTY.0
    }

    pub const fn is_set_index(self, index: u8) -> bool {
        self.0 & (1 << index) != 0
    }

    pub const fn has(self, square: Square) -> bool {
        self.is_set_index(square.to_index())
    }

    pub const fn to_square(self) -> Option<Square> {
        if self.is_empty() || self.0.count_ones() != 1 {
            return None;
        }

        let square = Square::from_index(self.0.trailing_zeros() as i8);
        Some(square)
    }

    #[inline(always)]
    pub const fn lsb(self) -> Square {
        debug_assert!(!self.is_empty(), "lsb() called on empty BitBoard");
        Square::from_index(self.0.trailing_zeros() as i8)
    }
}

#[derive(Debug, Clone, Copy)]
pub struct BitBoardIter {
    bits: u64,
}

impl Iterator for BitBoardIter {
    type Item = Square;

    #[inline(always)]
    fn next(&mut self) -> Option<Square> {
        if self.bits == 0 {
            return None;
        }

        let idx = self.bits.trailing_zeros();
        self.bits &= self.bits - 1; // Clear lowest set bit (efficient bit trick)
        Some(Square::from_index(idx as i8))
    }
}
