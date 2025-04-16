// movegen/tests/non_sliding_tests.rs
use aether_types::{BitBoard, Color, Square};
use movegen::pieces::{get_king_moves, get_knight_moves, get_pawn_attacks, get_pawn_moves};

#[test]
fn test_knight_moves() {
    // Knight in the middle
    let d4 = Square::D4;
    let moves = get_knight_moves(d4);
    let expected = BitBoard::from_square(Square::B3)
        | BitBoard::from_square(Square::B5)
        | BitBoard::from_square(Square::C2)
        | BitBoard::from_square(Square::C6)
        | BitBoard::from_square(Square::E2)
        | BitBoard::from_square(Square::E6)
        | BitBoard::from_square(Square::F3)
        | BitBoard::from_square(Square::F5);
    assert_eq!(moves, expected);

    // Knight in the corner
    let a1 = Square::A1;
    let moves = get_knight_moves(a1);
    let expected = BitBoard::from_square(Square::B3) | BitBoard::from_square(Square::C2);
    assert_eq!(moves, expected);
}

#[test]
fn test_king_moves() {
    // King in the middle
    let d4 = Square::D4;
    let moves = get_king_moves(d4);
    let expected = BitBoard::from_square(Square::C3)
        | BitBoard::from_square(Square::C4)
        | BitBoard::from_square(Square::C5)
        | BitBoard::from_square(Square::D3)
        | BitBoard::from_square(Square::D5)
        | BitBoard::from_square(Square::E3)
        | BitBoard::from_square(Square::E4)
        | BitBoard::from_square(Square::E5);
    assert_eq!(moves, expected);

    // King in the corner
    let h8 = Square::H8;
    let moves = get_king_moves(h8);
    let expected = BitBoard::from_square(Square::G7)
        | BitBoard::from_square(Square::G8)
        | BitBoard::from_square(Square::H7);
    assert_eq!(moves, expected);
}

#[test]
fn test_pawn_attacks() {
    // White pawn attacks
    let d4 = Square::D4;
    let white_attacks = get_pawn_attacks(d4, Color::White);
    let expected = BitBoard::from_square(Square::C5) | BitBoard::from_square(Square::E5);
    assert_eq!(white_attacks, expected);

    // Black pawn attacks
    let d4 = Square::D4;
    let black_attacks = get_pawn_attacks(d4, Color::Black);
    let expected = BitBoard::from_square(Square::C3) | BitBoard::from_square(Square::E3);
    assert_eq!(black_attacks, expected);

    // Edge pawn attacks
    let a2 = Square::A2;
    let white_attacks = get_pawn_attacks(a2, Color::White);
    let expected = BitBoard::from_square(Square::B3);
    assert_eq!(white_attacks, expected);
}

#[test]
fn test_pawn_moves() {
    // White pawn moves
    let d4 = Square::D4;
    let occupied = BitBoard::from_square(Square::D5);
    let white_moves = get_pawn_moves(d4, Color::White, occupied);
    assert_eq!(white_moves, BitBoard::EMPTY);

    // Black pawn moves
    let d4 = Square::D4;
    let occupied = BitBoard::from_square(Square::D3);
    let black_moves = get_pawn_moves(d4, Color::Black, occupied);
    assert_eq!(black_moves, BitBoard::EMPTY);

    // Double push
    let d2 = Square::D2;
    let white_moves = get_pawn_moves(d2, Color::White, BitBoard::EMPTY);
    let expected = BitBoard::from_square(Square::D3) | BitBoard::from_square(Square::D4);
    assert_eq!(white_moves, expected);

    // Blocked double push
    let d2 = Square::D2;
    let occupied = BitBoard::from_square(Square::D4);
    let white_moves = get_pawn_moves(d2, Color::White, occupied);
    let expected = BitBoard::from_square(Square::D3);
    assert_eq!(white_moves, expected);
}
