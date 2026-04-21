use crate::Square;
use std::fmt::{self, Display};
use std::ops::*;

#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Default, Hash)]
pub struct BitBoard(pub u64);

impl BitBoard {
    pub const EMPTY: Self = Self(0);
    pub const UNIVERSE: Self = Self(!0);

    #[inline(always)]
    pub const fn new(val: u64) -> Self {
        Self(val)
    }

    #[inline(always)]
    pub const fn value(self) -> u64 {
        self.0
    }

    #[inline(always)]
    pub const fn is_empty(self) -> bool {
        self.0 == 0
    }

    #[inline(always)]
    pub const fn count(self) -> usize {
        self.0.count_ones() as usize
    }

    #[inline(always)]
    pub fn set(&mut self, square: Square) {
        self.0 |= 1 << square.to_index();
    }

    #[inline(always)]
    pub fn clear(&mut self, square: Square) {
        self.0 &= !(1 << square.to_index());
    }

    #[inline(always)]
    pub fn contains(self, square: Square) -> bool {
        (self.0 & (1 << square.to_index())) != 0
    }

    #[inline(always)]
    pub const fn lsb(self) -> Square {
        debug_assert!(!self.is_empty(), "Can't invoke lsb() on empty BitBoard");
        Square::from_index(self.0.trailing_zeros() as i8)
    }

    #[inline(always)]
    pub fn pop_lsb(&mut self) -> Square {
        let sq = self.lsb();
        self.0 &= self.0 - 1;
        sq
    }

    #[inline(always)]
    pub const fn iter(self) -> BitBoardIter {
        BitBoardIter { bits: self.0 }
    }
}

impl BitAnd for BitBoard {
    type Output = Self;

    #[inline(always)]
    fn bitand(self, rhs: Self) -> Self {
        BitBoard(self.0 & rhs.0)
    }
}

impl BitAndAssign for BitBoard {
    #[inline(always)]
    fn bitand_assign(&mut self, rhs: Self) {
        self.0 &= rhs.0;
    }
}

impl BitOr for BitBoard {
    type Output = Self;

    #[inline(always)]
    fn bitor(self, rhs: Self) -> Self {
        BitBoard(self.0 | rhs.0)
    }
}

impl BitOrAssign for BitBoard {
    #[inline(always)]
    fn bitor_assign(&mut self, rhs: Self) {
        self.0 |= rhs.0;
    }
}

impl BitXor for BitBoard {
    type Output = Self;

    #[inline(always)]
    fn bitxor(self, rhs: Self) -> Self {
        BitBoard(self.0 ^ rhs.0)
    }
}

impl BitXorAssign for BitBoard {
    #[inline(always)]
    fn bitxor_assign(&mut self, rhs: Self) {
        self.0 ^= rhs.0;
    }
}

impl Not for BitBoard {
    type Output = Self;

    #[inline(always)]
    fn not(self) -> Self {
        BitBoard(!self.0)
    }
}

impl Sub for BitBoard {
    type Output = Self;

    #[inline(always)]
    fn sub(self, rhs: Self) -> Self {
        BitBoard(self.0 & !rhs.0)
    }
}

impl SubAssign for BitBoard {
    #[inline(always)]
    fn sub_assign(&mut self, rhs: Self) {
        self.0 &= !rhs.0;
    }
}

macro_rules! impl_bit_shifts {
    ($($t:ty),*) => {
        $(
            impl Shl<$t> for BitBoard {
                type Output = Self;
                #[inline(always)]
                fn shl(self, rhs: $t) -> Self {
                    BitBoard(self.0 << rhs)
                }
            }

            impl Shr<$t> for BitBoard {
                type Output = Self;
                #[inline(always)]
                fn shr(self, rhs: $t) -> Self {
                    BitBoard(self.0 >> rhs)
                }
            }
        )*
    };
}

impl_bit_shifts!(u8, u16, u32, u64, usize, i8, i16, i32, i64, isize);

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

impl Display for BitBoard {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(f)?;
        for rank in (0..8).rev() {
            for file in 0..8 {
                let sq = rank * 8 + file;
                if (self.0 & (1 << sq)) != 0 {
                    write!(f, "X ")?;
                } else {
                    write!(f, ". ")?;
                }
            }
            writeln!(f)?;
        }
        Ok(())
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::Square;

    #[test]
    fn test_new_and_empty() {
        let bb = BitBoard::EMPTY;
        assert_eq!(bb.value(), 0);
        assert!(bb.is_empty());
        assert_eq!(bb.count(), 0);

        let bb2 = BitBoard::new(0b101);
        assert!(!bb2.is_empty());
        assert_eq!(bb.count(), 2);
    }

    #[test]
    fn test_set_clear_contains() {
        let mut bb = BitBoard::EMPTY;

        assert!(!bb.contains(Square::A1));
        bb.set(Square::A1);
        assert!(bb.contains(Square::A1));
        assert_eq!(bb.value(), 1);

        bb.set(Square::B1);
        assert!(bb.contains(Square::B1));
        assert_eq!(bb.count(), 2);

        bb.clear(Square::A1);
        assert!(!bb.contains(Square::A1));
        assert!(bb.contains(Square::B1));
        assert_eq!(bb.count(), 1);
    }

    #[test]
    fn test_lsb_and_pop_lsb() {
        let mut bb = BitBoard::new((1 << 3) | (1 << 8));

        assert_eq!(bb.lsb(), Square::D1);
        assert_eq!(bb.count(), 2);

        assert_eq!(bb.pop_lsb(), Square::D1);
        assert_eq!(bb.count(), 1);
        assert!(!bb.contains(Square::D1));

        assert_eq!(bb.lsb(), Square::A2);
        assert_eq!(bb.pop_lsb(), Square::A2);

        assert!(bb.is_empty());
    }

    #[test]
    fn test_bitwise_operations() {
        let bb1 = BitBoard::new(0b1010);
        let bb2 = BitBoard::new(0b1100);

        assert_eq!((bb1 & bb2).value(), 0b1000);
        assert_eq!((bb1 | bb2).value(), 0b1110);
        assert_eq!((bb1 ^ bb2).value(), 0b0110);

        assert_eq!((bb1 - bb2).value(), 0b0010);
    }

    #[test]
    fn test_bit_shift() {
        let mut bb = BitBoard::EMPTY;
        bb.set(Square::A1);

        let shifted_left = bb << 8_u8;
        assert!(shifted_left.contains(Square::A2));
        assert_eq!(shifted_left.value(), 256);

        let shifted_right = shifted_left >> 8_u8;
        assert!(shifted_right.contains(Square::A1));
        assert_eq!(shifted_right.value(), 1);
    }

    #[test]
    fn test_iterator() {
        let bb = BitBoard::new((1 << 0) | (1 << 9) | (1 << 63));

        let mut iter = bb.iter();

        assert_eq!(iter.next(), Some(Square::A1));
        assert_eq!(iter.next(), Some(Square::B2));
        assert_eq!(iter.next(), Some(Square::H8));
        assert_eq!(iter.next(), None);
    }
}
