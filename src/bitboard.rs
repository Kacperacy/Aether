use std::ops::{BitAnd, BitOr, BitXor, Not, Shl, Shr};

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Bitboard(pub u64);

impl Bitboard {
    pub fn new() -> Self {
        Bitboard(0)
    }

    pub fn value(&self) -> u64 {
        self.0
    }

    pub fn set_bit(&mut self, index: usize) {
        self.0 |= 1 << index;
    }

    pub fn clear_bit(&mut self, index: usize) {
        self.0 &= !(1 << index);
    }

    pub fn toggle_bit(&mut self, index: usize) {
        self.0 ^= 1 << index;
    }

    pub fn is_set(&self, index: usize) -> bool {
        (self.0 & (1 << index)) != 0
    }

    pub fn and(&self, other: &Bitboard) -> Bitboard {
        Bitboard(self.0 & other.0)
    }

    pub fn or(&self, other: &Bitboard) -> Bitboard {
        Bitboard(self.0 | other.0)
    }

    pub fn xor(&self, other: &Bitboard) -> Bitboard {
        Bitboard(self.0 ^ other.0)
    }

    pub fn not(&self) -> Bitboard {
        Bitboard(!self.0)
    }

    pub fn left_shift(&self, shift: u32) -> Bitboard {
        Bitboard(self.0 << shift)
    }

    pub fn right_shift(&self, shift: u32) -> Bitboard {
        Bitboard(self.0 >> shift)
    }

    pub fn count_bits(&self) -> u32 {
        self.0.count_ones()
    }

    pub fn first_set_bit(&self) -> Option<usize> {
        if self.0 == 0 {
            None
        } else {
            Some(self.0.trailing_zeros() as usize)
        }
    }

    pub fn last_set_bit(&self) -> Option<usize> {
        if self.0 == 0 {
            None
        } else {
            Some(63 - self.0.leading_zeros() as usize)
        }
    }
}

impl BitAnd for Bitboard {
    type Output = Bitboard;

    fn bitand(self, rhs: Bitboard) -> Bitboard {
        Bitboard(self.0 & rhs.0)
    }
}

impl BitOr for Bitboard {
    type Output = Bitboard;

    fn bitor(self, rhs: Bitboard) -> Bitboard {
        Bitboard(self.0 | rhs.0)
    }
}

impl BitXor for Bitboard {
    type Output = Bitboard;

    fn bitxor(self, rhs: Bitboard) -> Bitboard {
        Bitboard(self.0 ^ rhs.0)
    }
}

impl Not for Bitboard {
    type Output = Bitboard;

    fn not(self) -> Bitboard {
        Bitboard(!self.0)
    }
}

impl Shl<u32> for Bitboard {
    type Output = Bitboard;

    fn shl(self, shift: u32) -> Bitboard {
        Bitboard(self.0 << shift)
    }
}

impl Shr<u32> for Bitboard {
    type Output = Bitboard;

    fn shr(self, shift: u32) -> Bitboard {
        Bitboard(self.0 >> shift)
    }
}