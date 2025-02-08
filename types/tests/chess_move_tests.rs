
#[cfg(test)]
mod tests {
    #[test]
    fn test_chess_move_from_str() {
        use aether_types::chess_move::Move;
        use aether_types::{Piece, Square};
        use std::str::FromStr;

        let m = Move::from_str("e2e4").unwrap();
        assert_eq!(m.from, Square::from_str("e2").unwrap());
        assert_eq!(m.to, Square::from_str("e4").unwrap());
        assert_eq!(m.promotion, None);

        let m = Move::from_str("e7e8q").unwrap();
        assert_eq!(m.from, Square::from_str("e7").unwrap());
        assert_eq!(m.to, Square::from_str("e8").unwrap());
        assert_eq!(m.promotion, Some(Piece::Queen));

        let m = Move::from_str("e7e8r").unwrap();
        assert_eq!(m.from, Square::from_str("e7").unwrap());
        assert_eq!(m.to, Square::from_str("e8").unwrap());
        assert_eq!(m.promotion, Some(Piece::Rook));

        let m = Move::from_str("e7e8b").unwrap();
        assert_eq!(m.from, Square::from_str("e7").unwrap());
        assert_eq!(m.to, Square::from_str("e8").unwrap());
        assert_eq!(m.promotion, Some(Piece::Bishop));

        let m = Move::from_str("e7e8n").unwrap();
        assert_eq!(m.from, Square::from_str("e7").unwrap());
        assert_eq!(m.to, Square::from_str("e8").unwrap());
        assert_eq!(m.promotion, Some(Piece::Knight));

        assert!(Move::from_str("e7e8k").is_err());
    }
}