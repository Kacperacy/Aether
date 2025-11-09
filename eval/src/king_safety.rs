//! King Safety Evaluation
//!
//! Evaluates the safety of both kings based on:
//! - Pawn shield (pawns in front of the king)
//! - Open files near the king
//! - Enemy attacking pieces near the king
//! - King exposure in the center vs edge

use aether_types::{BoardQuery, Color, File, Piece, Rank, Square};

use crate::Score;

/// Bonus for each pawn shielding the king
const PAWN_SHIELD_BONUS: Score = 15;

/// Penalty for missing pawn in front of king
const MISSING_PAWN_PENALTY: Score = 20;

/// Penalty for open file next to king
const OPEN_FILE_PENALTY: Score = 25;

/// Penalty for semi-open file next to king (our pawns gone, enemy pawns remain)
const SEMI_OPEN_FILE_PENALTY: Score = 15;

/// Penalty per enemy piece attacking king zone
const ATTACKER_PENALTY: Score = 20;

/// Bonus for castled king (implied by king position + castling rights lost)
const CASTLED_BONUS: Score = 30;

/// Penalty for king in center during middle game
const KING_CENTER_PENALTY: Score = 40;

/// Evaluate king safety for a given color
///
/// Returns a score where positive = safer king
pub fn evaluate_king_safety<T: BoardQuery>(board: &T, color: Color) -> Score {
    // Find king
    let king_square = match board.get_king_square(color) {
        Some(sq) => sq,
        None => return 0, // No king = already lost, doesn't matter
    };

    let mut safety_score = 0;

    // 1. Pawn shield evaluation
    safety_score += evaluate_pawn_shield(board, king_square, color);

    // 2. Open/semi-open files near king
    safety_score -= evaluate_open_files(board, king_square, color);

    // 3. Enemy attackers near king
    safety_score -= evaluate_attackers(board, king_square, color);

    // 4. King position (center vs edge, castling)
    safety_score += evaluate_king_position(board, king_square, color);

    safety_score
}

/// Evaluate pawn shield in front of king
fn evaluate_pawn_shield<T: BoardQuery>(board: &T, king_square: Square, color: Color) -> Score {
    let king_file = king_square.file();
    let king_rank = king_square.rank();

    // Direction pawns should be (white: up, black: down)
    let forward_rank = match color {
        Color::White => king_rank.offset(1),
        Color::Black => king_rank.offset(-1),
    };

    let mut shield_score = 0;

    // Check 3 files: left, center, right of king
    for file_offset in -1..=1 {
        if let Some(check_file) = offset_file(king_file, file_offset) {
            // Check immediate rank in front of king
            if let Some(fwd_rank) = forward_rank {
                let shield_square = Square::new(check_file, fwd_rank);

                if let Some((piece, piece_color)) = board.piece_at(shield_square) {
                    if piece == Piece::Pawn && piece_color == color {
                        shield_score += PAWN_SHIELD_BONUS;
                    }
                } else {
                    // Missing pawn in front
                    shield_score -= MISSING_PAWN_PENALTY;
                }
            }

            // Also check one rank further (double pawn shield)
            if let Some(fwd_rank) = forward_rank {
                if let Some(double_fwd) = fwd_rank.offset(match color {
                    Color::White => 1,
                    Color::Black => -1,
                }) {
                    let double_square = Square::new(check_file, double_fwd);
                    if let Some((piece, piece_color)) = board.piece_at(double_square) {
                        if piece == Piece::Pawn && piece_color == color {
                            shield_score += PAWN_SHIELD_BONUS / 2; // Half bonus for further pawn
                        }
                    }
                }
            }
        }
    }

    shield_score
}

/// Evaluate open and semi-open files near king
fn evaluate_open_files<T: BoardQuery>(board: &T, king_square: Square, color: Color) -> Score {
    let king_file = king_square.file();
    let mut penalty = 0;

    // Check king's file and adjacent files
    for file_offset in -1..=1 {
        if let Some(check_file) = offset_file(king_file, file_offset) {
            let (our_pawns, enemy_pawns) = count_pawns_on_file(board, check_file, color);

            if our_pawns == 0 && enemy_pawns == 0 {
                // Open file - dangerous!
                penalty += OPEN_FILE_PENALTY;
            } else if our_pawns == 0 && enemy_pawns > 0 {
                // Semi-open file (we have no pawns, enemy does) - also dangerous
                penalty += SEMI_OPEN_FILE_PENALTY;
            }
        }
    }

    penalty
}

