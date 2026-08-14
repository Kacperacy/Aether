use crate::{Piece, Square};
use std::fmt::{Display, Formatter, Result};

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Move(pub u16);

impl Move {
    pub const QUIET: u16 = 0;
    pub const DOUBLE_PUSH: u16 = 1;
    pub const CASTLE_KS: u16 = 2; // Kingside (Short)
    pub const CASTLE_QS: u16 = 3; // Queenside (Long)

    pub const CAPTURE: u16 = 4;
    pub const EN_PASSANT: u16 = 5;

    pub const PROMO_N: u16 = 8;
    pub const PROMO_B: u16 = 9;
    pub const PROMO_R: u16 = 10;
    pub const PROMO_Q: u16 = 11;

    pub const PROMO_CAP_N: u16 = 12;
    pub const PROMO_CAP_B: u16 = 13;
    pub const PROMO_CAP_R: u16 = 14;
    pub const PROMO_CAP_Q: u16 = 15;

    pub const NULL: Self = Self(0);

    #[inline(always)]
    pub const fn new(from: Square, to: Square, flags: u16) -> Self {
        Self(from.to_index() as u16 | ((to.to_index() as u16) << 6) | (flags << 12))
    }

    #[inline(always)]
    pub const fn from_sq(self) -> Square {
        Square::from_index((self.0 & 0x3F) as i8)
    }

    #[inline(always)]
    pub const fn to_sq(self) -> Square {
        Square::from_index(((self.0 >> 6) & 0x3F) as i8)
    }

    #[inline(always)]
    pub const fn flags(self) -> u16 {
        self.0 >> 12
    }

    #[inline(always)]
    pub const fn is_capture(self) -> bool {
        (self.flags() & Self::CAPTURE) != 0
    }

    #[inline(always)]
    pub const fn is_promotion(self) -> bool {
        (self.flags() & 8) != 0
    }

    #[inline(always)]
    pub const fn is_en_passant(self) -> bool {
        self.flags() == Self::EN_PASSANT
    }

    #[inline(always)]
    pub const fn is_castling(self) -> bool {
        let f = self.flags();
        f == Self::CASTLE_KS || f == Self::CASTLE_QS
    }

    #[inline(always)]
    pub const fn promotion_piece(self) -> Option<Piece> {
        if self.is_promotion() {
            // Extract the lowest 2 bits of the flag and map it directly to Piece
            let piece_idx = (self.flags() & 3) + 1;
            Some(Piece::from_index(piece_idx as u8))
        } else {
            None
        }
    }
}

impl Display for Move {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        if *self == Self::NULL {
            return write!(f, "0000");
        }

        // Print the move in UCI format (e.g., "e2e4", "e7e8q")
        if let Some(piece) = self.promotion_piece() {
            write!(f, "{}{}{}", self.from_sq(), self.to_sq(), piece.as_char())
        } else {
            write!(f, "{}{}", self.from_sq(), self.to_sq())
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::File;
    use crate::Rank;

    #[test]
    fn test_move_creation_and_extraction() {
        let e2 = Square::new(File::E, Rank::TWO);
        let e4 = Square::new(File::E, Rank::FOUR);

        let m = Move::new(e2, e4, Move::DOUBLE_PUSH);

        assert_eq!(m.from_sq(), e2);
        assert_eq!(m.to_sq(), e4);
        assert_eq!(m.flags(), Move::DOUBLE_PUSH);
    }

    #[test]
    fn test_capture_and_promotion_flags() {
        let dummy_sq = Square::A1;

        let quiet = Move::new(dummy_sq, dummy_sq, Move::QUIET);
        assert!(!quiet.is_capture());
        assert!(!quiet.is_promotion());

        let capture = Move::new(dummy_sq, dummy_sq, Move::CAPTURE);
        assert!(capture.is_capture());
        assert!(!capture.is_promotion());

        let promo_q = Move::new(dummy_sq, dummy_sq, Move::PROMO_Q);
        assert!(!promo_q.is_capture());
        assert!(promo_q.is_promotion());
        assert_eq!(promo_q.promotion_piece(), Some(Piece::Queen));

        let promo_cap_n = Move::new(dummy_sq, dummy_sq, Move::PROMO_CAP_N);
        assert!(promo_cap_n.is_capture());
        assert!(promo_cap_n.is_promotion());
        assert_eq!(promo_cap_n.promotion_piece(), Some(Piece::Knight));
    }

    #[test]
    fn test_uci_formatting() {
        let e2 = Square::new(File::E, Rank::TWO);
        let e4 = Square::new(File::E, Rank::FOUR);
        let normal_move = Move::new(e2, e4, Move::QUIET);
        assert_eq!(format!("{}", normal_move), "e2e4");

        let e7 = Square::new(File::E, Rank::SEVEN);
        let e8 = Square::new(File::E, Rank::EIGHT);
        let promo_move = Move::new(e7, e8, Move::PROMO_Q);
        assert_eq!(format!("{}", promo_move), "e7e8q");
    }
}
