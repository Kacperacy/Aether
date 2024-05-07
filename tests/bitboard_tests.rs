use aether::bitboard::Bitboard;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_set_bit() {
        let mut bitboard = Bitboard::new();
        bitboard.set_bit(4);
        assert!(bitboard.is_set(4));
    }

    #[test]
    fn test_clear_bit() {
        let mut bitboard = Bitboard::new();
        bitboard.set_bit(4);
        bitboard.clear_bit(4);
        assert!(!bitboard.is_set(4));
    }

    #[test]
    fn test_toggle_bit() {
        let mut bitboard = Bitboard::new();
        bitboard.toggle_bit(4);
        assert!(bitboard.is_set(4));
        bitboard.toggle_bit(4);
        assert!(!bitboard.is_set(4));
    }

    #[test]
    fn test_and() {
        let bitboard1 = Bitboard(0b1100);
        let bitboard2 = Bitboard(0b1010);
        let result = bitboard1.and(&bitboard2);
        assert_eq!(result.value(), 0b1000);
    }

    #[test]
    fn test_or() {
        let bitboard1 = Bitboard(0b1100);
        let bitboard2 = Bitboard(0b1010);
        let result = bitboard1.or(&bitboard2);
        assert_eq!(result.value(), 0b1110);
    }

    #[test]
    fn test_xor() {
        let bitboard1 = Bitboard(0b1100);
        let bitboard2 = Bitboard(0b1010);
        let result = bitboard1.xor(&bitboard2);
        assert_eq!(result.value(), 0b0110);
    }

    #[test]
    fn test_not() {
        let bitboard = Bitboard(0b1100);
        let result = bitboard.not();
        assert_eq!(
            result.value(),
            0b1111111111111111111111111111111111111111111111111111111111110011
        );
    }

    #[test]
    fn test_left_shift() {
        let bitboard = Bitboard(0b1000);
        let result = bitboard.left_shift(2);
        assert_eq!(result.value(), 0b100000);
    }

    #[test]
    fn test_right_shift() {
        let bitboard = Bitboard(0b1000);
        let result = bitboard.right_shift(2);
        assert_eq!(result.value(), 0b10);
    }
}
