//! Benchmark comparison utilities for search algorithms
//!
//! Provides structures and functions for comparing multiple search algorithms
//! on the same positions and exporting results to CSV format.

use aether_core::Move;
use engine::search::{BenchmarkMetrics, SearcherType};
use std::fmt::Write as FmtWrite;
use std::fs::File;
use std::io::{self, Write};
use std::path::Path;
use std::time::Duration;

/// Result of benchmarking a single algorithm on a single position
#[derive(Debug, Clone)]
pub struct AlgorithmBenchResult {
    /// The algorithm type that was tested
    pub algorithm: SearcherType,
    /// Search depth achieved
    pub depth: u8,
    /// Total nodes searched
    pub nodes: u64,
    /// Total time taken
    pub time: Duration,
    /// Nodes per second
    pub nps: u64,
    /// Time to first move
    pub time_to_first_move: Duration,
    /// Best move found
    pub best_move: Option<Move>,
    /// Score in centipawns
    pub score: i32,
    /// Extended benchmark metrics
    pub metrics: BenchmarkMetrics,
}

impl AlgorithmBenchResult {
    pub fn new(algorithm: SearcherType) -> Self {
        Self {
            algorithm,
            depth: 0,
            nodes: 0,
            time: Duration::ZERO,
            nps: 0,
            time_to_first_move: Duration::ZERO,
            best_move: None,
            score: 0,
            metrics: BenchmarkMetrics::new(),
        }
    }
}

/// Game phase for categorizing positions
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GamePhase {
    Opening,
    Middlegame,
    Endgame,
}

impl GamePhase {
    /// Determine game phase from board's game_phase value
    pub fn from_phase_value(phase: i32) -> Self {
        if phase > 200 {
            GamePhase::Opening
        } else if phase > 80 {
            GamePhase::Middlegame
        } else {
            GamePhase::Endgame
        }
    }

    pub fn as_str(&self) -> &'static str {
        match self {
            GamePhase::Opening => "opening",
            GamePhase::Middlegame => "middlegame",
            GamePhase::Endgame => "endgame",
        }
    }
}

impl std::fmt::Display for GamePhase {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

/// Comparison of all algorithms on a single position
#[derive(Debug, Clone)]
pub struct PositionComparison {
    /// Position ID or FEN
    pub position_id: String,
    /// Game phase of the position
    pub phase: GamePhase,
    /// Results for each algorithm
    pub results: Vec<AlgorithmBenchResult>,
}

impl PositionComparison {
    pub fn new(position_id: String, phase: GamePhase) -> Self {
        Self {
            position_id,
            phase,
            results: Vec::new(),
        }
    }

    pub fn add_result(&mut self, result: AlgorithmBenchResult) {
        self.results.push(result);
    }
}

/// Format a comparison table for display
pub fn format_comparison_table(results: &[AlgorithmBenchResult]) -> String {
    let mut output = String::new();

    // Header
    writeln!(
        output,
        "{:<21} {:>5} {:>12} {:>10} {:>12} {:>8} {:>8} {:>4}",
        "Algorithm", "Depth", "Nodes", "Time (ms)", "NPS", "TTFM", "BestMv", "Stab"
    )
    .unwrap();

    writeln!(output, "{}", "-".repeat(90)).unwrap();

    for result in results {
        let best_move_str = result
            .best_move
            .map(|m| format_move(&m))
            .unwrap_or_else(|| "none".to_string());

        let ttfm_ms = result.time_to_first_move.as_secs_f64() * 1000.0;

        writeln!(
            output,
            "{:<21} {:>5} {:>12} {:>10.1} {:>12} {:>8.1} {:>8} {:>4}",
            result.algorithm.name(),
            result.depth,
            result.nodes,
            result.time.as_secs_f64() * 1000.0,
            result.nps,
            ttfm_ms,
            best_move_str,
            result.metrics.score_stability
        )
        .unwrap();
    }

    output
}

/// Format a move in UCI notation
pub fn format_move(mv: &Move) -> String {
    let mut s = format!("{}{}", mv.from, mv.to);
    if let Some(promo) = mv.promotion {
        s.push(match promo {
            aether_core::Piece::Queen => 'q',
            aether_core::Piece::Rook => 'r',
            aether_core::Piece::Bishop => 'b',
            aether_core::Piece::Knight => 'n',
            _ => 'q',
        });
    }
    s
}

/// CSV record for a single benchmark result
#[derive(Debug)]
pub struct CsvRecord {
    pub algorithm: String,
    pub position_id: String,
    pub phase: String,
    pub depth: u8,
    pub nodes: u64,
    pub time_ms: f64,
    pub nps: u64,
    pub ttfm_ms: f64,
    pub best_move: String,
    pub score: i32,
    pub branching_factor: f64,
    pub stability: u32,
}

impl CsvRecord {
    pub fn from_result(
        result: &AlgorithmBenchResult,
        position_id: &str,
        phase: GamePhase,
    ) -> Self {
        Self {
            algorithm: result.algorithm.name().to_string(),
            position_id: position_id.to_string(),
            phase: phase.as_str().to_string(),
            depth: result.depth,
            nodes: result.nodes,
            time_ms: result.time.as_secs_f64() * 1000.0,
            nps: result.nps,
            ttfm_ms: result.time_to_first_move.as_secs_f64() * 1000.0,
            best_move: result
                .best_move
                .map(|m| format_move(&m))
                .unwrap_or_else(|| "none".to_string()),
            score: result.score,
            branching_factor: result.metrics.branching_factor,
            stability: result.metrics.score_stability,
        }
    }

