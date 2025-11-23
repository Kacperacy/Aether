use aether_core::bitboard::BitBoard;

#[cfg(test)]
mod tests {
    use super::*;
    use aether_core::Square;

    #[test]
    fn test_bitboard_new() {
        let bb = BitBoard::new();
        assert_eq!(bb.value(), 0);
    }

    #[test]
    fn test_flip_rank() {
        let bb = BitBoard(0xff000000ff00);
        let flipped = bb.flip_rank();
        assert_eq!(flipped.value(), 0xff000000ff0000);
    }

    #[test]
    fn test_flip_file() {
        let bb = BitBoard(0x2222222222222222);
        let flipped = bb.flip_file();

        assert_eq!(flipped.value(), 0x4444444444444444);
    }

    #[test]
    fn test_len() {
        let bb = BitBoard(0xaaaaaaaaaaaaaaaa);
        assert_eq!(bb.len(), 32);
    }

    #[test]
    fn test_is_empty() {
        let bb = BitBoard::new();
        assert!(bb.is_empty());
    }

    #[test]
    fn test_is_subset() {
        let bb1 = BitBoard(0xcccccccccccccccc);
        let bb2 = BitBoard(0xeeeeeeeeeeeeeeee);
        assert!(bb1.is_subset(bb2));
    }

    #[test]
    fn test_is_superset() {
        let bb1 = BitBoard(0xeeeeeeeeeeeeeeee);
        let bb2 = BitBoard(0xcccccccccccccccc);
        assert!(bb1.is_superset(bb2));
    }

    #[test]
    fn test_is_set_index() {
        let bb = BitBoard(0x1);
        assert!(bb.is_set_index(0));
        assert!(!bb.is_set_index(1));
    }

    #[test]
    fn test_contains() {
        let bb1 = BitBoard(0xaaaaaaaaaaaaaaaa);
        let bb2 = BitBoard(0xaaaaaaaa00000000);
        assert!(bb1.contains(bb2));
    }

    #[test]
    fn test_has() {
        let bb = BitBoard(0xaaaaaaaaaaaaaaaa);
        let square = Square::B1;
        assert!(bb.has(square));
    }

    #[test]
    fn test_reverse() {
        let bb = BitBoard(0xaaaaaaaaaaaaaaaa);
        let reversed = bb.reverse();
        assert_eq!(reversed.value(), 0x5555555555555555);
    }

    #[test]
    fn test_from_square() {
        let square = Square::A1;
        let bb = BitBoard::from_square(square);
        assert_eq!(bb.value(), 1);
    }

    #[test]
    fn test_to_square_some() {
        let bb = BitBoard(0x1);
        let square = bb.to_square().unwrap();
        assert_eq!(square, Square::A1);
    }

    #[test]
    fn test_to_square_none() {
        let bb = BitBoard(0x0);
        let square = bb.to_square();
        assert_eq!(square, None);
    }

    #[test]
    fn test_next_square_some() {
        let bb = BitBoard(0x1);
        let square = bb.next_square().unwrap();
        assert_eq!(square, Square::A1);
    }

    #[test]
    fn test_next_square_none() {
        let bb = BitBoard(0x0);
        let square = bb.next_square();
        assert_eq!(square, None);
    }

    #[test]
    fn test_next() {
        let mut bb = BitBoard(0xaaaaaaaaaaaaaaaa);
        let square = bb.next().unwrap();
        assert_eq!(square, Square::B1);
        assert_eq!(bb.value(), 0xaaaaaaaaaaaaaaa8);
    }

