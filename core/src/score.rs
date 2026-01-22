pub type Score = i32;

pub const MATE_SCORE: Score = 100_000;
pub const NEG_MATE_SCORE: Score = -100_000;
pub const MATE_THRESHOLD: Score = 90_000;

pub const fn mate_in(n: u32) -> Score {
    MATE_SCORE - (n as Score)
}

pub const fn mated_in(n: u32) -> Score {
    NEG_MATE_SCORE + (n as Score)
}

pub const fn score_to_mate_moves(score: Score) -> Option<i32> {
    if score > MATE_THRESHOLD {
        let plies = MATE_SCORE - score;
        Some((plies + 1) / 2)
    } else if score < -MATE_THRESHOLD {
        let plies = MATE_SCORE + score;
        Some(-((plies + 1) / 2))
    } else {
        None
    }
}
