pub mod alpha_beta;
pub mod mcts;
mod move_ordering;
pub mod mtdf;
pub mod negascout;
pub mod see;
pub mod searcher;
pub mod tt;

pub use alpha_beta::{FullAlphaBetaSearcher, PureAlphaBetaSearcher};
pub use mcts::{ClassicMctsSearcher, MctsSearcher};
pub use mtdf::MtdfSearcher;
pub use negascout::PureNegaScoutSearcher;
pub use searcher::{Searcher, SearcherType};
pub use tt::{NodeType, TTEntry, TranspositionTable};

use crate::eval::SimpleEvaluator;
use aether_core::{Move, Score};
use std::time::Duration;

pub(crate) const MAX_PLY: usize = 128;
pub(crate) const MAX_PV_LENGTH: usize = MAX_PLY;

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
            depth: Some(3),
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

/// Extended benchmark metrics for algorithm comparison
#[derive(Debug, Clone, Default)]
pub struct BenchmarkMetrics {
    /// Time to find the first move
    pub time_to_first_move: Duration,
    /// Time spent at each depth
    pub time_per_depth: Vec<Duration>,
    /// Nodes searched at each depth
    pub nodes_per_depth: Vec<u64>,
    /// Average branching factor
    pub branching_factor: f64,
    /// Number of times the best move changed during search
    pub score_stability: u32,
    /// Best move found at each depth
    pub best_move_per_depth: Vec<Option<Move>>,
}

impl BenchmarkMetrics {
    pub fn new() -> Self {
        Self::default()
    }

    /// Calculate the average branching factor from nodes per depth
    pub fn calculate_branching_factor(&mut self) {
        if self.nodes_per_depth.len() >= 2 {
            let mut sum = 0.0;
            let mut count = 0;
            for i in 1..self.nodes_per_depth.len() {
                if self.nodes_per_depth[i - 1] > 0 {
                    sum += self.nodes_per_depth[i] as f64 / self.nodes_per_depth[i - 1] as f64;
                    count += 1;
                }
            }
            if count > 0 {
                self.branching_factor = sum / count as f64;
            }
        }
    }

    /// Calculate score stability (how many times the best move changed)
    pub fn calculate_stability(&mut self) {
        let mut changes = 0u32;
        let mut last_move: Option<Move> = None;
        for mv in &self.best_move_per_depth {
            if let Some(current) = mv {
                if let Some(prev) = last_move {
                    if prev != *current {
                        changes += 1;
                    }
                }
                last_move = Some(*current);
            }
        }
        self.score_stability = changes;
    }
}

/// Factory function to create a searcher based on the type
pub fn create_searcher(searcher_type: SearcherType, tt_size_mb: usize) -> Box<dyn Searcher> {
    let evaluator = SimpleEvaluator::new();

    match searcher_type {
        SearcherType::PureAlphaBeta => Box::new(PureAlphaBetaSearcher::new(evaluator)),
        SearcherType::FullAlphaBeta => Box::new(FullAlphaBetaSearcher::new(evaluator, tt_size_mb)),
        SearcherType::Mtdf => Box::new(MtdfSearcher::new(evaluator, tt_size_mb)),
        SearcherType::NegaScout => Box::new(PureNegaScoutSearcher::new(evaluator)),
        SearcherType::Mcts => Box::new(MctsSearcher::new(evaluator)),
        SearcherType::ClassicMcts => Box::new(ClassicMctsSearcher::new()),
    }
}
