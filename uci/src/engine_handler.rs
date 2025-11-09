//! UCI Engine handler - manages engine state and search

use crate::{commands::GoCommand, send_bestmove, send_hashfull, send_info, send_search_info};
use aether_types::{BoardQuery, MoveGen};
use board::{Board, BoardOps, FenOps};
use movegen::Generator;
use opening::OpeningBook;
use search::{AlphaBetaSearcher, SearchLimits, Searcher};

/// UCI Engine state
pub struct UciEngine {
    board: Board,
    searcher: AlphaBetaSearcher,
    generator: Generator,
    opening_book: OpeningBook,
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
            opening_book: OpeningBook::default_book(),
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

    /// Try to get a move from the opening book
    fn try_book_move(&self) -> Option<String> {
        // Get board hash
        let hash = self.board.compute_zobrist_hash();

        // Query the book
        if let Some(book_move) = self.opening_book.pick_move(hash) {
            // Verify the move is legal
            let mut legal_moves = Vec::new();
            self.generator.legal(&self.board, &mut legal_moves);

            let is_legal = legal_moves.iter().any(|m| m.to_string() == book_move);

            if is_legal {
                send_info(&format!("Book move: {}", book_move));
                return Some(book_move);
            }
        }

        None
    }
    
    /// Start search
    pub fn go(&mut self, go_cmd: GoCommand) {
        // Try opening book first
        if let Some(book_move) = self.try_book_move() {
            // Send book move immediately (no search needed)
            send_bestmove(&book_move, None);
            return;
        }

        // Calculate search limits
        let mut limits = SearchLimits::default();

        if go_cmd.infinite {
            limits = SearchLimits::infinite();
        } else if let Some(depth) = go_cmd.depth {
            limits.depth = Some(depth);
            limits.time = None;  // Clear time limit when depth is specified
        } else if let Some(nodes) = go_cmd.nodes {
            limits.nodes = Some(nodes);
            limits.depth = None;  // Clear depth limit when nodes are specified
        } else if let Some(time) = go_cmd.calculate_time(
            self.board.side_to_move() == aether_types::Color::White,
            self.move_overhead_ms
        ) {
            limits.time = Some(time);
            limits.depth = None;  // Clear depth limit when using time control!
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
            // CRITICAL ERROR: No legal moves found
            // This should NEVER happen in a normal game (GUI ends game before this)
            // Log diagnostic information
            send_info("ERROR: Search returned no best move!");
            send_info(&format!("Position FEN: {}", self.board.to_fen()));

            // Check if there are actually any legal moves
            let mut legal_moves = Vec::new();
            self.generator.legal(&self.board, &mut legal_moves);
            send_info(&format!("Legal moves count: {}", legal_moves.len()));

            if !legal_moves.is_empty() {
                // BUG: We have legal moves but search didn't return one!
                // Use first legal move as emergency fallback
                send_info(&format!("EMERGENCY: Using first legal move: {}", legal_moves[0]));
                send_bestmove(&legal_moves[0].to_string(), None);
            } else {
                // No legal moves (checkmate or stalemate)
                send_info("No legal moves available (checkmate/stalemate)");
                send_bestmove("0000", None);
            }
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