/// Count pawns on a file for both colors
fn count_pawns_on_file<T: BoardQuery>(
    board: &T,
    file: File,
    our_color: Color,
) -> (usize, usize) {
    let mut our_pawns = 0;
    let mut enemy_pawns = 0;

    for rank_idx in 0..8 {
        let rank = Rank::new(rank_idx);
        let square = Square::new(file, rank);

        if let Some((piece, color)) = board.piece_at(square) {
            if piece == Piece::Pawn {
                if color == our_color {
                    our_pawns += 1;
                } else {
                    enemy_pawns += 1;
                }
            }
        }
    }

    (our_pawns, enemy_pawns)
}

/// Evaluate enemy pieces attacking near king (king zone)
fn evaluate_attackers<T: BoardQuery>(board: &T, king_square: Square, color: Color) -> Score {
    let enemy_color = color.opponent();
    let mut attacker_count = 0;

    // King zone: 3x3 area around king
    let king_file = king_square.file();
    let king_rank = king_square.rank();

    for file_offset in -1..=1 {
        for rank_offset in -1..=1 {
            if let Some(check_file) = offset_file(king_file, file_offset) {
                if let Some(check_rank) = king_rank.offset(rank_offset) {
                    let zone_square = Square::new(check_file, check_rank);

                    // Check if this square is attacked by enemy
                    if board.is_square_attacked(zone_square, enemy_color) {
                        // Count major pieces (queen, rook) more heavily
                        if let Some((piece, piece_color)) = board.piece_at(zone_square) {
                            if piece_color == enemy_color {
                                match piece {
                                    Piece::Queen => attacker_count += 3,
                                    Piece::Rook => attacker_count += 2,
                                    Piece::Knight | Piece::Bishop => attacker_count += 1,
                                    _ => {}
                                }
                            }
                        }
                    }
                }
            }
        }
    }

    attacker_count * ATTACKER_PENALTY
}

/// Evaluate king position (castled, center, edge)
fn evaluate_king_position<T: BoardQuery>(board: &T, king_square: Square, color: Color) -> Score {
    let mut position_score = 0;

    let king_file = king_square.file();
    let king_rank = king_square.rank();

    // Check if king is castled (on g-file or c-file on back rank)
    let back_rank = match color {
        Color::White => Rank::One,
        Color::Black => Rank::Eight,
    };

    if king_rank == back_rank {
        // King on back rank
        if king_file == File::G || king_file == File::C {
            // Likely castled position
            position_score += CASTLED_BONUS;
        }
    }

    // Penalty for king in center files (d, e) during middle game
    // (We approximate middle game by checking if there are still many pieces)
    let piece_count = count_total_pieces(board);

    if piece_count > 20 { // Middle game (most pieces still on board)
        if king_file == File::D || king_file == File::E {
            position_score -= KING_CENTER_PENALTY;
        }
    }

    position_score
}

/// Helper: offset file by delta (-1, 0, 1)
fn offset_file(file: File, delta: i8) -> Option<File> {
    let file_idx = file as i8;
    let new_idx = file_idx + delta;

    if new_idx >= 0 && new_idx < 8 {
        Some(File::from_index(new_idx))
    } else {
        None
    }
}

/// Count total pieces on board (for game phase detection)
fn count_total_pieces<T: BoardQuery>(board: &T) -> usize {
    let mut count = 0;

    for square_idx in 0..64 {
        let square = Square::from_index(square_idx);
        if board.piece_at(square).is_some() {
            count += 1;
        }
    }

    count
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_file_offset() {
        assert_eq!(offset_file(File::E, 0), Some(File::E));
        assert_eq!(offset_file(File::E, 1), Some(File::F));
        assert_eq!(offset_file(File::E, -1), Some(File::D));
        assert_eq!(offset_file(File::A, -1), None);
        assert_eq!(offset_file(File::H, 1), None);
    }
}
