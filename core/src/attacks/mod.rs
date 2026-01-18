#[cfg(feature = "codegen")]
pub mod codegen;
mod magic;
mod magic_constants;
mod pieces;

use crate::{BitBoard, Color, File, Piece, Rank, Square};
pub use magic::*;
pub use magic_constants::*;
pub use pieces::*;

#[must_use]
#[inline(always)]
pub fn attackers_to_square(
    square: Square,
    color: Color,
    occupied: BitBoard,
    pieces: &[BitBoard; 6],
) -> BitBoard {
    let pawn_attackers = pawn_attacks_from(square, color) & pieces[Piece::Pawn as usize];
    let knight_attackers = knight_attacks(square) & pieces[Piece::Knight as usize];
    let king_attackers = king_attacks(square) & pieces[Piece::King as usize];

    let diagonal_attackers = bishop_attacks(square, occupied)
        & (pieces[Piece::Bishop as usize] | pieces[Piece::Queen as usize]);

    let straight_attackers = rook_attacks(square, occupied)
        & (pieces[Piece::Rook as usize] | pieces[Piece::Queen as usize]);

    pawn_attackers | knight_attackers | king_attackers | diagonal_attackers | straight_attackers
}

#[inline]
pub fn is_square_attacked(
    square: Square,
    color: Color,
    occupied: BitBoard,
    pieces: &[BitBoard; 6],
) -> bool {
    if !(pawn_attacks_from(square, color) & pieces[Piece::Pawn as usize]).is_empty() {
        return true;
    }

    if !(knight_attacks(square) & pieces[Piece::Knight as usize]).is_empty() {
        return true;
    }

    if !(bishop_attacks(square, occupied)
        & (pieces[Piece::Bishop as usize] | pieces[Piece::Queen as usize]))
        .is_empty()
    {
        return true;
    }

    if !(rook_attacks(square, occupied)
        & (pieces[Piece::Rook as usize] | pieces[Piece::Queen as usize]))
        .is_empty()
    {
        return true;
    }

    !(king_attacks(square) & pieces[Piece::King as usize]).is_empty()
}

#[inline]
pub fn compute_slider_blockers(
    king_sq: Square,
    our_pieces: BitBoard,
    enemy_pieces: &[BitBoard; 6],
    occupied: BitBoard,
) -> (BitBoard, BitBoard) {
    let mut blockers = BitBoard::EMPTY;
    let mut pinners = BitBoard::EMPTY;

    let enemy_bishops = enemy_pieces[Piece::Bishop as usize];
    let enemy_rooks = enemy_pieces[Piece::Rook as usize];
    let enemy_queens = enemy_pieces[Piece::Queen as usize];

    let diagonal_snipers =
        (enemy_bishops | enemy_queens) & bishop_attacks(king_sq, BitBoard::EMPTY);

    let straight_snipers = (enemy_rooks | enemy_queens) & rook_attacks(king_sq, BitBoard::EMPTY);

    let all_snipers = diagonal_snipers | straight_snipers;

    for sniper_sq in all_snipers.iter() {
        let between = line_between(king_sq, sniper_sq) & occupied;

        if between.count() == 1 {
            blockers |= between;

            if !(between & our_pieces).is_empty() {
                pinners |= sniper_sq.bitboard();
            }
        }
    }

    (blockers, pinners)
}

#[inline]
fn line_direction(sq1: Square, sq2: Square) -> Option<(i8, i8)> {
    if sq1 == sq2 {
        return None;
    }

    let f1 = sq1.file().to_index() as i8;
    let r1 = sq1.rank().to_index() as i8;
    let f2 = sq2.file().to_index() as i8;
    let r2 = sq2.rank().to_index() as i8;

    let df = (f2 - f1).signum();
    let dr = (r2 - r1).signum();

    let file_diff = (f2 - f1).abs();
    let rank_diff = (r2 - r1).abs();

    if file_diff != rank_diff && f1 != f2 && r1 != r2 {
        return None;
    }

    Some((df, dr))
}

#[must_use]
#[inline]
pub fn line_between(sq1: Square, sq2: Square) -> BitBoard {
    let Some((df, dr)) = line_direction(sq1, sq2) else {
        return BitBoard::EMPTY;
    };

    let f2 = sq2.file().to_index() as i8;
    let r2 = sq2.rank().to_index() as i8;

    let mut result = BitBoard::EMPTY;
    let mut f = sq1.file().to_index() as i8 + df;
    let mut r = sq1.rank().to_index() as i8 + dr;

    while f != f2 || r != r2 {
        let sq = Square::new(File::from_index(f), Rank::from_index(r));
        result |= sq.bitboard();
        f += df;
        r += dr;
    }

    result
}

#[must_use]
#[inline]
pub fn line_through(sq1: Square, sq2: Square) -> BitBoard {
    let Some((df, dr)) = line_direction(sq1, sq2) else {
        return BitBoard::EMPTY;
    };

    let mut f = sq1.file().to_index() as i8;
    let mut r = sq1.rank().to_index() as i8;

    while f - df >= 0 && f - df <= 7 && r - dr >= 0 && r - dr <= 7 {
        f -= df;
        r -= dr;
    }

    let mut result = BitBoard::EMPTY;

    while f >= 0 && f <= 7 && r >= 0 && r <= 7 {
        let sq = Square::new(File::from_index(f), Rank::from_index(r));
        result |= sq.bitboard();
        f += df;
        r += dr;
    }

    result
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_rook_attacks_empty_board() {
        let attacks = rook_attacks(Square::E4, BitBoard::EMPTY);
        assert_eq!(attacks.count(), 14); // 7 horizontal + 7 vertical
    }

    #[test]
    fn test_bishop_attacks_empty_board() {
        let attacks = bishop_attacks(Square::E4, BitBoard::EMPTY);
        assert_eq!(attacks.count(), 13); // diagonal moves from e4
    }

    #[test]
    fn test_knight_attacks() {
        let attacks = knight_attacks(Square::E4);
        assert_eq!(attacks.count(), 8); // knight from e4 has 8 moves
    }

    #[test]
    fn test_attackers_to_square() {
        let mut pieces = [BitBoard::EMPTY; 6];
        // Place white rook on e1
        pieces[Piece::Rook as usize] = Square::E1.bitboard();

        let attackers = attackers_to_square(Square::E4, Color::White, BitBoard::EMPTY, &pieces);

        assert!(attackers.has(Square::E1));
        assert_eq!(attackers.count(), 1);
    }

    #[test]
    fn test_pawn_attacks_from_white() {
        let attacks = pawn_attacks_from(Square::E4, Color::White);
        // White pawns attack e4 from d3 and f3
        assert!(attacks.has(Square::D3));
        assert!(attacks.has(Square::F3));
        assert_eq!(attacks.count(), 2);
    }

    #[test]
    fn test_blocked_rook() {
        let mut occupied = BitBoard::EMPTY;
        occupied |= Square::E6.bitboard(); // blocker

        let attacks = rook_attacks(Square::E4, occupied);

        // Should attack e5, e6 (blocker) but not e7, e8
        assert!(attacks.has(Square::E5));
        assert!(attacks.has(Square::E6));
        assert!(!attacks.has(Square::E7));
        assert!(!attacks.has(Square::E8));
    }
}
