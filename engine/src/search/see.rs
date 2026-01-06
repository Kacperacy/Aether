//! Static Exchange Evaluation (SEE)
//!
//! Evaluates the outcome of a sequence of captures on a single square,
//! assuming both sides always recapture with their least valuable piece.

use aether_core::{
    BISHOP_VALUE, BitBoard, Color, KNIGHT_VALUE, Move, PAWN_VALUE, PIECE_VALUES, Piece,
    QUEEN_VALUE, ROOK_VALUE, Score, Square, bishop_attacks, king_attacks, knight_attacks,
    pawn_attacks, rook_attacks,
};

/// Returns true if the capture's SEE value is >= threshold.
#[inline]
pub fn see_ge(
    mv: &Move,
    side: Color,
    threshold: Score,
    occupied: BitBoard,
    pieces: &[[BitBoard; 6]; 2],
) -> bool {
    let from = mv.from;
    let to = mv.to;

    let target_value = match mv.capture {
        Some(piece) => PIECE_VALUES[piece as usize],
        None => return threshold <= 0,
    };

    let mut swap = target_value - threshold;
    if swap < 0 {
        return false;
    }

    let attacker_value = PIECE_VALUES[mv.piece as usize];
    swap = attacker_value - swap;
    if swap <= 0 {
        return true;
    }

    let mut occ = occupied ^ from.bitboard() ^ to.bitboard();
    let mut attackers = all_attackers_to_square(to, occ, pieces);
    attackers &= occ;

    let mut stm = side.opponent();
    let mut result = 1i32;

    loop {
        let stm_attackers = attackers & get_color_pieces(stm, pieces);

        if stm_attackers.is_empty() {
            break;
        }

        result ^= 1;

        // Pawns
        let pawn_attackers = stm_attackers & pieces[stm as usize][Piece::Pawn as usize];
        if !pawn_attackers.is_empty() {
            swap = PAWN_VALUE - swap;
            if swap < result as Score {
                break;
            }
            occ ^= pawn_attackers.lsb().bitboard();
            attackers |= bishop_attacks(to, occ) & get_diagonal_sliders(pieces);
            attackers &= occ;
            stm = stm.opponent();
            continue;
        }

        // Knights
        let knight_attackers = stm_attackers & pieces[stm as usize][Piece::Knight as usize];
        if !knight_attackers.is_empty() {
            swap = KNIGHT_VALUE - swap;
            if swap < result as Score {
                break;
            }
            occ ^= knight_attackers.lsb().bitboard();
            attackers &= occ;
            stm = stm.opponent();
            continue;
        }

        // Bishops
        let bishop_attackers = stm_attackers & pieces[stm as usize][Piece::Bishop as usize];
        if !bishop_attackers.is_empty() {
            swap = BISHOP_VALUE - swap;
            if swap < result as Score {
                break;
            }
            occ ^= bishop_attackers.lsb().bitboard();
            attackers |= bishop_attacks(to, occ) & get_diagonal_sliders(pieces);
            attackers &= occ;
            stm = stm.opponent();
            continue;
        }

        // Rooks
        let rook_attackers = stm_attackers & pieces[stm as usize][Piece::Rook as usize];
        if !rook_attackers.is_empty() {
            swap = ROOK_VALUE - swap;
            if swap < result as Score {
                break;
            }
            occ ^= rook_attackers.lsb().bitboard();
            attackers |= rook_attacks(to, occ) & get_straight_sliders(pieces);
            attackers &= occ;
            stm = stm.opponent();
            continue;
        }

        // Queens
        let queen_attackers = stm_attackers & pieces[stm as usize][Piece::Queen as usize];
        if !queen_attackers.is_empty() {
            swap = QUEEN_VALUE - swap;
            occ ^= queen_attackers.lsb().bitboard();
            attackers |= bishop_attacks(to, occ) & get_diagonal_sliders(pieces);
            attackers |= rook_attacks(to, occ) & get_straight_sliders(pieces);
            attackers &= occ;
            stm = stm.opponent();
            continue;
        }

        // Only king left
        let opponent_attackers = attackers & get_color_pieces(stm.opponent(), pieces);
        return opponent_attackers.is_empty() != (result != 0);
    }

    result != 0
}

