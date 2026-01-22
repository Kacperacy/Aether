pub mod alpha_beta;
mod move_ordering;
pub mod see;
pub mod tt;

pub use tt::{NodeType, TTEntry, TranspositionTable};

use aether_core::{Move, Score};
use std::time::Duration;

const MAX_PLY: usize = 128;
const MAX_PV_LENGTH: usize = MAX_PLY;

#[derive(Debug, Clone)]
pub struct SearchLimits {
    pub depth: Option<u8>,
    pub nodes: Option<u64>,
    pub time: Option<Duration>,
    pub hard_time: Option<Duration>,
    pub infinite: bool,
}

impl Default for SearchLimits {
    fn default() -> Self {
        Self {
            depth: Some(3), // Default depth of 3
            nodes: None,
            time: None,
            hard_time: None,
            infinite: false,
        }
    }
}

impl SearchLimits {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn depth(depth: u8) -> Self {
        Self {
            depth: Some(depth),
            ..Self::default()
        }
    }

    pub fn nodes(nodes: u64) -> Self {
        Self {
            depth: None,
            nodes: Some(nodes),
            time: None,
            hard_time: None,
            infinite: false,
        }
    }

    pub fn time(time: Duration) -> Self {
        Self {
            depth: None,
            nodes: None,
            time: Some(time),
            hard_time: None,
            infinite: false,
        }
    }

    pub fn time_with_hard_limit(time: Duration, hard_time: Duration) -> Self {
        Self {
            depth: None,
            nodes: None,
            time: Some(time),
            hard_time: Some(hard_time),
            infinite: false,
        }
    }

    pub fn infinite() -> Self {
        Self {
            depth: Some(128),
            nodes: None,
            time: None,
            hard_time: None,
            infinite: true,
        }
    }
}

#[derive(Debug, Clone, Default)]
pub struct SearchInfo {
    pub depth: u8,
    pub selective_depth: u8,
    pub nodes: u64,
    pub time_elapsed: Duration,
    pub pv: Vec<Move>,
    pub score: Score,
    pub nps: u64,
    pub hash_full: u16,
}

impl SearchInfo {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn calculate_nps(&mut self) {
        if self.time_elapsed.as_millis() > 0 {
            self.nps = (self.nodes as u128 * 1000 / self.time_elapsed.as_millis()) as u64;
        }
    }
}

#[derive(Debug, Clone)]
pub struct SearchResult {
    pub best_move: Option<Move>,
    pub score: Score,
    pub pv: Vec<Move>,
    pub info: SearchInfo,
}

impl SearchResult {
    pub fn new(best_move: Option<Move>, score: Score) -> Self {
        Self {
            best_move,
            score,
            pv: Vec::new(),
            info: SearchInfo::new(),
        }
    }

    pub fn with_info(
        best_move: Option<Move>,
        score: Score,
        pv: Vec<Move>,
        info: SearchInfo,
    ) -> Self {
        Self {
            best_move,
            score,
            pv,
            info,
        }
    }
}
