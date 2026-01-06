pub type Score = i32;

pub const MATE_SCORE: Score = 100_000;
pub const NEG_MATE_SCORE: Score = -100_000;

pub const fn mate_in(n: u32) -> Score {
    MATE_SCORE - (n as Score)
}

pub const fn mated_in(n: u32) -> Score {
    NEG_MATE_SCORE + (n as Score)
}
