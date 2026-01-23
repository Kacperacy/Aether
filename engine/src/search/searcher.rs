use crate::search::{SearchInfo, SearchLimits, SearchResult};
use aether_core::{Move, Score};
use board::Board;
use std::sync::atomic::AtomicBool;
use std::sync::Arc;

/// Callback type for search info updates
pub type SearchCallback<'a> = &'a mut dyn FnMut(&SearchInfo, Option<Move>, Score);

/// Trait defining the interface for all search algorithms
pub trait Searcher: Send {
    /// Perform a search on the given board position
    fn search(
        &mut self,
        board: &mut Board,
        limits: &SearchLimits,
        on_info: SearchCallback<'_>,
    ) -> SearchResult;

    /// Get the stop flag for external abort control
    fn stop_flag(&self) -> Arc<AtomicBool>;

    /// Signal the search to stop
    fn stop(&mut self);

    /// Reset state for a new game
    fn new_game(&mut self);

    /// Resize the transposition table
    fn resize_tt(&mut self, size_mb: usize);

    /// Get the transposition table fill rate (per mille)
    fn hashfull(&self) -> u16;

    /// Get the current search info
    fn get_info(&self) -> &SearchInfo;

    /// Get the algorithm name for display
    fn algorithm_name(&self) -> &'static str;
}

/// Enum representing the available search algorithm types
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum SearcherType {
    /// Pure Alpha-Beta without optimizations (baseline)
    PureAlphaBeta,
    /// Full Alpha-Beta with all optimizations
    #[default]
    FullAlphaBeta,
    /// MTD(f) - Memory-enhanced Test Driver
    Mtdf,
    /// NegaScout / Principal Variation Search (baseline, comparable to Pure Alpha-Beta)
    NegaScout,
    /// Monte Carlo Tree Search with static evaluation
    Mcts,
    /// Classic MCTS with random playouts (baseline)
    ClassicMcts,
}

impl SearcherType {
    /// Get a human-readable name for the algorithm
    #[must_use]
    pub const fn name(&self) -> &'static str {
        match self {
            Self::PureAlphaBeta => "Pure Alpha-Beta",
            Self::FullAlphaBeta => "Full Alpha-Beta",
            Self::Mtdf => "MTD(f)",
            Self::NegaScout => "NegaScout",
            Self::Mcts => "MCTS",
            Self::ClassicMcts => "Classic MCTS",
        }
    }

    /// Get all available searcher types
    #[must_use]
    pub const fn all() -> &'static [SearcherType] {
        &[
            Self::PureAlphaBeta,
            Self::FullAlphaBeta,
            Self::Mtdf,
            Self::NegaScout,
            Self::Mcts,
            Self::ClassicMcts,
        ]
    }
}

impl std::fmt::Display for SearcherType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.name())
    }
}

impl std::str::FromStr for SearcherType {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "pure" | "purealphabeta" | "pure-alpha-beta" | "pure_alpha_beta" => {
                Ok(Self::PureAlphaBeta)
            }
            "full" | "fullalphabeta" | "full-alpha-beta" | "full_alpha_beta" => {
                Ok(Self::FullAlphaBeta)
            }
            "mtdf" | "mtd-f" | "mtd(f)" => Ok(Self::Mtdf),
            "negascout" | "pvs" | "pv-search" => Ok(Self::NegaScout),
            "mcts" | "monte-carlo" | "montecarlo" => Ok(Self::Mcts),
            "classic-mcts" | "classicmcts" | "classic_mcts" | "classic" => Ok(Self::ClassicMcts),
            _ => Err(format!("Unknown searcher type: {}", s)),
        }
    }
}
