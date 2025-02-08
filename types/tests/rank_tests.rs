#[cfg(test)]
mod tests {
    use aether_types::{Color, Rank};
    use std::str::FromStr;
    #[test]
    fn test_rank_from_str() {
        assert_eq!(Rank::from_str("1").unwrap(), Rank::One);
        assert_eq!(Rank::from_str("2").unwrap(), Rank::Two);
        assert_eq!(Rank::from_str("3").unwrap(), Rank::Three);
        assert_eq!(Rank::from_str("4").unwrap(), Rank::Four);
        assert_eq!(Rank::from_str("5").unwrap(), Rank::Five);
        assert_eq!(Rank::from_str("6").unwrap(), Rank::Six);
        assert_eq!(Rank::from_str("7").unwrap(), Rank::Seven);
        assert_eq!(Rank::from_str("8").unwrap(), Rank::Eight);
        assert!(Rank::from_str("9").is_err());
    }

    #[test]
    fn test_rank_new() {
        assert_eq!(Rank::new(0), Rank::One);
        assert_eq!(Rank::new(1), Rank::Two);
        assert_eq!(Rank::new(2), Rank::Three);
        assert_eq!(Rank::new(3), Rank::Four);
        assert_eq!(Rank::new(4), Rank::Five);
        assert_eq!(Rank::new(5), Rank::Six);
        assert_eq!(Rank::new(6), Rank::Seven);
        assert_eq!(Rank::new(7), Rank::Eight);
    }

    #[test]
    fn test_rank_flip() {
        assert_eq!(Rank::One.flip(), Rank::Eight);
        assert_eq!(Rank::Two.flip(), Rank::Seven);
        assert_eq!(Rank::Three.flip(), Rank::Six);
        assert_eq!(Rank::Four.flip(), Rank::Five);
        assert_eq!(Rank::Five.flip(), Rank::Four);
        assert_eq!(Rank::Six.flip(), Rank::Three);
        assert_eq!(Rank::Seven.flip(), Rank::Two);
        assert_eq!(Rank::Eight.flip(), Rank::One);
    }

    #[test]
    fn test_rank_bitboard() {
        assert_eq!(Rank::One.bitboard().value(), 0x00000000000000ff);
        assert_eq!(Rank::Two.bitboard().value(), 0x000000000000ff00);
        assert_eq!(Rank::Three.bitboard().value(), 0x0000000000ff0000);
        assert_eq!(Rank::Four.bitboard().value(), 0x00000000ff000000);
        assert_eq!(Rank::Five.bitboard().value(), 0x000000ff00000000);
        assert_eq!(Rank::Six.bitboard().value(), 0x0000ff0000000000);
        assert_eq!(Rank::Seven.bitboard().value(), 0x00ff000000000000);
        assert_eq!(Rank::Eight.bitboard().value(), 0xff00000000000000);
    }

    #[test]
    fn test_rank_relative_to() {
        assert_eq!(Rank::One.relative_to(Color::Black), Rank::Eight);
        assert_eq!(Rank::Two.relative_to(Color::Black), Rank::Seven);
        assert_eq!(Rank::Three.relative_to(Color::Black), Rank::Six);
        assert_eq!(Rank::Four.relative_to(Color::Black), Rank::Five);
        assert_eq!(Rank::Five.relative_to(Color::Black), Rank::Four);
        assert_eq!(Rank::Six.relative_to(Color::Black), Rank::Three);
        assert_eq!(Rank::Seven.relative_to(Color::Black), Rank::Two);
        assert_eq!(Rank::Eight.relative_to(Color::Black), Rank::One);

        assert_eq!(Rank::One.relative_to(Color::White), Rank::One);
        assert_eq!(Rank::Two.relative_to(Color::White), Rank::Two);
        assert_eq!(Rank::Three.relative_to(Color::White), Rank::Three);
        assert_eq!(Rank::Four.relative_to(Color::White), Rank::Four);
        assert_eq!(Rank::Five.relative_to(Color::White), Rank::Five);
        assert_eq!(Rank::Six.relative_to(Color::White), Rank::Six);
        assert_eq!(Rank::Seven.relative_to(Color::White), Rank::Seven);
        assert_eq!(Rank::Eight.relative_to(Color::White), Rank::Eight);
    }
}