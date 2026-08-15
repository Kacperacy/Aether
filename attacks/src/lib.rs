//! Attack generation: magic-bitboard sliders, leaper tables, and the
//! aggregate queries (attackers, blockers, line geometry) built on them.

#[cfg(feature = "codegen")]
pub mod codegen;
mod magic;
mod magic_constants;
mod pieces;

use aether_core::{BitBoard, Color, Piece, Square};
pub use magic::*;
pub use pieces::*;

// Generated lookup tables are an implementation detail of this crate.
pub(crate) use magic_constants::*;

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

/// Only the test-only reference implementations use this now; the shipping
/// path is a table lookup.
#[cfg(test)]
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

// `line_between` and `line_through` are called from the two hottest loops in the
// engine — `update_blockers` on every make_move, and the per-move pin test in
// `movegen::legality` — so they are tables, not ray-walks.
//
// Both are `static` with a `const` initialiser: the tables are built by the
// compiler and cost nothing at runtime. A `LazyLock` would have put an atomic
// acquire on every call, which is the same overhead this is meant to remove.
// 4096 entries x 8 bytes = 32KB each.

/// Step direction from `a` towards `b`, plus whether the two squares share a
/// rank, file or diagonal at all. Mirrors `line_direction`.
const fn line_step(a: usize, b: usize) -> (i8, i8, bool) {
    if a == b {
        return (0, 0, false);
    }

    let (f1, r1) = ((a & 7) as i8, (a >> 3) as i8);
    let (f2, r2) = ((b & 7) as i8, (b >> 3) as i8);
    let (fd, rd) = (f2 - f1, r2 - r1);

    let file_diff = if fd < 0 { -fd } else { fd };
    let rank_diff = if rd < 0 { -rd } else { rd };

    if file_diff != rank_diff && f1 != f2 && r1 != r2 {
        return (0, 0, false);
    }

    let df = if fd > 0 {
        1
    } else if fd < 0 {
        -1
    } else {
        0
    };
    let dr = if rd > 0 {
        1
    } else if rd < 0 {
        -1
    } else {
        0
    };

    (df, dr, true)
}

/// Squares strictly between `a` and `b`, exclusive of both.
const fn between_bits(a: usize, b: usize) -> u64 {
    let (df, dr, aligned) = line_step(a, b);
    if !aligned {
        return 0;
    }

    let (f2, r2) = ((b & 7) as i8, (b >> 3) as i8);
    let mut f = (a & 7) as i8 + df;
    let mut r = (a >> 3) as i8 + dr;
    let mut result = 0u64;

    while f != f2 || r != r2 {
        result |= 1u64 << (r * 8 + f);
        f += df;
        r += dr;
    }

    result
}

/// The whole rank, file or diagonal containing both squares, edge to edge.
const fn through_bits(a: usize, b: usize) -> u64 {
    let (df, dr, aligned) = line_step(a, b);
    if !aligned {
        return 0;
    }

    // Walk back to the edge of the board, then forward across it.
    let mut f = (a & 7) as i8;
    let mut r = (a >> 3) as i8;
    while f - df >= 0 && f - df <= 7 && r - dr >= 0 && r - dr <= 7 {
        f -= df;
        r -= dr;
    }

    let mut result = 0u64;
    while f >= 0 && f <= 7 && r >= 0 && r <= 7 {
        result |= 1u64 << (r * 8 + f);
        f += df;
        r += dr;
    }

    result
}

const fn build_table(between: bool) -> [[u64; 64]; 64] {
    let mut table = [[0u64; 64]; 64];
    let mut a = 0;

    while a < 64 {
        let mut b = 0;
        while b < 64 {
            table[a][b] = if between {
                between_bits(a, b)
            } else {
                through_bits(a, b)
            };
            b += 1;
        }
        a += 1;
    }

    table
}

static LINE_BETWEEN: [[u64; 64]; 64] = build_table(true);
static LINE_THROUGH: [[u64; 64]; 64] = build_table(false);

/// Squares strictly between `sq1` and `sq2`, or empty when they do not share a
/// rank, file or diagonal.
#[must_use]
#[inline(always)]
pub fn line_between(sq1: Square, sq2: Square) -> BitBoard {
    BitBoard::new(LINE_BETWEEN[sq1.to_index() as usize][sq2.to_index() as usize])
}

/// The full line through `sq1` and `sq2`, or empty when they are not aligned.
#[must_use]
#[inline(always)]
pub fn line_through(sq1: Square, sq2: Square) -> BitBoard {
    BitBoard::new(LINE_THROUGH[sq1.to_index() as usize][sq2.to_index() as usize])
}

/// The ray-walking originals, kept as the definition the tables are checked
/// against. Not compiled into the engine.
#[cfg(test)]
mod reference {
    use super::*;
    use aether_core::{File, Rank};

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

        while (0..=7).contains(&f) && (0..=7).contains(&r) {
            let sq = Square::new(File::from_index(f), Rank::from_index(r));
            result |= sq.bitboard();
            f += df;
            r += dr;
        }

        result
    }
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

        assert!(attackers.contains(Square::E1));
        assert_eq!(attackers.count(), 1);
    }

    #[test]
    fn test_pawn_attacks_from_white() {
        let attacks = pawn_attacks_from(Square::E4, Color::White);
        // White pawns attack e4 from d3 and f3
        assert!(attacks.contains(Square::D3));
        assert!(attacks.contains(Square::F3));
        assert_eq!(attacks.count(), 2);
    }

    #[test]
    fn test_blocked_rook() {
        let mut occupied = BitBoard::EMPTY;
        occupied |= Square::E6.bitboard(); // blocker

        let attacks = rook_attacks(Square::E4, occupied);

        // Should attack e5, e6 (blocker) but not e7, e8
        assert!(attacks.contains(Square::E5));
        assert!(attacks.contains(Square::E6));
        assert!(!attacks.contains(Square::E7));
        assert!(!attacks.contains(Square::E8));
    }
}

#[cfg(test)]
mod line_table_tests {
    use super::*;

    /// The tables must agree with the ray-walk definition on every one of the
    /// 4096 square pairs — including the unaligned pairs and `sq == sq`, which
    /// both yield an empty board.
    #[test]
    fn test_line_tables_match_the_ray_walk() {
        for a in 0..64i8 {
            for b in 0..64i8 {
                let (sq1, sq2) = (Square::from_index(a), Square::from_index(b));

                assert_eq!(
                    line_between(sq1, sq2),
                    reference::line_between(sq1, sq2),
                    "line_between({sq1}, {sq2})"
                );
                assert_eq!(
                    line_through(sq1, sq2),
                    reference::line_through(sq1, sq2),
                    "line_through({sq1}, {sq2})"
                );
            }
        }
    }

    #[test]
    fn test_unaligned_and_identical_squares_are_empty() {
        assert_eq!(line_between(Square::A1, Square::B3), BitBoard::EMPTY);
        assert_eq!(line_through(Square::A1, Square::B3), BitBoard::EMPTY);
        assert_eq!(line_between(Square::E4, Square::E4), BitBoard::EMPTY);
        assert_eq!(line_through(Square::E4, Square::E4), BitBoard::EMPTY);
    }

    /// Adjacent squares have nothing between them, but do share a full line.
    #[test]
    fn test_adjacent_squares() {
        assert_eq!(line_between(Square::A1, Square::A2), BitBoard::EMPTY);
        assert!(line_through(Square::A1, Square::A2).contains(Square::A8));
    }
}
