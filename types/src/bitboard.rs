use crate::{File, Rank, Square};
use std::fmt;
use std::fmt::Display;
use std::ops::*;

/// A bitboard representing a set of squares on a chess board.
///
/// Each bit in the underlying `u64` represents whether a specific square is set.
/// Bit 0 represents A1, bit 1 represents B1, ..., bit 63 represents H8.
///
/// # Examples
///
/// ```
/// use aether_types::{BitBoard, Square};
///
/// let mut bb = BitBoard::new();
/// bb = bb | BitBoard::from_square(Square::E4);
/// assert!(bb.has(Square::E4));
/// ```
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
    /// Prints the bitboard to stdout in a visual chess board format.
    ///
    /// Displays 'X' for set bits and '.' for unset bits, with rank and file labels.
    pub fn print(&self) {
        println!("  +-----------------+");
        for rank in (0..8).rev() {
            print!("{} | ", rank + 1);
            for file in 0..8 {
                let square = Square::new(File::from_index(file), Rank::new(rank));
                if self.has(square) {
                    print!("X ");
                } else {
                    print!(". ");
                }
            }
            println!("|");
        }
        println!("  +-----------------+");
        println!("    a b c d e f g h");
    }

    /// An empty bitboard (no squares set).
    pub const EMPTY: Self = Self(0);

    /// A full bitboard (all squares set).
    pub const FULL: Self = Self(!0);

    /// Bitboard representing all edge squares (rank 1, rank 8, file a, file h).
    pub const EDGES: Self = Self(0xff818181818181ff);

    /// Bitboard representing the four corner squares (a1, h1, a8, h8).
    pub const CORNERS: Self = Self(0x8100000000000081);

    /// Bitboard representing all white (light) squares on the board.
    pub const WHITE_SQUARES: Self = Self(0x55aa55aa55aa55aa);

    /// Bitboard representing all black (dark) squares on the board.
    pub const BLACK_SQUARES: Self = Self(!0x55aa55aa55aa55aa);

    /// Creates an empty bitboard.
    pub const fn new() -> Self {
        Self(0)
    }

    /// Returns the underlying u64 representation.
    pub const fn value(self) -> u64 {
        self.0
    }

    /// Flips the bitboard vertically (mirrors across the horizontal axis).
    ///
    /// Rank 1 becomes rank 8, rank 2 becomes rank 7, etc.
    pub const fn flip_rank(self) -> Self {
        Self(self.0.swap_bytes())
    }

    /// Flips the bitboard horizontally (mirrors across the vertical axis).
    ///
    /// File a becomes file h, file b becomes file g, etc.
    pub const fn flip_file(self) -> Self {
        const K1: u64 = 0x5555555555555555;
        const K2: u64 = 0x3333333333333333;
        const K4: u64 = 0x0f0f0f0f0f0f0f0f;
        let mut x = self.0;
        x = ((x >> 1) & K1) | ((x & K1) << 1);
        x = ((x >> 2) & K2) | ((x & K2) << 2);
        x = ((x >> 4) & K4) | ((x & K4) << 4);
        Self(x)
    }

    /// Returns the number of set bits (population count).
    pub const fn len(self) -> u32 {
        self.0.count_ones()
    }

    /// Returns `true` if no squares are set.
    pub const fn is_empty(self) -> bool {
        self.0 == Self::EMPTY.0
    }

    /// Returns `true` if this bitboard is a subset of `other`.
    ///
    /// All set bits in `self` must also be set in `other`.
    pub const fn is_subset(self, other: BitBoard) -> bool {
        self.0 & !other.0 == 0
    }

    /// Returns `true` if this bitboard is a superset of `other`.
    ///
    /// All set bits in `other` must also be set in `self`.
    pub const fn is_superset(self, other: BitBoard) -> bool {
        other.is_subset(self)
    }

    /// Returns `true` if the bit at the given index (0-63) is set.
    pub const fn is_set_index(self, index: u8) -> bool {
        self.0 & (1 << index) != 0
    }

    /// Returns `true` if this bitboard has any overlap with `other`.
    ///
    /// Returns `true` if at least one bit is set in both bitboards.
    pub const fn contains(self, other: BitBoard) -> bool {
        self.0 & other.0 != Self::EMPTY.0
    }

    /// Returns `true` if the given square is set in this bitboard.
    pub const fn has(self, square: Square) -> bool {
        self.contains(square.bitboard())
    }

    /// Reverses all bits (bitwise NOT followed by bit reversal).
    pub const fn reverse(self) -> Self {
        Self(self.0.reverse_bits())
    }

    /// Creates a bitboard with only the given square set.
    pub const fn from_square(square: Square) -> Self {
        square.bitboard()
    }

    /// Converts a bitboard to a square if exactly one bit is set.
    ///
    /// Returns `None` if the bitboard is empty or has more than one bit set.
    pub const fn to_square(self) -> Option<Square> {
        if self.is_empty() || self.0.count_ones() != 1 {
            return None;
        }

        let square = Square::from_index(self.0.trailing_zeros() as i8);
        Some(square)
    }

    /// Returns the first (least significant) set square without removing it.
    ///
    /// Returns `None` if the bitboard is empty.
    /// This is used internally by the Iterator implementation.
    pub const fn next_square(self) -> Option<Square> {
        if self.is_empty() {
            return None;
        }

        let square = Square::from_index(self.0.trailing_zeros() as i8);
        Some(square)
    }
}

impl Iterator for BitBoard {
    type Item = Square;

    fn next(&mut self) -> Option<Square> {
        let square = self.next_square();
        *self -= square?.bitboard();
        square
    }
}
