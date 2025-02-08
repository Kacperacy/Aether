#[cfg(test)]
mod tests {
    use aether_types::{Color, File, Rank, Square};
    use std::str::FromStr;
    #[test]
    fn test_square_from_str() {
        assert_eq!(Square::from_str("a1").unwrap(), Square::A1);
        assert_eq!(Square::from_str("b2").unwrap(), Square::B2);
        assert_eq!(Square::from_str("c3").unwrap(), Square::C3);
        assert_eq!(Square::from_str("d4").unwrap(), Square::D4);
        assert_eq!(Square::from_str("e5").unwrap(), Square::E5);
        assert_eq!(Square::from_str("f6").unwrap(), Square::F6);
        assert_eq!(Square::from_str("g7").unwrap(), Square::G7);
        assert_eq!(Square::from_str("h8").unwrap(), Square::H8);
        assert!(Square::from_str("i9").is_err());
    }

    #[test]
    fn test_square_from_index() {
        assert_eq!(Square::from_index(0), Square::A1);
        assert_eq!(Square::from_index(9), Square::B2);
        assert_eq!(Square::from_index(18), Square::C3);
        assert_eq!(Square::from_index(27), Square::D4);
        assert_eq!(Square::from_index(36), Square::E5);
        assert_eq!(Square::from_index(45), Square::F6);
        assert_eq!(Square::from_index(54), Square::G7);
        assert_eq!(Square::from_index(63), Square::H8);
    }

    #[test]
    fn test_square_file() {
        assert_eq!(Square::A1.file(), File::A);
        assert_eq!(Square::B2.file(), File::B);
        assert_eq!(Square::C3.file(), File::C);
        assert_eq!(Square::D4.file(), File::D);
        assert_eq!(Square::E5.file(), File::E);
        assert_eq!(Square::F6.file(), File::F);
        assert_eq!(Square::G7.file(), File::G);
        assert_eq!(Square::H8.file(), File::H);
    }

    #[test]
    fn test_square_rank() {
        assert_eq!(Square::A1.rank(), Rank::One);
        assert_eq!(Square::B2.rank(), Rank::Two);
        assert_eq!(Square::C3.rank(), Rank::Three);
        assert_eq!(Square::D4.rank(), Rank::Four);
        assert_eq!(Square::E5.rank(), Rank::Five);
        assert_eq!(Square::F6.rank(), Rank::Six);
        assert_eq!(Square::G7.rank(), Rank::Seven);
        assert_eq!(Square::H8.rank(), Rank::Eight);
    }

    #[test]
    fn test_square_bitboard() {
        assert_eq!(Square::A1.bitboard().value(), 0x0000000000000001);
        assert_eq!(Square::B2.bitboard().value(), 0x200);
        assert_eq!(Square::C3.bitboard().value(), 0x40000);
        assert_eq!(Square::D4.bitboard().value(), 0x8000000);
        assert_eq!(Square::E5.bitboard().value(), 0x1000000000);
        assert_eq!(Square::F6.bitboard().value(), 0x200000000000);
        assert_eq!(Square::G7.bitboard().value(), 0x40000000000000);
        assert_eq!(Square::H8.bitboard().value(), 0x8000000000000000);
    }

    #[test]
    fn test_square_offset() {
        assert_eq!(Square::A1.offset(1, 1), Some(Square::B2));
        assert_eq!(Square::B2.offset(1, 1), Some(Square::C3));
        assert_eq!(Square::C3.offset(1, 1), Some(Square::D4));
        assert_eq!(Square::D4.offset(1, 1), Some(Square::E5));
        assert_eq!(Square::E5.offset(1, 1), Some(Square::F6));
        assert_eq!(Square::F6.offset(1, 1), Some(Square::G7));
        assert_eq!(Square::G7.offset(1, 1), Some(Square::H8));

        assert_eq!(Square::A1.offset(-1, -1), None);
        assert_eq!(Square::H8.offset(1, 1), None);
        assert_eq!(Square::A8.offset(-1, 1), None);
        assert_eq!(Square::H1.offset(1, -1), None);
    }

