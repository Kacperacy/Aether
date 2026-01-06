use aether_core::{Color, PIECE_VALUES, Piece, Square};

/// Pawn middlegame PST
#[rustfmt::skip]
const PAWN_PST_MG: [i32; 64] = [
      0,   0,   0,   0,   0,   0,   0,   0,
     50,  50,  50,  50,  50,  50,  50,  50,
     20,  20,  30,  40,  40,  30,  20,  20,
     10,  10,  20,  30,  30,  20,  10,  10,
      5,   5,  15,  25,  25,  15,   5,   5,
      0,   0,  10,  20,  20,  10,   0,   0,
      5,  10,   0,  -5,  -5,   0,  10,   5,
      0,   0,   0,   0,   0,   0,   0,   0,
];

/// Knight middlegame PST
#[rustfmt::skip]
const KNIGHT_PST_MG: [i32; 64] = [
    -50, -40, -30, -30, -30, -30, -40, -50,
    -40, -20,   0,   5,   5,   0, -20, -40,
    -30,   5,  15,  20,  20,  15,   5, -30,
    -30,   0,  20,  25,  25,  20,   0, -30,
    -30,   5,  20,  25,  25,  20,   5, -30,
    -30,   0,  15,  20,  20,  15,   0, -30,
    -40, -20,   0,   5,   5,   0, -20, -40,
    -50, -40, -30, -30, -30, -30, -40, -50,
];

/// Bishop middlegame PST
#[rustfmt::skip]
const BISHOP_PST_MG: [i32; 64] = [
    -20, -10, -10, -10, -10, -10, -10, -20,
    -10,   5,   0,   0,   0,   0,   5, -10,
    -10,  10,  10,  10,  10,  10,  10, -10,
    -10,   0,  15,  15,  15,  15,   0, -10,
    -10,   5,  15,  15,  15,  15,   5, -10,
    -10,   0,  10,  15,  15,  10,   0, -10,
    -10,  10,   0,   5,   5,   0,  10, -10,
    -20, -10, -10, -10, -10, -10, -10, -20,
];

/// Rook middlegame PST
#[rustfmt::skip]
const ROOK_PST_MG: [i32; 64] = [
      0,   0,   0,   5,   5,   0,   0,   0,
     10,  15,  15,  20,  20,  15,  15,  10,
      0,   0,   0,   5,   5,   0,   0,   0,
      0,   0,   0,   5,   5,   0,   0,   0,
      0,   0,   0,   5,   5,   0,   0,   0,
      0,   0,   0,   5,   5,   0,   0,   0,
      0,   0,   0,   5,   5,   0,   0,   0,
     -5,   0,   5,  10,  10,   5,   0,  -5,
];

/// Queen middlegame PST
#[rustfmt::skip]
const QUEEN_PST_MG: [i32; 64] = [
    -20, -10, -10,  -5,  -5, -10, -10, -20,
    -10,   0,   5,   5,   5,   5,   0, -10,
    -10,   5,  10,  10,  10,  10,   5, -10,
     -5,   0,  10,  10,  10,  10,   0,  -5,
     -5,   0,  10,  10,  10,  10,   0,  -5,
    -10,   0,   5,  10,  10,   5,   0, -10,
    -10,   0,   0,   0,   0,   0,   0, -10,
    -20, -10, -10,  -5,  -5, -10, -10, -20,
];

/// King middlegame PST
#[rustfmt::skip]
const KING_PST_MG: [i32; 64] = [
    -50, -40, -30, -20, -20, -30, -40, -50,
    -30, -20, -10,   0,   0, -10, -20, -30,
    -30, -10,  20,  30,  30,  20, -10, -30,
    -30, -10,  30,  40,  40,  30, -10, -30,
    -30, -10,  30,  40,  40,  30, -10, -30,
    -30, -10,  20,  30,  30,  20, -10, -30,
    -30, -30, -10,   0,   0, -10, -30, -30,
     20,  30,  -5, -30, -10, -30,  30,  20,
];

/// Pawn endgame PST
#[rustfmt::skip]
const PAWN_PST_EG: [i32; 64] = [
      0,   0,   0,   0,   0,   0,   0,   0,
    100, 100, 100, 100, 100, 100, 100, 100,
     60,  60,  60,  60,  60,  60,  60,  60,
     40,  40,  40,  40,  40,  40,  40,  40,
     20,  20,  20,  20,  20,  20,  20,  20,
     10,  10,  10,  10,  10,  10,  10,  10,
      5,   5,   5,   5,   5,   5,   5,   5,
      0,   0,   0,   0,   0,   0,   0,   0,
];

/// Knight endgame PST
#[rustfmt::skip]
const KNIGHT_PST_EG: [i32; 64] = [
    -50, -40, -30, -30, -30, -30, -40, -50,
    -40, -20,   0,   0,   0,   0, -20, -40,
    -30,   0,  10,  15,  15,  10,   0, -30,
    -30,   5,  15,  20,  20,  15,   5, -30,
    -30,   0,  15,  20,  20,  15,   0, -30,
    -30,   5,  10,  15,  15,  10,   5, -30,
    -40, -20,   0,   5,   5,   0, -20, -40,
    -50, -40, -30, -30, -30, -30, -40, -50,
];

