use crate::magic_constants::{BISHOP_MAGICS, BISHOP_MOVES, MagicEntry, ROOK_MAGICS, ROOK_MOVES};
use aether_types::{BitBoard, Square};

fn magic_index(entry: &MagicEntry, blockers: BitBoard) -> usize {
    let blockers = blockers & entry.mask;
    let hash = blockers.0.wrapping_mul(entry.magic);
    let index = (hash >> (64 - entry.index_bits)) as usize;
    index
}

pub fn get_rook_moves(square: Square, blockers: BitBoard) -> BitBoard {
    let magic = &ROOK_MAGICS[square as usize];
    let moves = &ROOK_MOVES[square as usize];
    moves[magic_index(magic, blockers)]
}

pub fn get_bishop_moves(square: Square, blockers: BitBoard) -> BitBoard {
    let magic = &BISHOP_MAGICS[square as usize];
    let moves = &BISHOP_MOVES[square as usize];
    moves[magic_index(magic, blockers)]
}

pub fn get_queen_moves(square: Square, blockers: BitBoard) -> BitBoard {
    let rook_moves = get_rook_moves(square, blockers);
    let bishop_moves = get_bishop_moves(square, blockers);
    rook_moves | bishop_moves
}

#[cfg(test)]
mod tests {
    use super::*;
    use aether_types::{BitBoard, Square};

    #[test]
    fn test_rook_moves() {
        let empty_board = BitBoard::EMPTY;

        let a1 = Square::A1;
        let a1_moves = get_rook_moves(a1, empty_board);
        let expected_a1 = BitBoard(72340172838076926);
        assert_eq!(
            a1_moves, expected_a1,
            "Rook on A1 should attack rank 1 and file A"
        );

        // Center square D4
        let d4 = Square::D4;
        let d4_moves = get_rook_moves(d4, empty_board);
        let expected_d4 = BitBoard(578721386714368008);
        assert_eq!(
            d4_moves, expected_d4,
            "Rook on D4 should attack rank 4 and file D"
        );

        let blockers = BitBoard::from_square(Square::D3)
            | BitBoard::from_square(Square::D5)
            | BitBoard::from_square(Square::D6)
            | BitBoard::from_square(Square::C4)
            | BitBoard::from_square(Square::E4);

        let d4_blocked_moves = get_rook_moves(d4, blockers);

        let expected_blocked = BitBoard(34695806976);

        assert_eq!(
            d4_blocked_moves, expected_blocked,
            "Rook on D4 with blockers should have correct moves"
        );
    }

    #[test]
    fn test_bishop_moves() {
        let empty_board = BitBoard::EMPTY;

        let a1 = Square::A1;
        let a1_moves = get_bishop_moves(a1, empty_board);
        let expected_a1 = BitBoard(9241421688590303744);
        assert_eq!(a1_moves, expected_a1, "Bishop on A1 should attack diagonal");

        let d4 = Square::D4;
        let d4_moves = get_bishop_moves(d4, empty_board);
        let expected_d4 = BitBoard(9241705379636978241);
        assert_eq!(
            d4_moves, expected_d4,
            "Bishop on D4 should attack diagonals"
        );

        let blockers = BitBoard::from_square(Square::C3)
            | BitBoard::from_square(Square::E3)
            | BitBoard::from_square(Square::C5)
            | BitBoard::from_square(Square::E5);

        let d4_blocked_moves = get_bishop_moves(d4, blockers);

        let expected_blocked = BitBoard(85900656640);

        assert_eq!(
            d4_blocked_moves, expected_blocked,
            "Bishop on D4 with blockers should have correct moves"
        );
    }

    #[test]
    fn test_queen_moves() {
        let empty_board = BitBoard::EMPTY;
        let d4 = Square::D4;

        let queen_moves = get_queen_moves(d4, empty_board);
        let rook_moves = get_rook_moves(d4, empty_board);
        let bishop_moves = get_bishop_moves(d4, empty_board);

        assert_eq!(
            queen_moves,
            rook_moves | bishop_moves,
            "Queen moves should equal rook moves + bishop moves"
        );
    }

    #[test]
    fn test_edge_cases() {
        let all_occupied = BitBoard::FULL ^ BitBoard::from_square(Square::D4);
        let d4 = Square::D4;

        let rook_moves = get_rook_moves(d4, all_occupied);
        let bishop_moves = get_bishop_moves(d4, all_occupied);

        let expected_rook = BitBoard::from_square(Square::D3)
            | BitBoard::from_square(Square::D5)
            | BitBoard::from_square(Square::C4)
            | BitBoard::from_square(Square::E4);

        let expected_bishop = BitBoard::from_square(Square::C3)
            | BitBoard::from_square(Square::C5)
            | BitBoard::from_square(Square::E3)
            | BitBoard::from_square(Square::E5);

        assert_eq!(
            rook_moves, expected_rook,
            "Rook on crowded board should only see adjacent pieces"
        );
        assert_eq!(
            bishop_moves, expected_bishop,
            "Bishop on crowded board should only see adjacent pieces"
        );
    }

    #[test]
    fn test_magic_index() {
        let sq = Square::E4;
        let rook_entry = &ROOK_MAGICS[sq as usize];
        let bishop_entry = &BISHOP_MAGICS[sq as usize];

        let blockers = BitBoard(0x1000080800000);

        let rook_idx = magic_index(rook_entry, blockers);
        let bishop_idx = magic_index(bishop_entry, blockers);

        assert!(
            rook_idx < (1 << rook_entry.index_bits),
            "Rook magic index should be within bounds"
        );
        assert!(
            bishop_idx < (1 << bishop_entry.index_bits),
            "Bishop magic index should be within bounds"
        );

        let rook_moves = ROOK_MOVES[sq as usize][rook_idx];
        let bishop_moves = BISHOP_MOVES[sq as usize][bishop_idx];

        assert_ne!(rook_moves.0, 0, "Rook moves should not be empty");
        assert_ne!(bishop_moves.0, 0, "Bishop moves should not be empty");

        assert_eq!(
            rook_moves,
            get_rook_moves(sq, blockers),
            "Rook moves from table should match get_rook_moves result"
        );
        assert_eq!(
            bishop_moves,
            get_bishop_moves(sq, blockers),
            "Bishop moves from table should match get_bishop_moves result"
        );
    }
}
