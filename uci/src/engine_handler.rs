//! UCI Engine handler - manages engine state and search

use crate::{commands::GoCommand, send_bestmove, send_hashfull, send_info, send_search_info};
use aether_types::{BoardQuery, MoveGen};
use board::{Board, BoardOps, FenOps};
use movegen::Generator;
use search::{AlphaBetaSearcher, SearchLimits, Searcher};

/// UCI Engine state
pub struct UciEngine {
    board: Board,
    searcher: AlphaBetaSearcher,
    generator: Generator,
    hash_size_mb: usize, // Current TT size
    move_overhead_ms: u64, // Move overhead in milliseconds (for online play)
}

impl UciEngine {
    /// Create a new UCI engine
    pub fn new() -> Result<Self, String> {
        let board = Board::starting_position()
            .map_err(|e| format!("Failed to create starting position: {}", e))?;

        Ok(Self {
            board,
            searcher: AlphaBetaSearcher::with_tt_size(64), // 64 MB transposition table
            generator: Generator,
            hash_size_mb: 64,
            move_overhead_ms: 10, // Default 10ms overhead
        })
    }
    
    /// Set position from FEN
    pub fn set_position(&mut self, fen: Option<String>, moves: Vec<String>) -> Result<(), String> {
        // Load position
        self.board = if let Some(fen_str) = fen {
            Board::from_fen(&fen_str)
                .map_err(|e| format!("Invalid FEN: {}", e))?
        } else {
            Board::starting_position()
                .map_err(|e| format!("Failed to create starting position: {}", e))?
        };
        
        // Apply moves
        for move_str in moves {
            self.make_move(&move_str)?;
        }
        
        Ok(())
    }
    
    /// Make a move on the board
    fn make_move(&mut self, move_str: &str) -> Result<(), String> {
        let mut legal_moves = Vec::new();
        self.generator.legal(&self.board, &mut legal_moves);
        
        // Find matching move
        let matching_move = legal_moves
            .iter()
            .find(|m| m.to_string() == move_str)
            .copied()
            .ok_or_else(|| format!("Illegal move: {}", move_str))?;
        
        self.board.make_move(matching_move)
            .map_err(|e| format!("Failed to make move: {}", e))?;
        
        Ok(())
    }
    
    /// Start search
    pub fn go(&mut self, go_cmd: GoCommand) {
        // Calculate search limits
        let mut limits = SearchLimits::default();

        if go_cmd.infinite {
            limits = SearchLimits::infinite();
        } else if let Some(depth) = go_cmd.depth {
            limits.depth = Some(depth);
        } else if let Some(nodes) = go_cmd.nodes {
            limits.nodes = Some(nodes);
        } else if let Some(time) = go_cmd.calculate_time(
            self.board.side_to_move() == aether_types::Color::White,
            self.move_overhead_ms
        ) {
            limits.time = Some(time);
        } else {
            // Default: depth 6
            limits.depth = Some(6);
        }
        
        send_info(&format!("Starting search with limits: depth={:?}, time={:?}, nodes={:?}", 
            limits.depth, limits.time, limits.nodes));
        
        // Search
        let result = self.searcher.search(&self.board, &limits);
        
        // Send final info
        let pv_strings: Vec<String> = result.pv.iter().map(|m| m.to_string()).collect();
        send_search_info(
            result.info.depth,
            result.info.selective_depth,
            result.score,
            result.info.nodes,
            result.info.nps,
            result.info.time_elapsed.as_millis() as u64,
            &pv_strings,
        );
        
        send_hashfull(result.info.hash_full);
        
        // Send best move
        if let Some(best_move) = result.best_move {
            let ponder = result.pv.get(1).map(|m| m.to_string());
            send_bestmove(&best_move.to_string(), ponder.as_deref());
        } else {
            send_bestmove("0000", None); // No legal moves
        }
    }
    
    /// Reset for new game
    pub fn new_game(&mut self) {
        self.board = Board::starting_position().expect("Failed to create starting position");
    }

    /// Set hash table size (in MB)
    pub fn set_hash_size(&mut self, size_mb: usize) {
        if size_mb != self.hash_size_mb && size_mb > 0 && size_mb <= 1024 {
            self.hash_size_mb = size_mb;
            self.searcher = AlphaBetaSearcher::with_tt_size(size_mb);
        }
    }

    /// Set move overhead (in milliseconds)
    ///
    /// Move overhead is the time reserved before each move to account for
    /// GUI/network latency. This is crucial for online play (e.g., Lichess).
    /// Typical values: 10ms (local), 100-300ms (online).
    pub fn set_move_overhead(&mut self, overhead_ms: u64) {
        if overhead_ms <= 5000 { // Max 5 seconds overhead
            self.move_overhead_ms = overhead_ms;
        }
    }

    /// Get current move overhead setting
    pub fn move_overhead(&self) -> u64 {
        self.move_overhead_ms
    }

    /// Stop current search
    ///
    /// Note: Current implementation uses synchronous search, so this cannot
    /// interrupt an ongoing search. For production use with Lichess, this should
    /// be implemented using async search or a stop flag checked during search.
    pub fn stop(&mut self) {
        self.searcher.stop();
    }
}

impl Default for UciEngine {
    fn default() -> Self {
        Self::new().expect("Failed to create UCI engine")
    }
}
