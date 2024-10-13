use aether::bitboard::Bitboard;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_bitboard_new() {
        let bb = Bitboard::new();
        assert_eq!(bb.value(), 0);
    }

    #[test]
    fn test_set_bit() {
        let mut bb = Bitboard::new();
        bb.set_bit(3);
        assert_eq!(bb.value(), 8);
    }

    #[test]
    fn test_clear_bit() {
        let mut bb = Bitboard(15);
        bb.clear_bit(3);
        assert_eq!(bb.value(), 7);
    }

    #[test]
    fn test_toggle_bit() {
        let mut bb = Bitboard(0);
        bb.toggle_bit(3);
        assert_eq!(bb.value(), 8);
        bb.toggle_bit(3);
        assert_eq!(bb.value(), 0);
    }

    #[test]
    fn test_is_set() {
        let bb = Bitboard(8);
        assert!(bb.is_set(3));
        assert!(!bb.is_set(2));
    }

    #[test]
    fn test_and() {
        let bb1 = Bitboard(12);
        let bb2 = Bitboard(10);
        assert_eq!(bb1.and(&bb2).value(), 8);
    }

    #[test]
    fn test_or() {
        let bb1 = Bitboard(12);
        let bb2 = Bitboard(10);
        assert_eq!(bb1.or(&bb2).value(), 14);
    }

    #[test]
    fn test_xor() {
        let bb1 = Bitboard(12);
        let bb2 = Bitboard(10);
        assert_eq!(bb1.xor(&bb2).value(), 6);
    }

    #[test]
    fn test_not() {
        let bb = Bitboard(0);
        assert_eq!(bb.not().value(), !0);
    }

    #[test]
    fn test_left_shift() {
        let bb = Bitboard(1);
        assert_eq!(bb.left_shift(3).value(), 8);
    }

    #[test]
    fn test_right_shift() {
        let bb = Bitboard(8);
        assert_eq!(bb.right_shift(3).value(), 1);
    }

    #[test]
    fn test_count_bits() {
        let bb = Bitboard(0b1011);
        assert_eq!(bb.count_bits(), 3);
    }

    #[test]
    fn test_first_set_bit() {
        let bb = Bitboard(0b1010);
        assert_eq!(bb.first_set_bit(), Some(1));
    }

    #[test]
    fn test_last_set_bit() {
        let bb = Bitboard(0b1010);
        assert_eq!(bb.last_set_bit(), Some(3));
    }

    #[test]
    fn test_bitboard_from_index() {
        let bb = Bitboard::from_index(3);
        assert_eq!(bb.value(), 8); // 1 << 3 = 8
    }

    #[test]
    fn test_bitboard_default() {
        let bb: Bitboard = Default::default();
        assert_eq!(bb.value(), 0);
    }
}