/// Returns the exact SEE value of a capture.
#[inline]
pub fn see_value(mv: &Move, side: Color, occupied: BitBoard, pieces: &[[BitBoard; 6]; 2]) -> Score {
    let to = mv.to;
    let from = mv.from;

    let target_value = match mv.capture {
        Some(piece) => PIECE_VALUES[piece as usize],
        None => return 0,
    };

    let mut gain: [Score; 32] = [0; 32];
    let mut depth = 0;

    gain[0] = target_value;

    // Remove both attacker and victim from occupied
    let mut occ = occupied ^ from.bitboard() ^ to.bitboard();

    let mut attackers = all_attackers_to_square(to, occ, pieces);

    // Add X-ray attackers revealed by removing the initial piece
    if matches!(mv.piece, Piece::Pawn | Piece::Bishop | Piece::Queen) {
        attackers |= bishop_attacks(to, occ) & get_diagonal_sliders(pieces);
    }
    if matches!(mv.piece, Piece::Rook | Piece::Queen) {
        attackers |= rook_attacks(to, occ) & get_straight_sliders(pieces);
    }
    attackers &= occ;

    let mut current_piece_value = PIECE_VALUES[mv.piece as usize];
    let mut stm = side.opponent();

    while let Some((attacker_sq, attacker_piece)) =
        get_least_valuable_attacker(attackers & get_color_pieces(stm, pieces), pieces, stm)
    {
        depth += 1;

        gain[depth] = current_piece_value - gain[depth - 1];

        if (-gain[depth - 1]).max(gain[depth]) < 0 {
            break;
        }

        occ ^= attacker_sq.bitboard();

        if matches!(attacker_piece, Piece::Pawn | Piece::Bishop | Piece::Queen) {
            attackers |= bishop_attacks(to, occ) & get_diagonal_sliders(pieces);
        }
        if matches!(attacker_piece, Piece::Rook | Piece::Queen) {
            attackers |= rook_attacks(to, occ) & get_straight_sliders(pieces);
        }

        attackers &= occ;
        current_piece_value = PIECE_VALUES[attacker_piece as usize];
        stm = stm.opponent();
    }

    while depth > 0 {
        gain[depth - 1] = -(-gain[depth - 1]).max(gain[depth]);
        depth -= 1;
    }

    gain[0]
}

#[inline]
fn all_attackers_to_square(
    square: Square,
    occupied: BitBoard,
    pieces: &[[BitBoard; 6]; 2],
) -> BitBoard {
    let white = &pieces[Color::White as usize];
    let black = &pieces[Color::Black as usize];

    let white_pawn_attackers = pawn_attacks(square, Color::Black) & white[Piece::Pawn as usize];
    let black_pawn_attackers = pawn_attacks(square, Color::White) & black[Piece::Pawn as usize];

    let knight_attackers =
        knight_attacks(square) & (white[Piece::Knight as usize] | black[Piece::Knight as usize]);

    let diagonal_attackers = bishop_attacks(square, occupied) & get_diagonal_sliders(pieces);
    let straight_attackers = rook_attacks(square, occupied) & get_straight_sliders(pieces);

    let king_attackers =
        king_attacks(square) & (white[Piece::King as usize] | black[Piece::King as usize]);

    white_pawn_attackers
        | black_pawn_attackers
        | knight_attackers
        | diagonal_attackers
        | straight_attackers
        | king_attackers
}

#[inline]
fn get_diagonal_sliders(pieces: &[[BitBoard; 6]; 2]) -> BitBoard {
    pieces[Color::White as usize][Piece::Bishop as usize]
        | pieces[Color::White as usize][Piece::Queen as usize]
        | pieces[Color::Black as usize][Piece::Bishop as usize]
        | pieces[Color::Black as usize][Piece::Queen as usize]
}

#[inline]
fn get_straight_sliders(pieces: &[[BitBoard; 6]; 2]) -> BitBoard {
    pieces[Color::White as usize][Piece::Rook as usize]
        | pieces[Color::White as usize][Piece::Queen as usize]
        | pieces[Color::Black as usize][Piece::Rook as usize]
        | pieces[Color::Black as usize][Piece::Queen as usize]
}

#[inline]
fn get_color_pieces(color: Color, pieces: &[[BitBoard; 6]; 2]) -> BitBoard {
    let p = &pieces[color as usize];
    p[0] | p[1] | p[2] | p[3] | p[4] | p[5]
}

#[inline]
fn get_least_valuable_attacker(
    attackers: BitBoard,
    pieces: &[[BitBoard; 6]; 2],
    color: Color,
) -> Option<(Square, Piece)> {
    const PIECE_ORDER: [Piece; 6] = [
        Piece::Pawn,
        Piece::Knight,
        Piece::Bishop,
        Piece::Rook,
        Piece::Queen,
        Piece::King,
    ];

    let color_pieces = &pieces[color as usize];
    for &piece in &PIECE_ORDER {
        let piece_attackers = attackers & color_pieces[piece as usize];
        if !piece_attackers.is_empty() {
            return Some((piece_attackers.lsb(), piece));
        }
    }

    None
}