/// Bishop endgame PST
#[rustfmt::skip]
const BISHOP_PST_EG: [i32; 64] = [
    -20, -10, -10, -10, -10, -10, -10, -20,
    -10,   0,   0,   0,   0,   0,   0, -10,
    -10,   0,  10,  10,  10,  10,   0, -10,
    -10,   5,  10,  15,  15,  10,   5, -10,
    -10,   0,  10,  15,  15,  10,   0, -10,
    -10,  10,  10,  10,  10,  10,  10, -10,
    -10,   5,   0,   0,   0,   0,   5, -10,
    -20, -10, -10, -10, -10, -10, -10, -20,
];

/// Rook endgame PST
#[rustfmt::skip]
const ROOK_PST_EG: [i32; 64] = [
      0,   0,   0,   0,   0,   0,   0,   0,
     15,  15,  15,  15,  15,  15,  15,  15,
      0,   0,   0,   0,   0,   0,   0,   0,
      0,   0,   0,   0,   0,   0,   0,   0,
      0,   0,   0,   0,   0,   0,   0,   0,
      0,   0,   0,   0,   0,   0,   0,   0,
      0,   0,   0,   0,   0,   0,   0,   0,
      0,   0,   0,   0,   0,   0,   0,   0,
];

/// Queen endgame PST
#[rustfmt::skip]
const QUEEN_PST_EG: [i32; 64] = [
    -20, -10, -10,  -5,  -5, -10, -10, -20,
    -10,   0,   5,   5,   5,   5,   0, -10,
    -10,   5,  10,  15,  15,  10,   5, -10,
     -5,   0,  15,  20,  20,  15,   0,  -5,
     -5,   0,  15,  20,  20,  15,   0,  -5,
    -10,   5,  10,  15,  15,  10,   5, -10,
    -10,   0,   0,   5,   5,   0,   0, -10,
    -20, -10, -10,  -5,  -5, -10, -10, -20,
];

/// King endgame PST
#[rustfmt::skip]
const KING_PST_EG: [i32; 64] = [
    -50, -30, -20, -10, -10, -20, -30, -50,
    -30, -10,   0,  10,  10,   0, -10, -30,
    -20,   0,  20,  30,  30,  20,   0, -20,
    -10,  10,  30,  40,  40,  30,  10, -10,
    -10,  10,  30,  40,  40,  30,  10, -10,
    -20,   0,  20,  30,  30,  20,   0, -20,
    -30, -10,   0,  10,  10,   0, -10, -30,
    -50, -30, -20, -10, -10, -20, -30, -50,
];

/// All PST tables for middlegame indexed by piece type
const PST_MG: [[i32; 64]; 6] = [
    PAWN_PST_MG,
    KNIGHT_PST_MG,
    BISHOP_PST_MG,
    ROOK_PST_MG,
    QUEEN_PST_MG,
    KING_PST_MG,
];

/// All PST tables for endgame indexed by piece type
const PST_EG: [[i32; 64]; 6] = [
    PAWN_PST_EG,
    KNIGHT_PST_EG,
    BISHOP_PST_EG,
    ROOK_PST_EG,
    QUEEN_PST_EG,
    KING_PST_EG,
];

/// Get PST index for a square from a specific color's perspective
#[inline(always)]
fn pst_index(square: Square, color: Color) -> usize {
    match color {
        Color::White => square.flip_rank().to_index() as usize,
        Color::Black => square.to_index() as usize,
    }
}

/// Get the PST + material value for a piece on a square
/// Returns (middlegame_value, endgame_value) from white's perspective
#[inline(always)]
pub fn piece_value(piece: Piece, square: Square, color: Color) -> (i32, i32) {
    let idx = pst_index(square, color);
    let piece_idx = piece as usize;
    let material = PIECE_VALUES[piece_idx];
    let mg = material + PST_MG[piece_idx][idx];
    let eg = material + PST_EG[piece_idx][idx];

    if color == Color::White {
        (mg, eg)
    } else {
        (-mg, -eg)
    }
}

/// Compute full PST score for a board position
/// Returns (middlegame_score, endgame_score) from white's perspective
pub fn compute_pst_score(pieces: &[[aether_core::BitBoard; 6]; 2]) -> (i32, i32) {
    use aether_core::{ALL_COLORS, ALL_PIECES};

    let mut mg = 0i32;
    let mut eg = 0i32;

    for &piece in &ALL_PIECES {
        let piece_idx = piece as usize;

        for color in ALL_COLORS {
            for square in pieces[color as usize][piece_idx].iter() {
                let (pmg, peg) = piece_value(piece, square, color);
                mg += pmg;
                eg += peg;
            }
        }
    }

    (mg, eg)
}