    #[test]
    fn test_square_flip_file() {
        assert_eq!(Square::A1.flip_file(), Square::H1);
        assert_eq!(Square::B2.flip_file(), Square::G2);
        assert_eq!(Square::C3.flip_file(), Square::F3);
        assert_eq!(Square::D4.flip_file(), Square::E4);
        assert_eq!(Square::E5.flip_file(), Square::D5);
        assert_eq!(Square::F6.flip_file(), Square::C6);
        assert_eq!(Square::G7.flip_file(), Square::B7);
        assert_eq!(Square::H8.flip_file(), Square::A8);
    }

    #[test]
    fn test_square_flip_rank() {
        assert_eq!(Square::A1.flip_rank(), Square::A8);
        assert_eq!(Square::B2.flip_rank(), Square::B7);
        assert_eq!(Square::C3.flip_rank(), Square::C6);
        assert_eq!(Square::D4.flip_rank(), Square::D5);
        assert_eq!(Square::E5.flip_rank(), Square::E4);
        assert_eq!(Square::F6.flip_rank(), Square::F3);
        assert_eq!(Square::G7.flip_rank(), Square::G2);
        assert_eq!(Square::H8.flip_rank(), Square::H1);
    }

    #[test]
    fn test_square_relative_to() {
        assert_eq!(Square::A1.relative_to(Color::Black), Square::A8);
        assert_eq!(Square::B2.relative_to(Color::Black), Square::B7);
        assert_eq!(Square::C3.relative_to(Color::Black), Square::C6);
        assert_eq!(Square::D4.relative_to(Color::Black), Square::D5);
        assert_eq!(Square::E5.relative_to(Color::Black), Square::E4);
        assert_eq!(Square::F6.relative_to(Color::Black), Square::F3);
        assert_eq!(Square::G7.relative_to(Color::Black), Square::G2);
        assert_eq!(Square::H8.relative_to(Color::Black), Square::H1);

        assert_eq!(Square::A1.relative_to(Color::White), Square::A1);
        assert_eq!(Square::B2.relative_to(Color::White), Square::B2);
        assert_eq!(Square::C3.relative_to(Color::White), Square::C3);
        assert_eq!(Square::D4.relative_to(Color::White), Square::D4);
        assert_eq!(Square::E5.relative_to(Color::White), Square::E5);
        assert_eq!(Square::F6.relative_to(Color::White), Square::F6);
        assert_eq!(Square::G7.relative_to(Color::White), Square::G7);
        assert_eq!(Square::H8.relative_to(Color::White), Square::H8);
    }

    #[test]
    fn test_square_up() {
        assert_eq!(Square::A1.up(Color::White), Some(Square::A2));
        assert_eq!(Square::B2.up(Color::White), Some(Square::B3));
        assert_eq!(Square::C3.up(Color::White), Some(Square::C4));
        assert_eq!(Square::D4.up(Color::White), Some(Square::D5));
        assert_eq!(Square::E5.up(Color::White), Some(Square::E6));
        assert_eq!(Square::F6.up(Color::White), Some(Square::F7));
        assert_eq!(Square::G7.up(Color::White), Some(Square::G8));
        assert_eq!(Square::H8.up(Color::White), None);

        assert_eq!(Square::A1.up(Color::Black), None);
        assert_eq!(Square::B2.up(Color::Black), Some(Square::B1));
        assert_eq!(Square::C3.up(Color::Black), Some(Square::C2));
        assert_eq!(Square::D4.up(Color::Black), Some(Square::D3));
        assert_eq!(Square::E5.up(Color::Black), Some(Square::E4));
        assert_eq!(Square::F6.up(Color::Black), Some(Square::F5));
        assert_eq!(Square::G7.up(Color::Black), Some(Square::G6));
        assert_eq!(Square::H8.up(Color::Black), Some(Square::H7));
    }

