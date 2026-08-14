use aether_core::{
    BitBoard, Color, Move, Piece, Score, Square, bishop_attacks, king_attacks, knight_attacks,
    pawn_attacks, rook_attacks,
};

#[inline]
pub fn see_ge(
    mv: &Move,
    side: Color,
    threshold: Score,
    occupied: BitBoard,
    pieces: &[[BitBoard; 6]; 2],
) -> bool {
    let from = mv.from_sq();
    let to = mv.to_sq();

    let target_value = if mv.is_en_passant() {
        Piece::PAWN_VALUE
    } else if mv.is_capture() {
        match piece_on(to, &pieces[side.opponent() as usize]) {
            Some(piece) => piece.value(),
            None => return threshold <= 0,
        }
    } else {
        return threshold <= 0;
    };

    let (promotion_gain, attacker_value) = match mv.promotion_piece() {
        Some(promo_piece) => {
            let promo_value = promo_piece.value();
            (promo_value - Piece::PAWN_VALUE, promo_value)
        }
        None => {
            let moving_piece =
                piece_on(from, &pieces[side as usize]).expect("SEE: no piece at from-square");
            (0, moving_piece.value())
        }
    };

    let mut swap = target_value + promotion_gain - threshold;
    if swap < 0 {
        return false;
    }

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

        let pawn_attackers = stm_attackers & pieces[stm as usize][Piece::Pawn as usize];
        if !pawn_attackers.is_empty() {
            swap = Piece::PAWN_VALUE - swap;
            if swap < result as Score {
                break;
            }
            occ ^= pawn_attackers.lsb().bitboard();
            attackers |= bishop_attacks(to, occ) & get_diagonal_sliders(pieces);
            attackers &= occ;
            stm = stm.opponent();
            continue;
        }

        let knight_attackers = stm_attackers & pieces[stm as usize][Piece::Knight as usize];
        if !knight_attackers.is_empty() {
            swap = Piece::KNIGHT_VALUE - swap;
            if swap < result as Score {
                break;
            }
            occ ^= knight_attackers.lsb().bitboard();
            attackers &= occ;
            stm = stm.opponent();
            continue;
        }

        let bishop_attackers = stm_attackers & pieces[stm as usize][Piece::Bishop as usize];
        if !bishop_attackers.is_empty() {
            swap = Piece::BISHOP_VALUE - swap;
            if swap < result as Score {
                break;
            }
            occ ^= bishop_attackers.lsb().bitboard();
            attackers |= bishop_attacks(to, occ) & get_diagonal_sliders(pieces);
            attackers &= occ;
            stm = stm.opponent();
            continue;
        }

        let rook_attackers = stm_attackers & pieces[stm as usize][Piece::Rook as usize];
        if !rook_attackers.is_empty() {
            swap = Piece::ROOK_VALUE - swap;
            if swap < result as Score {
                break;
            }
            occ ^= rook_attackers.lsb().bitboard();
            attackers |= rook_attacks(to, occ) & get_straight_sliders(pieces);
            attackers &= occ;
            stm = stm.opponent();
            continue;
        }

        let queen_attackers = stm_attackers & pieces[stm as usize][Piece::Queen as usize];
        if !queen_attackers.is_empty() {
            swap = Piece::QUEEN_VALUE - swap;
            occ ^= queen_attackers.lsb().bitboard();
            attackers |= bishop_attacks(to, occ) & get_diagonal_sliders(pieces);
            attackers |= rook_attacks(to, occ) & get_straight_sliders(pieces);
            attackers &= occ;
            stm = stm.opponent();
            continue;
        }

        let opponent_attackers = attackers & get_color_pieces(stm.opponent(), pieces);
        return opponent_attackers.is_empty() != (result != 0);
    }

    result != 0
}

