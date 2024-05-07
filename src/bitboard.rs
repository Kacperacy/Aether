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
}