#[cfg(test)]
mod tests {
    use super::*;
    use aether_core::MoveFlags;

    fn empty_pieces() -> [[BitBoard; 6]; 2] {
        [[BitBoard::EMPTY; 6]; 2]
    }

    fn make_capture(from: Square, to: Square, piece: Piece, captured: Piece) -> Move {
        Move {
            from,
            to,
            piece,
            capture: Some(captured),
            promotion: None,
            flags: MoveFlags::default(),
        }
    }

    fn get_occupied(pieces: &[[BitBoard; 6]; 2]) -> BitBoard {
        let mut occ = BitBoard::EMPTY;
        for color in 0..2 {
            for piece in 0..6 {
                occ |= pieces[color][piece];
            }
        }
        occ
    }

    #[test]
    fn test_see_simple_pawn_takes_pawn() {
        let mut pieces = empty_pieces();
        pieces[Color::White as usize][Piece::Pawn as usize] = Square::E4.bitboard();
        pieces[Color::Black as usize][Piece::Pawn as usize] = Square::D5.bitboard();

        let mv = make_capture(Square::E4, Square::D5, Piece::Pawn, Piece::Pawn);
        let occupied = get_occupied(&pieces);

        assert!(see_ge(&mv, Color::White, 0, occupied, &pieces));
        assert!(see_ge(&mv, Color::White, PAWN_VALUE, occupied, &pieces));
        assert!(!see_ge(
            &mv,
            Color::White,
            PAWN_VALUE + 1,
            occupied,
            &pieces
        ));
        assert_eq!(see_value(&mv, Color::White, occupied, &pieces), PAWN_VALUE);
    }

    #[test]
    fn test_see_pawn_takes_defended_pawn() {
        let mut pieces = empty_pieces();
        pieces[Color::White as usize][Piece::Pawn as usize] = Square::E4.bitboard();
        pieces[Color::Black as usize][Piece::Pawn as usize] =
            Square::D5.bitboard() | Square::C6.bitboard();

        let mv = make_capture(Square::E4, Square::D5, Piece::Pawn, Piece::Pawn);
        let occupied = get_occupied(&pieces);

        assert!(see_ge(&mv, Color::White, 0, occupied, &pieces));
        assert!(!see_ge(&mv, Color::White, 1, occupied, &pieces));
        assert_eq!(see_value(&mv, Color::White, occupied, &pieces), 0);
    }

    #[test]
    fn test_see_queen_takes_defended_pawn() {
        let mut pieces = empty_pieces();
        pieces[Color::White as usize][Piece::Queen as usize] = Square::D1.bitboard();
        pieces[Color::Black as usize][Piece::Pawn as usize] =
            Square::D5.bitboard() | Square::E6.bitboard();

        let mv = make_capture(Square::D1, Square::D5, Piece::Queen, Piece::Pawn);
        let occupied = get_occupied(&pieces);

        assert!(!see_ge(&mv, Color::White, 0, occupied, &pieces));
        assert!(see_ge(
            &mv,
            Color::White,
            PAWN_VALUE - QUEEN_VALUE,
            occupied,
            &pieces
        ));
        assert_eq!(
            see_value(&mv, Color::White, occupied, &pieces),
            PAWN_VALUE - QUEEN_VALUE
        );
    }

    #[test]
    fn test_see_knight_takes_defended_rook() {
        let mut pieces = empty_pieces();
        pieces[Color::White as usize][Piece::Knight as usize] = Square::F3.bitboard();
        pieces[Color::Black as usize][Piece::Rook as usize] = Square::E5.bitboard();
        pieces[Color::Black as usize][Piece::Pawn as usize] = Square::F6.bitboard();

        let mv = make_capture(Square::F3, Square::E5, Piece::Knight, Piece::Rook);
        let occupied = get_occupied(&pieces);

        let expected = ROOK_VALUE - KNIGHT_VALUE;
        assert!(see_ge(&mv, Color::White, 0, occupied, &pieces));
        assert!(see_ge(&mv, Color::White, expected, occupied, &pieces));
        assert!(!see_ge(&mv, Color::White, expected + 1, occupied, &pieces));
        assert_eq!(see_value(&mv, Color::White, occupied, &pieces), expected);
    }