    #[test]
    fn test_square_down() {
        assert_eq!(Square::A1.down(Color::White), None);
        assert_eq!(Square::B2.down(Color::White), Some(Square::B1));
        assert_eq!(Square::C3.down(Color::White), Some(Square::C2));
        assert_eq!(Square::D4.down(Color::White), Some(Square::D3));
        assert_eq!(Square::E5.down(Color::White), Some(Square::E4));
        assert_eq!(Square::F6.down(Color::White), Some(Square::F5));
        assert_eq!(Square::G7.down(Color::White), Some(Square::G6));
        assert_eq!(Square::H8.down(Color::White), Some(Square::H7));

        assert_eq!(Square::A1.down(Color::Black), Some(Square::A2));
        assert_eq!(Square::B2.down(Color::Black), Some(Square::B3));
        assert_eq!(Square::C3.down(Color::Black), Some(Square::C4));
        assert_eq!(Square::D4.down(Color::Black), Some(Square::D5));
        assert_eq!(Square::E5.down(Color::Black), Some(Square::E6));
        assert_eq!(Square::F6.down(Color::Black), Some(Square::F7));
        assert_eq!(Square::G7.down(Color::Black), Some(Square::G8));
        assert_eq!(Square::H8.down(Color::Black), None);
    }

    #[test]
    fn test_square_left() {
        assert_eq!(Square::A1.left(Color::White), None);
        assert_eq!(Square::B2.left(Color::White), Some(Square::A2));
        assert_eq!(Square::C3.left(Color::White), Some(Square::B3));
        assert_eq!(Square::D4.left(Color::White), Some(Square::C4));
        assert_eq!(Square::E5.left(Color::White), Some(Square::D5));
        assert_eq!(Square::F6.left(Color::White), Some(Square::E6));
        assert_eq!(Square::G7.left(Color::White), Some(Square::F7));
        assert_eq!(Square::H8.left(Color::White), Some(Square::G8));

        assert_eq!(Square::A1.left(Color::Black), Some(Square::B1));
        assert_eq!(Square::B2.left(Color::Black), Some(Square::C2));
        assert_eq!(Square::C3.left(Color::Black), Some(Square::D3));
        assert_eq!(Square::D4.left(Color::Black), Some(Square::E4));
        assert_eq!(Square::E5.left(Color::Black), Some(Square::F5));
        assert_eq!(Square::F6.left(Color::Black), Some(Square::G6));
        assert_eq!(Square::G7.left(Color::Black), Some(Square::H7));
        assert_eq!(Square::H8.left(Color::Black), None);
    }

    #[test]
    fn test_square_right() {
        assert_eq!(Square::A1.right(Color::White), Some(Square::B1));
        assert_eq!(Square::B2.right(Color::White), Some(Square::C2));
        assert_eq!(Square::C3.right(Color::White), Some(Square::D3));
        assert_eq!(Square::D4.right(Color::White), Some(Square::E4));
        assert_eq!(Square::E5.right(Color::White), Some(Square::F5));
        assert_eq!(Square::F6.right(Color::White), Some(Square::G6));
        assert_eq!(Square::G7.right(Color::White), Some(Square::H7));
        assert_eq!(Square::H8.right(Color::White), None);

        assert_eq!(Square::A1.right(Color::Black), None);
        assert_eq!(Square::B2.right(Color::Black), Some(Square::A2));
        assert_eq!(Square::C3.right(Color::Black), Some(Square::B3));
        assert_eq!(Square::D4.right(Color::Black), Some(Square::C4));
        assert_eq!(Square::E5.right(Color::Black), Some(Square::D5));
        assert_eq!(Square::F6.right(Color::Black), Some(Square::E6));
        assert_eq!(Square::G7.right(Color::Black), Some(Square::F7));
        assert_eq!(Square::H8.right(Color::Black), Some(Square::G8));
    }
}