    #[test]
    fn test_iter() {
        let bb = BitBoard(0xffffffffffffffff);
        let mut iter = bb.into_iter();
        assert_eq!(iter.next().unwrap(), Square::A1);
        assert_eq!(iter.next().unwrap(), Square::B1);
        assert_eq!(iter.next().unwrap(), Square::C1);
        assert_eq!(iter.next().unwrap(), Square::D1);
        assert_eq!(iter.next().unwrap(), Square::E1);
        assert_eq!(iter.next().unwrap(), Square::F1);
        assert_eq!(iter.next().unwrap(), Square::G1);
        assert_eq!(iter.next().unwrap(), Square::H1);
        assert_eq!(iter.next().unwrap(), Square::A2);
        assert_eq!(iter.next().unwrap(), Square::B2);
        assert_eq!(iter.next().unwrap(), Square::C2);
        assert_eq!(iter.next().unwrap(), Square::D2);
        assert_eq!(iter.next().unwrap(), Square::E2);
        assert_eq!(iter.next().unwrap(), Square::F2);
        assert_eq!(iter.next().unwrap(), Square::G2);
        assert_eq!(iter.next().unwrap(), Square::H2);
        assert_eq!(iter.next().unwrap(), Square::A3);
        assert_eq!(iter.next().unwrap(), Square::B3);
        assert_eq!(iter.next().unwrap(), Square::C3);
        assert_eq!(iter.next().unwrap(), Square::D3);
        assert_eq!(iter.next().unwrap(), Square::E3);
        assert_eq!(iter.next().unwrap(), Square::F3);
        assert_eq!(iter.next().unwrap(), Square::G3);
        assert_eq!(iter.next().unwrap(), Square::H3);
        assert_eq!(iter.next().unwrap(), Square::A4);
        assert_eq!(iter.next().unwrap(), Square::B4);
        assert_eq!(iter.next().unwrap(), Square::C4);
        assert_eq!(iter.next().unwrap(), Square::D4);
        assert_eq!(iter.next().unwrap(), Square::E4);
        assert_eq!(iter.next().unwrap(), Square::F4);
        assert_eq!(iter.next().unwrap(), Square::G4);
        assert_eq!(iter.next().unwrap(), Square::H4);
        assert_eq!(iter.next().unwrap(), Square::A5);
        assert_eq!(iter.next().unwrap(), Square::B5);
        assert_eq!(iter.next().unwrap(), Square::C5);
        assert_eq!(iter.next().unwrap(), Square::D5);
        assert_eq!(iter.next().unwrap(), Square::E5);
        assert_eq!(iter.next().unwrap(), Square::F5);
        assert_eq!(iter.next().unwrap(), Square::G5);
        assert_eq!(iter.next().unwrap(), Square::H5);
        assert_eq!(iter.next().unwrap(), Square::A6);
        assert_eq!(iter.next().unwrap(), Square::B6);
        assert_eq!(iter.next().unwrap(), Square::C6);
        assert_eq!(iter.next().unwrap(), Square::D6);
        assert_eq!(iter.next().unwrap(), Square::E6);
        assert_eq!(iter.next().unwrap(), Square::F6);
        assert_eq!(iter.next().unwrap(), Square::G6);
        assert_eq!(iter.next().unwrap(), Square::H6);
        assert_eq!(iter.next().unwrap(), Square::A7);
        assert_eq!(iter.next().unwrap(), Square::B7);
        assert_eq!(iter.next().unwrap(), Square::C7);
        assert_eq!(iter.next().unwrap(), Square::D7);
        assert_eq!(iter.next().unwrap(), Square::E7);
        assert_eq!(iter.next().unwrap(), Square::F7);
        assert_eq!(iter.next().unwrap(), Square::G7);
        assert_eq!(iter.next().unwrap(), Square::H7);
        assert_eq!(iter.next().unwrap(), Square::A8);
        assert_eq!(iter.next().unwrap(), Square::B8);
        assert_eq!(iter.next().unwrap(), Square::C8);
        assert_eq!(iter.next().unwrap(), Square::D8);
        assert_eq!(iter.next().unwrap(), Square::E8);
        assert_eq!(iter.next().unwrap(), Square::F8);
        assert_eq!(iter.next().unwrap(), Square::G8);
        assert_eq!(iter.next().unwrap(), Square::H8);
        assert_eq!(iter.next(), None);
    }
}