    #[test]
    fn test_see_xray_rook_attack() {
        let mut pieces = empty_pieces();
        pieces[Color::White as usize][Piece::Rook as usize] =
            Square::D4.bitboard() | Square::D1.bitboard();
        pieces[Color::Black as usize][Piece::Queen as usize] = Square::D8.bitboard();

        let mv = make_capture(Square::D4, Square::D8, Piece::Rook, Piece::Queen);
        let occupied = get_occupied(&pieces);

        assert!(see_ge(&mv, Color::White, 0, occupied, &pieces));
        assert!(see_ge(&mv, Color::White, QUEEN_VALUE, occupied, &pieces));
        assert_eq!(see_value(&mv, Color::White, occupied, &pieces), QUEEN_VALUE);
    }

    #[test]
    fn test_see_xray_with_defender() {
        let mut pieces = empty_pieces();
        pieces[Color::White as usize][Piece::Rook as usize] =
            Square::D4.bitboard() | Square::A4.bitboard();
        pieces[Color::Black as usize][Piece::Bishop as usize] = Square::E4.bitboard();

        let mv = make_capture(Square::D4, Square::E4, Piece::Rook, Piece::Bishop);
        let occupied = get_occupied(&pieces);

        assert!(see_ge(&mv, Color::White, 0, occupied, &pieces));
        assert_eq!(
            see_value(&mv, Color::White, occupied, &pieces),
            BISHOP_VALUE
        );
    }

    #[test]
    fn test_see_losing_exchange() {
        let mut pieces = empty_pieces();
        pieces[Color::White as usize][Piece::Queen as usize] = Square::D1.bitboard();
        pieces[Color::Black as usize][Piece::Pawn as usize] =
            Square::D5.bitboard() | Square::E6.bitboard();

        let mv = make_capture(Square::D1, Square::D5, Piece::Queen, Piece::Pawn);
        let occupied = get_occupied(&pieces);

        assert!(!see_ge(&mv, Color::White, 0, occupied, &pieces));
        assert_eq!(
            see_value(&mv, Color::White, occupied, &pieces),
            PAWN_VALUE - QUEEN_VALUE
        );
    }

    #[test]
    fn test_see_knight_takes_bishop_defended_by_queen() {
        let mut pieces = empty_pieces();
        pieces[Color::White as usize][Piece::Knight as usize] = Square::C3.bitboard();
        pieces[Color::Black as usize][Piece::Bishop as usize] = Square::E4.bitboard();
        pieces[Color::Black as usize][Piece::Queen as usize] = Square::E7.bitboard();

        let mv = make_capture(Square::C3, Square::E4, Piece::Knight, Piece::Bishop);
        let occupied = get_occupied(&pieces);

        // NxB (+330), QxN (-320) = +10
        assert!(see_ge(&mv, Color::White, 0, occupied, &pieces));
        assert_eq!(
            see_value(&mv, Color::White, occupied, &pieces),
            BISHOP_VALUE - KNIGHT_VALUE
        );
    }

    #[test]
    fn test_see_equal_exchange() {
        let mut pieces = empty_pieces();
        pieces[Color::White as usize][Piece::Knight as usize] = Square::C3.bitboard();
        pieces[Color::Black as usize][Piece::Knight as usize] = Square::E4.bitboard();

        let mv = make_capture(Square::C3, Square::E4, Piece::Knight, Piece::Knight);
        let occupied = get_occupied(&pieces);

        assert!(see_ge(&mv, Color::White, 0, occupied, &pieces));
        assert!(see_ge(&mv, Color::White, KNIGHT_VALUE, occupied, &pieces));
        assert_eq!(
            see_value(&mv, Color::White, occupied, &pieces),
            KNIGHT_VALUE
        );
    }

    #[test]
    fn test_see_defended_equal_exchange() {
        let mut pieces = empty_pieces();
        pieces[Color::White as usize][Piece::Knight as usize] = Square::C3.bitboard();
        pieces[Color::Black as usize][Piece::Knight as usize] =
            Square::E4.bitboard() | Square::G5.bitboard();

        let mv = make_capture(Square::C3, Square::E4, Piece::Knight, Piece::Knight);
        let occupied = get_occupied(&pieces);

        assert!(see_ge(&mv, Color::White, 0, occupied, &pieces));
        assert!(!see_ge(&mv, Color::White, 1, occupied, &pieces));
        assert_eq!(see_value(&mv, Color::White, occupied, &pieces), 0);
    }
}
