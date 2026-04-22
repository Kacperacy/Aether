use crate::CoreError::InvalidSquare;
use crate::{BitBoard, CoreError, File, Rank, Result};
use std::{fmt::Display, str::FromStr};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct Square(u8);

#[rustfmt::skip]
impl Square {
    pub const NUM: usize = 64;

    pub const A1: Square = Square(0);  pub const B1: Square = Square(1);  pub const C1: Square = Square(2);  pub const D1: Square = Square(3);
    pub const E1: Square = Square(4);  pub const F1: Square = Square(5);  pub const G1: Square = Square(6);  pub const H1: Square = Square(7);
    pub const A2: Square = Square(8);  pub const B2: Square = Square(9);  pub const C2: Square = Square(10); pub const D2: Square = Square(11);
    pub const E2: Square = Square(12); pub const F2: Square = Square(13); pub const G2: Square = Square(14); pub const H2: Square = Square(15);
    pub const A3: Square = Square(16); pub const B3: Square = Square(17); pub const C3: Square = Square(18); pub const D3: Square = Square(19);
    pub const E3: Square = Square(20); pub const F3: Square = Square(21); pub const G3: Square = Square(22); pub const H3: Square = Square(23);
    pub const A4: Square = Square(24); pub const B4: Square = Square(25); pub const C4: Square = Square(26); pub const D4: Square = Square(27);
    pub const E4: Square = Square(28); pub const F4: Square = Square(29); pub const G4: Square = Square(30); pub const H4: Square = Square(31);
    pub const A5: Square = Square(32); pub const B5: Square = Square(33); pub const C5: Square = Square(34); pub const D5: Square = Square(35);
    pub const E5: Square = Square(36); pub const F5: Square = Square(37); pub const G5: Square = Square(38); pub const H5: Square = Square(39);
    pub const A6: Square = Square(40); pub const B6: Square = Square(41); pub const C6: Square = Square(42); pub const D6: Square = Square(43);
    pub const E6: Square = Square(44); pub const F6: Square = Square(45); pub const G6: Square = Square(46); pub const H6: Square = Square(47);
    pub const A7: Square = Square(48); pub const B7: Square = Square(49); pub const C7: Square = Square(50); pub const D7: Square = Square(51);
    pub const E7: Square = Square(52); pub const F7: Square = Square(53); pub const G7: Square = Square(54); pub const H7: Square = Square(55);
    pub const A8: Square = Square(56); pub const B8: Square = Square(57); pub const C8: Square = Square(58); pub const D8: Square = Square(59);
    pub const E8: Square = Square(60); pub const F8: Square = Square(61); pub const G8: Square = Square(62); pub const H8: Square = Square(63);

    pub const ALL: [Self; Self::NUM] = [
        Self::A1, Self::B1, Self::C1, Self::D1, Self::E1, Self::F1, Self::G1, Self::H1,
        Self::A2, Self::B2, Self::C2, Self::D2, Self::E2, Self::F2, Self::G2, Self::H2,
        Self::A3, Self::B3, Self::C3, Self::D3, Self::E3, Self::F3, Self::G3, Self::H3,
        Self::A4, Self::B4, Self::C4, Self::D4, Self::E4, Self::F4, Self::G4, Self::H4,
        Self::A5, Self::B5, Self::C5, Self::D5, Self::E5, Self::F5, Self::G5, Self::H5,
        Self::A6, Self::B6, Self::C6, Self::D6, Self::E6, Self::F6, Self::G6, Self::H6,
        Self::A7, Self::B7, Self::C7, Self::D7, Self::E7, Self::F7, Self::G7, Self::H7,
        Self::A8, Self::B8, Self::C8, Self::D8, Self::E8, Self::F8, Self::G8, Self::H8,
    ];
}

impl Square {
    #[inline(always)]
    pub const fn new(file: File, rank: Rank) -> Self {
        Self(rank.to_index() * 8 + file.to_index())
    }

    #[inline(always)]
    pub const fn from_index(index: i8) -> Self {
        debug_assert!(index >= 0 && index < 64, "Square index out of bounds");
        Self(index as u8)
    }

    #[inline(always)]
    pub const fn to_index(self) -> u8 {
        self.0
    }

    #[inline(always)]
    pub const fn file(self) -> File {
        File::from_index((self.0 & 7) as i8)
    }

    #[inline(always)]
    pub const fn rank(self) -> Rank {
        Rank::from_index((self.0 >> 3) as i8)
    }

    #[inline(always)]
    pub const fn bitboard(self) -> BitBoard {
        BitBoard::new(1_u64 << self.0)
    }

    pub const fn offset(self, file_offset: i8, rank_offset: i8) -> Option<Self> {
        let f = self.file().offset(file_offset);
        let r = self.rank().offset(rank_offset);

        match (f, r) {
            (Some(file), Some(rank)) => Some(Self::new(file, rank)),
            _ => None,
        }
    }

    #[inline(always)]
    pub const fn flip(self) -> Self {
        Self(self.0 ^ 56)
    }

    #[inline(always)]
    pub const fn up(self, color: crate::Color) -> Option<Self> {
        self.offset(0, color.forward_direction())
    }

    #[inline(always)]
    pub const fn down(self, color: crate::Color) -> Option<Self> {
        self.offset(0, -color.forward_direction())
    }
}

impl FromStr for Square {
    type Err = CoreError;

    fn from_str(s: &str) -> Result<Self> {
        if s.len() == 2 {
            let f_str = &s[0..1];
            let r_str = &s[1..2];

            if let (Ok(file), Ok(rank)) = (File::from_str(f_str), Rank::from_str(r_str)) {
                return Ok(Self::new(file, rank));
            }
        }

        Err(InvalidSquare {
            square: s.to_string(),
        })
    }
}

impl Display for Square {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}{}", self.file(), self.rank())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_square_creation_and_index() {
        assert_eq!(Square::A1.to_index(), 0);
        assert_eq!(Square::H8.to_index(), 63);
        assert_eq!(Square::from_index(28), Square::E4);

        assert_eq!(Square::new(File::E, Rank::FOUR), Square::E4);
    }

    #[test]
    fn test_file_and_rank() {
        assert_eq!(Square::E4.file(), File::E);
        assert_eq!(Square::E4.rank(), Rank::FOUR);

        assert_eq!(Square::A1.file(), File::A);
        assert_eq!(Square::A1.rank(), Rank::ONE);
    }

    #[test]
    fn test_flip() {
        assert_eq!(Square::E4.flip(), Square::E5);
        assert_eq!(Square::A1.flip(), Square::A8);
        assert_eq!(Square::H8.flip(), Square::H1);
    }

    #[test]
    fn test_offset() {
        assert_eq!(Square::E4.offset(0, 1), Some(Square::E5));
        assert_eq!(Square::E4.offset(1, 0), Some(Square::F4));
        assert_eq!(Square::A1.offset(-1, 0), None);
        assert_eq!(Square::H8.offset(0, 1), None);
    }

    #[test]
    fn test_display_and_from_str() {
        assert_eq!(format!("{}", Square::E4), "e4");
        assert_eq!(Square::from_str("e4").unwrap(), Square::E4);
        assert_eq!(Square::from_str("a1").unwrap(), Square::A1);

        assert!(Square::from_str("i9").is_err());
        assert!(Square::from_str("e").is_err());
        assert!(Square::from_str("e44").is_err());
    }

    #[test]
    fn test_bitboard() {
        assert_eq!(Square::A1.bitboard().value(), 1);
        assert_eq!(Square::B1.bitboard().value(), 2);
        assert_eq!(Square::H8.bitboard().value(), 1_u64 << 63);
    }
}
