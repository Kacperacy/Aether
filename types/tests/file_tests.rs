#[cfg(test)]
mod tests {
    use aether_types::File;
    use std::str::FromStr;

    #[test]
    fn test_file_from_str() {
        assert_eq!(File::from_str("a").unwrap(), File::A);
        assert_eq!(File::from_str("b").unwrap(), File::B);
        assert_eq!(File::from_str("c").unwrap(), File::C);
        assert_eq!(File::from_str("d").unwrap(), File::D);
        assert_eq!(File::from_str("e").unwrap(), File::E);
        assert_eq!(File::from_str("f").unwrap(), File::F);
        assert_eq!(File::from_str("g").unwrap(), File::G);
        assert_eq!(File::from_str("h").unwrap(), File::H);
        assert!(File::from_str("i").is_err());
    }

    #[test]
    fn test_file_from_index() {
        assert_eq!(File::from_index(0), File::A);
        assert_eq!(File::from_index(1), File::B);
        assert_eq!(File::from_index(2), File::C);
        assert_eq!(File::from_index(3), File::D);
        assert_eq!(File::from_index(4), File::E);
        assert_eq!(File::from_index(5), File::F);
        assert_eq!(File::from_index(6), File::G);
        assert_eq!(File::from_index(7), File::H);
    }

    #[test]
    fn test_file_flip() {
        assert_eq!(File::A.flip(), File::H);
        assert_eq!(File::B.flip(), File::G);
        assert_eq!(File::C.flip(), File::F);
        assert_eq!(File::D.flip(), File::E);
        assert_eq!(File::E.flip(), File::D);
        assert_eq!(File::F.flip(), File::C);
        assert_eq!(File::G.flip(), File::B);
        assert_eq!(File::H.flip(), File::A);
    }

    #[test]
    fn test_file_bitboard() {
        assert_eq!(File::A.bitboard().value(), 0x0101010101010101);
        assert_eq!(File::B.bitboard().value(), 0x0202020202020202);
        assert_eq!(File::C.bitboard().value(), 0x0404040404040404);
        assert_eq!(File::D.bitboard().value(), 0x0808080808080808);
        assert_eq!(File::E.bitboard().value(), 0x1010101010101010);
        assert_eq!(File::F.bitboard().value(), 0x2020202020202020);
        assert_eq!(File::G.bitboard().value(), 0x4040404040404040);
        assert_eq!(File::H.bitboard().value(), 0x8080808080808080);
    }

    #[test]
    fn test_file_adjacent() {
        assert_eq!(File::A.adjacent().value(), 0x202020202020202);
        assert_eq!(File::B.adjacent().value(), 0x505050505050505);
        assert_eq!(File::C.adjacent().value(), 0xa0a0a0a0a0a0a0a);
        assert_eq!(File::D.adjacent().value(), 0x1414141414141414);
        assert_eq!(File::E.adjacent().value(), 0x2828282828282828);
        assert_eq!(File::F.adjacent().value(), 0x5050505050505050);
        assert_eq!(File::G.adjacent().value(), 0xa0a0a0a0a0a0a0a0);
        assert_eq!(File::H.adjacent().value(), 0x4040404040404040);
    }
}