    pub fn to_csv_line(&self) -> String {
        format!(
            "{},{},{},{},{},{:.2},{},{:.2},{},{},{:.2},{}",
            self.algorithm,
            self.position_id,
            self.phase,
            self.depth,
            self.nodes,
            self.time_ms,
            self.nps,
            self.ttfm_ms,
            self.best_move,
            self.score,
            self.branching_factor,
            self.stability
        )
    }
}

/// Export benchmark results to a CSV file
pub fn export_to_csv<P: AsRef<Path>>(
    path: P,
    comparisons: &[PositionComparison],
) -> io::Result<()> {
    let mut file = File::create(path)?;

    // Write header
    writeln!(
        file,
        "algorithm,position_id,phase,depth,nodes,time_ms,nps,ttfm_ms,best_move,score,branching_factor,stability"
    )?;

    // Write records
    for comparison in comparisons {
        for result in &comparison.results {
            let record = CsvRecord::from_result(result, &comparison.position_id, comparison.phase);
            writeln!(file, "{}", record.to_csv_line())?;
        }
    }

    file.flush()?;
    Ok(())
}

/// Format summary statistics for multiple comparisons
pub fn format_summary(comparisons: &[PositionComparison]) -> String {
    let mut output = String::new();

    // Collect stats per algorithm
    let algorithms = SearcherType::all();

    writeln!(output, "\n=== Summary Statistics ===\n").unwrap();
    writeln!(
        output,
        "{:<21} {:>8} {:>12} {:>12} {:>10}",
        "Algorithm", "Avg Dep", "Avg Nodes", "Avg NPS", "Avg TTFM"
    )
    .unwrap();
    writeln!(output, "{}", "-".repeat(65)).unwrap();

    for algo in algorithms {
        let results: Vec<&AlgorithmBenchResult> = comparisons
            .iter()
            .flat_map(|c| c.results.iter())
            .filter(|r| r.algorithm == *algo)
            .collect();

        if results.is_empty() {
            continue;
        }

        let count = results.len() as f64;
        let avg_depth: f64 = results.iter().map(|r| r.depth as f64).sum::<f64>() / count;
        let avg_nodes: f64 = results.iter().map(|r| r.nodes as f64).sum::<f64>() / count;
        let avg_nps: f64 = results.iter().map(|r| r.nps as f64).sum::<f64>() / count;
        let avg_ttfm: f64 = results
            .iter()
            .map(|r| r.time_to_first_move.as_secs_f64() * 1000.0)
            .sum::<f64>()
            / count;

        writeln!(
            output,
            "{:<21} {:>8.1} {:>12.0} {:>12.0} {:>10.2}",
            algo.name(),
            avg_depth,
            avg_nodes,
            avg_nps,
            avg_ttfm
        )
        .unwrap();
    }

    output
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_game_phase_from_value() {
        assert_eq!(GamePhase::from_phase_value(256), GamePhase::Opening);
        assert_eq!(GamePhase::from_phase_value(150), GamePhase::Middlegame);
        assert_eq!(GamePhase::from_phase_value(50), GamePhase::Endgame);
    }

    #[test]
    fn test_csv_record_to_line() {
        let record = CsvRecord {
            algorithm: "FullAlphaBeta".to_string(),
            position_id: "pos1".to_string(),
            phase: "opening".to_string(),
            depth: 10,
            nodes: 45678,
            time_ms: 123.4,
            nps: 370121,
            ttfm_ms: 8.1,
            best_move: "e2e4".to_string(),
            score: 35,
            branching_factor: 3.45,
            stability: 1,
        };

        let line = record.to_csv_line();
        assert!(line.contains("FullAlphaBeta"));
        assert!(line.contains("pos1"));
        assert!(line.contains("e2e4"));
    }
}