#[inline]
pub fn see_value(mv: &Move, side: Color, occupied: BitBoard, pieces: &[[BitBoard; 6]; 2]) -> Score {
    let to = mv.to_sq();
    let from = mv.from_sq();

    let target_value = if mv.is_en_passant() {
        Piece::PAWN_VALUE
    } else if mv.is_capture() {
        match piece_on(to, &pieces[side.opponent() as usize]) {
            Some(piece) => piece.value(),
            None => return 0,
        }
    } else {
        return 0;
    };

    let moving_piece =
        piece_on(from, &pieces[side as usize]).expect("SEE: no piece at from-square");

    let (promotion_gain, attacker_value) = match mv.promotion_piece() {
        Some(promo_piece) => {
            let promo_value = promo_piece.value();
            (promo_value - Piece::PAWN_VALUE, promo_value)
        }
        None => (0, moving_piece.value()),
    };

    let mut gain: [Score; 32] = [0; 32];
    let mut depth = 0;

    gain[0] = target_value + promotion_gain;

    let mut occ = occupied ^ from.bitboard() ^ to.bitboard();

    let mut attackers = all_attackers_to_square(to, occ, pieces);

    if matches!(moving_piece, Piece::Pawn | Piece::Bishop | Piece::Queen) {
        attackers |= bishop_attacks(to, occ) & get_diagonal_sliders(pieces);
    }
    if matches!(moving_piece, Piece::Rook | Piece::Queen) {
        attackers |= rook_attacks(to, occ) & get_straight_sliders(pieces);
    }
    attackers &= occ;

    let mut current_piece_value = attacker_value;
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
        current_piece_value = Piece::ALL[attacker_piece as usize].value();
        stm = stm.opponent();
    }

    while depth > 0 {
        gain[depth - 1] = -(-gain[depth - 1]).max(gain[depth]);
        depth -= 1;
    }

    gain[0]
}

