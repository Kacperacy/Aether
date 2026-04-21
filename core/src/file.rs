use crate::CoreError::InvalidFile;
use crate::{BitBoard, CoreError, Result};
use std::fmt::Display;
use std::str::FromStr;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct File(u8);

impl File {
    pub const NUM: usize = 8;

    pub const A: File = File(0);
    pub const B: File = File(1);
    pub const C: File = File(2);
    pub const D: File = File(3);
    pub const E: File = File(4);
    pub const F: File = File(5);
    pub const G: File = File(6);
    pub const H: File = File(7);

    pub const ALL: [Self; 8] = [
        Self::A,
        Self::B,
        Self::C,
        Self::D,
        Self::E,
        Self::F,
        Self::G,
        Self::H,
    ];

    #[inline(always)]
    pub const fn from_index(index: i8) -> Self {
        debug_assert!(index >= 0 && index < 8, "File index out of bounds");
        Self(index as u8)
    }

    #[inline(always)]
    pub const fn to_index(self) -> u8 {
        self.0
    }

    #[inline(always)]
    pub const fn as_char(self) -> char {
        (b'a' + self.0) as char
    }

    pub const fn offset(self, offset: i8) -> Option<Self> {
        let new_file = self.0 as i8 + offset;
        if new_file < 0 || new_file > 7 {
            None
        } else {
            Some(Self(new_file as u8))
        }
    }

    #[inline(always)]
    pub const fn flip(self) -> Self {
        Self(7 - self.0)
    }

    #[inline(always)]
    pub const fn bitboard(self) -> BitBoard {
        BitBoard::new(0x0101010101010101_u64 << self.0)
    }

    #[inline(always)]
    pub const fn adjacent(self) -> BitBoard {
        let bb = self.bitboard().value();
        let mut adj = 0;
        if self.0 > 0 {
            adj |= bb >> 1;
        }
        if self.0 < 7 {
            adj |= bb << 1;
        }
        BitBoard::new(adj)
    }
}

impl FromStr for File {
    type Err = CoreError;

    fn from_str(s: &str) -> Result<Self> {
        let chars: Vec<char> = s.chars().collect();
        if chars.len() == 1 && chars[0] >= 'a' && chars[0] <= 'h' {
            Ok(Self((chars[0] as u8) - b'a'))
        } else {
            Err(InvalidFile {
                file: s.to_string(),
            })
        }
    }
}

impl Display for File {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.as_char())
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_file_creation_and_index() {
        assert_eq!(File::A.to_index(), 0);
        assert_eq!(File::H.to_index(), 7);
        assert_eq!(File::from_index(3), File::D);
        assert_eq!(File::from_index(7), File::H);
    }

    #[test]
    fn test_as_char_and_display() {
        assert_eq!(File::A.as_char(), 'a');
        assert_eq!(File::E.as_char(), 'e');
        assert_eq!(File::H.as_char(), 'h');

        assert_eq!(format!("{}", File::C), "c");
    }

    #[test]
    fn test_from_str() {
        assert_eq!(File::from_str("a").unwrap(), File::A);
        assert_eq!(File::from_str("h").unwrap(), File::H);

        assert!(File::from_str("i").is_err());
        assert!(File::from_str("A").is_err());
        assert!(File::from_str("ab").is_err());
    }

    #[test]
    fn test_offset() {
        assert_eq!(File::D.offset(1), Some(File::E));
        assert_eq!(File::D.offset(-1), Some(File::C));
        assert_eq!(File::D.offset(4), Some(File::H));

        assert_eq!(File::A.offset(-1), None);
        assert_eq!(File::A.offset(-5), None);
        assert_eq!(File::H.offset(1), None);
        assert_eq!(File::H.offset(5), None);
    }

    #[test]
    fn test_flip() {
        assert_eq!(File::A.flip(), File::H);
        assert_eq!(File::B.flip(), File::G);
        assert_eq!(File::C.flip(), File::F);
        assert_eq!(File::D.flip(), File::E);
    }

    #[test]
    fn test_bitboard() {
        assert_eq!(File::A.bitboard().value(), 0x0101010101010101);
        assert_eq!(File::B.bitboard().value(), 0x0202020202020202);
        assert_eq!(File::H.bitboard().value(), 0x0808080808080808);
    }

    #[test]
    fn test_adjacent() {
        assert_eq!(File::A.adjacent().value(), File::B.bitboard().value());

        let expected_b_adj = (File::A.bitboard() | File::C.bitboard()).value();
        assert_eq!(File::B.adjacent().value(), expected_b_adj);

        assert_eq!(File::H.adjacent().value(), File::G.bitboard().value());
    }
}