#[inline]
pub(crate) fn piece_on(square: Square, color_pieces: &[BitBoard; 6]) -> Option<Piece> {
    Piece::ALL
        .iter()
        .find(|&&p| color_pieces[p as usize].contains(square))
        .copied()
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

    fn empty_pieces() -> [[BitBoard; 6]; 2] {
        [[BitBoard::EMPTY; 6]; 2]
    }

    fn make_capture(from: Square, to: Square) -> Move {
        Move::new(from, to, Move::CAPTURE)
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

        let mv = make_capture(Square::E4, Square::D5);
        let occupied = get_occupied(&pieces);

        assert!(see_ge(&mv, Color::White, 0, occupied, &pieces));
        assert!(see_ge(
            &mv,
            Color::White,
            Piece::PAWN_VALUE,
            occupied,
            &pieces
        ));
        assert!(!see_ge(
            &mv,
            Color::White,
            Piece::PAWN_VALUE + 1,
            occupied,
            &pieces
        ));
        assert_eq!(
            see_value(&mv, Color::White, occupied, &pieces),
            Piece::PAWN_VALUE
        );
    }

    #[test]
    fn test_see_pawn_takes_defended_pawn() {
        let mut pieces = empty_pieces();
        pieces[Color::White as usize][Piece::Pawn as usize] = Square::E4.bitboard();
        pieces[Color::Black as usize][Piece::Pawn as usize] =
            Square::D5.bitboard() | Square::C6.bitboard();

        let mv = make_capture(Square::E4, Square::D5);
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

        let mv = make_capture(Square::D1, Square::D5);
        let occupied = get_occupied(&pieces);

        assert!(!see_ge(&mv, Color::White, 0, occupied, &pieces));
        assert!(see_ge(
            &mv,
            Color::White,
            Piece::PAWN_VALUE - Piece::QUEEN_VALUE,
            occupied,
            &pieces
        ));
        assert_eq!(
            see_value(&mv, Color::White, occupied, &pieces),
            Piece::PAWN_VALUE - Piece::QUEEN_VALUE
        );
    }

    #[test]
    fn test_see_knight_takes_defended_rook() {
        let mut pieces = empty_pieces();
        pieces[Color::White as usize][Piece::Knight as usize] = Square::F3.bitboard();
        pieces[Color::Black as usize][Piece::Rook as usize] = Square::E5.bitboard();
        pieces[Color::Black as usize][Piece::Pawn as usize] = Square::F6.bitboard();

        let mv = make_capture(Square::F3, Square::E5);
        let occupied = get_occupied(&pieces);

        let expected = Piece::ROOK_VALUE - Piece::KNIGHT_VALUE;
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

        let mv = make_capture(Square::D4, Square::D8);
        let occupied = get_occupied(&pieces);

        assert!(see_ge(&mv, Color::White, 0, occupied, &pieces));
        assert!(see_ge(
            &mv,
            Color::White,
            Piece::QUEEN_VALUE,
            occupied,
            &pieces
        ));
        assert_eq!(
            see_value(&mv, Color::White, occupied, &pieces),
            Piece::QUEEN_VALUE
        );
    }

    #[test]
    fn test_see_xray_with_defender() {
        let mut pieces = empty_pieces();
        pieces[Color::White as usize][Piece::Rook as usize] =
            Square::D4.bitboard() | Square::A4.bitboard();
        pieces[Color::Black as usize][Piece::Bishop as usize] = Square::E4.bitboard();

        let mv = make_capture(Square::D4, Square::E4);
        let occupied = get_occupied(&pieces);

        assert!(see_ge(&mv, Color::White, 0, occupied, &pieces));
        assert_eq!(
            see_value(&mv, Color::White, occupied, &pieces),
            Piece::BISHOP_VALUE
        );
    }

    #[test]
    fn test_see_losing_exchange() {
        let mut pieces = empty_pieces();
        pieces[Color::White as usize][Piece::Queen as usize] = Square::D1.bitboard();
        pieces[Color::Black as usize][Piece::Pawn as usize] =
            Square::D5.bitboard() | Square::E6.bitboard();

        let mv = make_capture(Square::D1, Square::D5);
        let occupied = get_occupied(&pieces);

        assert!(!see_ge(&mv, Color::White, 0, occupied, &pieces));
        assert_eq!(
            see_value(&mv, Color::White, occupied, &pieces),
            Piece::PAWN_VALUE - Piece::QUEEN_VALUE
        );
    }

    #[test]
    fn test_see_knight_takes_bishop_defended_by_queen() {
        let mut pieces = empty_pieces();
        pieces[Color::White as usize][Piece::Knight as usize] = Square::C3.bitboard();
        pieces[Color::Black as usize][Piece::Bishop as usize] = Square::E4.bitboard();
        pieces[Color::Black as usize][Piece::Queen as usize] = Square::E7.bitboard();

        let mv = make_capture(Square::C3, Square::E4);
        let occupied = get_occupied(&pieces);

        // NxB (+330), QxN (-320) = +10
        assert!(see_ge(&mv, Color::White, 0, occupied, &pieces));
        assert_eq!(
            see_value(&mv, Color::White, occupied, &pieces),
            Piece::BISHOP_VALUE - Piece::KNIGHT_VALUE
        );
    }

    #[test]
    fn test_see_equal_exchange() {
        let mut pieces = empty_pieces();
        pieces[Color::White as usize][Piece::Knight as usize] = Square::C3.bitboard();
        pieces[Color::Black as usize][Piece::Knight as usize] = Square::E4.bitboard();

        let mv = make_capture(Square::C3, Square::E4);
        let occupied = get_occupied(&pieces);

        assert!(see_ge(&mv, Color::White, 0, occupied, &pieces));
        assert!(see_ge(
            &mv,
            Color::White,
            Piece::KNIGHT_VALUE,
            occupied,
            &pieces
        ));
        assert_eq!(
            see_value(&mv, Color::White, occupied, &pieces),
            Piece::KNIGHT_VALUE
        );
    }

    #[test]
    fn test_see_defended_equal_exchange() {
        let mut pieces = empty_pieces();
        pieces[Color::White as usize][Piece::Knight as usize] = Square::C3.bitboard();
        pieces[Color::Black as usize][Piece::Knight as usize] =
            Square::E4.bitboard() | Square::G5.bitboard();

        let mv = make_capture(Square::C3, Square::E4);
        let occupied = get_occupied(&pieces);

        assert!(see_ge(&mv, Color::White, 0, occupied, &pieces));
        assert!(!see_ge(&mv, Color::White, 1, occupied, &pieces));
        assert_eq!(see_value(&mv, Color::White, occupied, &pieces), 0);
    }
}
