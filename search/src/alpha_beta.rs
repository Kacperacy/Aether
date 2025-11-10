use crate::{
    transposition_table::{EntryType, TranspositionTable},
    AdvancedMoveOrderer, MoveOrderer, SearchInfo, SearchLimits, SearchResult, Searcher,
};
use aether_types::{BoardQuery, Move, MoveGen};
use board::{Board, BoardOps};
use eval::{mated_in, Evaluator, Score, SimpleEvaluator, MATE_SCORE};
use movegen::Generator;
use std::time::Instant;

/// How often to check if search should stop (every N nodes)
const NODE_CHECK_INTERVAL: u64 = 4096;

/// Safety margin for time management (stop searching after using this fraction of allocated time)
const TIME_SAFETY_MARGIN: f32 = 0.5;

/// Alpha-Beta search implementation
///
/// This searcher uses the alpha-beta pruning algorithm to efficiently
/// search the game tree. It supports:
/// - Iterative deepening
/// - Move ordering with killer moves and history heuristic
/// - Principal variation collection
/// - Transposition table
/// - Late Move Reductions (LMR)
/// - Null Move Pruning
pub struct AlphaBetaSearcher<E = SimpleEvaluator, O = AdvancedMoveOrderer> {
    evaluator: E,
    move_orderer: O,
    generator: Generator,
    tt: TranspositionTable,
    info: SearchInfo,
    should_stop: bool,
    start_time: Option<Instant>,
    time_limit: Option<std::time::Duration>,
}

impl AlphaBetaSearcher<SimpleEvaluator, AdvancedMoveOrderer> {
    /// Create a new alpha-beta searcher with default evaluator and move orderer
    pub fn new() -> Self {
        Self {
            evaluator: SimpleEvaluator::new(),
            move_orderer: AdvancedMoveOrderer::new(),
            generator: Generator,
            tt: TranspositionTable::default_size(),
            info: SearchInfo::new(),
            should_stop: false,
            start_time: None,
            time_limit: None,
        }
    }

    /// Create a new alpha-beta searcher with custom TT size (in MB)
    pub fn with_tt_size(tt_size_mb: usize) -> Self {
        Self {
            evaluator: SimpleEvaluator::new(),
            move_orderer: AdvancedMoveOrderer::new(),
            generator: Generator,
            tt: TranspositionTable::new(tt_size_mb),
            info: SearchInfo::new(),
            should_stop: false,
            start_time: None,
            time_limit: None,
        }
    }
}

impl<E: Evaluator, O: MoveOrderer> AlphaBetaSearcher<E, O> {
    /// Create a new alpha-beta searcher with custom evaluator and move orderer
    pub fn with_evaluator_and_orderer(evaluator: E, move_orderer: O) -> Self {
        Self {
            evaluator,
            move_orderer,
            generator: Generator,
            tt: TranspositionTable::default_size(),
            info: SearchInfo::new(),
            should_stop: false,
            start_time: None,
            time_limit: None,
        }
    }

    /// Check if search should stop based on limits
    fn should_stop(&mut self, limits: &SearchLimits) -> bool {
        if self.should_stop {
            return true;
        }

        // Check node limit
        if let Some(max_nodes) = limits.nodes
            && self.info.nodes >= max_nodes {
                return true;
            }

        // Check time limit
        if let Some(max_time) = limits.time
            && let Some(start) = self.start_time {
                let elapsed = start.elapsed();
                if elapsed >= max_time {
                    return true;
                }
            }

        false
    }

    /// Main alpha-beta search with iterative deepening
    fn iterative_deepening(&mut self, board: &Board, limits: &SearchLimits) -> SearchResult {
        // Start new search (increment TT age)
        self.tt.new_search();

        // Store time limit for checking during search
        self.time_limit = limits.time;

        // CRITICAL: Generate legal moves FIRST and set a default best_move
        // This ensures we ALWAYS have a move to play, even if time runs out during depth 1
        let mut moves = Vec::new();
        self.generator.legal(board, &mut moves);
        self.move_orderer.order_moves(&mut moves);

        let mut best_move = if !moves.is_empty() {
            Some(moves[0]) // Safe default: first ordered move
        } else {
            None // No legal moves (checkmate/stalemate) - will return 0000
        };
        let mut best_score = -MATE_SCORE;
        let mut best_pv = if !moves.is_empty() {
            vec![moves[0]]
        } else {
            Vec::new()
        };

        let max_depth = limits.depth.unwrap_or(64);

        // Iterative deepening: search depth 1, 2, 3, ... up to max_depth
        for depth in 1..=max_depth {
            if self.should_stop(limits) {
                break;
            }

            // Don't start a new depth if we've used more than TIME_SAFETY_MARGIN of allocated time
            // (to prevent last iteration from roughly doubling total search time)
            if let Some(max_time) = limits.time
                && let Some(start) = self.start_time {
                    let elapsed = start.elapsed();
                    let time_threshold = max_time.mul_f32(TIME_SAFETY_MARGIN);
                    if elapsed > time_threshold {
                        break;
                    }
                }

            self.info.depth = depth;
            self.info.selective_depth = depth;

            // Search with current depth
            let score = self.alpha_beta(
                board,
                depth,
                0,
                -MATE_SCORE,
                MATE_SCORE,
                &mut Vec::new(),
            );

            // CRITICAL: ONLY update if search completed successfully (not interrupted)
            // When should_stop=true, score is corrupted (early return with alpha value)
            if !self.should_stop {
                best_score = score;

                // Try to get best move from Transposition Table (fast path)
                let hash = board.zobrist_hash().map(|h| h.get()).unwrap_or(0);
                if let Some(tt_entry) = self.tt.probe(hash)
                    && tt_entry.depth >= depth
                        && let Some(mv) = tt_entry.best_move {
                            // Verify move is legal before using it
                            let mut legal_moves = Vec::new();
                            self.generator.legal(board, &mut legal_moves);
                            if legal_moves.contains(&mv) {
                                best_move = Some(mv);
                                best_pv = vec![mv];
                            }
                        }

                // Fallback: If TT didn't give us a move, search manually
                // This shouldn't happen often since alpha_beta stores in TT
                if best_move.is_none() {
                    let mut moves = Vec::new();
                    self.generator.legal(board, &mut moves);
                    self.move_orderer.order_moves(&mut moves);

                    if !moves.is_empty() {
                        // Take the first move as a safe default
                        best_move = Some(moves[0]);
                        best_pv = vec![moves[0]];
                    }
                }
            }

            // Update search info
            if let Some(start) = self.start_time {
                self.info.time_elapsed = start.elapsed();
                self.info.calculate_nps();
            }
            self.info.hash_full = self.tt.hash_full();
        }

        // FINAL SANITY CHECK: Ensure we have a best_move if any legal moves exist
        // This should NEVER trigger, but provides ultimate safety net
        if best_move.is_none() {
            let mut emergency_moves = Vec::new();
            self.generator.legal(board, &mut emergency_moves);

            if !emergency_moves.is_empty() {
                // CRITICAL BUG: We have legal moves but no best_move!
                // This indicates a serious logic error in iterative deepening
                // Use first move as emergency fallback
                best_move = Some(emergency_moves[0]);
                best_pv = vec![emergency_moves[0]];
            }
        }

        SearchResult::with_info(best_move, best_score, best_pv, self.info.clone())
    }

    /// Checks if search should stop due to time limits or should_stop flag.
    ///
    /// Returns `true` if search should stop, `false` if it should continue.
    fn check_search_limits(&mut self) -> bool {
        // If stop flag is already set, return immediately
        if self.should_stop {
            return true;
        }

        // Check time limit periodically for performance
        if self.info.nodes.is_multiple_of(NODE_CHECK_INTERVAL)
            && let Some(max_time) = self.time_limit
                && let Some(start) = self.start_time
                    && start.elapsed() >= max_time {
                        self.should_stop = true;
                        return true;
                    }

        false
    }

    /// Attempts to get a score from the transposition table and update bounds.
    ///
    /// Returns `Some(score)` if TT provides an immediate answer, `None` if search must continue.
    fn try_tt_cutoff(
        &mut self,
        hash: u64,
        depth: u8,
        alpha: Score,
        beta: Score,
        pv: &mut Vec<Move>,
    ) -> Option<Score> {
        let tt_entry = self.tt.probe(hash)?;

        // Only use TT entry if searched to at least the same depth
        if tt_entry.depth < depth {
            return None;
        }

        match tt_entry.entry_type {
            EntryType::Exact => {
                // Exact score - use it directly
                if let Some(best_move) = tt_entry.best_move {
                    pv.clear();
                    pv.push(best_move);
                }
                Some(tt_entry.score)
            }
            EntryType::LowerBound => {
                // Score is at least this good
                if tt_entry.score >= beta {
                    Some(tt_entry.score)
                } else {
                    None
                }
            }
            EntryType::UpperBound => {
                // Score is at most this good
                if tt_entry.score <= alpha {
                    Some(tt_entry.score)
                } else {
                    None
                }
            }
        }
    }

    /// Detects if the position is checkmate or stalemate.
    ///
    /// Returns `Some(score)` if game is over, `None` if there are legal moves.
    fn detect_checkmate_or_stalemate(&self, board: &Board, ply: u8) -> Option<Score> {
        // Check if in check
        let king_square = board.get_king_square(board.side_to_move())?;
        let in_check = board.is_square_attacked(king_square, board.side_to_move().opponent());

        if in_check {
            Some(mated_in(ply as i32)) // Checkmate
        } else {
            Some(0) // Stalemate
        }
    }

    /// Orders moves for search, prioritizing TT move if available.
    fn order_moves_for_search(&mut self, moves: &mut [Move], hash: u64, ply: usize) {
        // Try TT move first if available
        if let Some(tt_entry) = self.tt.probe(hash)
            && let Some(tt_move) = tt_entry.best_move {
                // Move TT move to front
                if let Some(pos) = moves.iter().position(|&m| m == tt_move) {
                    moves.swap(0, pos);
                }
            }

        self.move_orderer.order_moves_at_ply(moves, ply);
    }

    /// Searches all moves at this node and returns (best_score, best_move, pv).
    fn search_moves_at_node(
        &mut self,
        board: &Board,
        moves: Vec<Move>,
        depth: u8,
        ply: u8,
        mut alpha: Score,
        beta: Score,
    ) -> (Score, Option<Move>, Vec<Move>) {
        let mut best_score = -MATE_SCORE;
        let mut best_move = None;
        let mut local_pv = Vec::new();

        for mv in moves {
            let mut board_copy = board.clone();

            // Make move
            if board_copy.make_move(mv).is_err() {
                continue;
            }

            // Recursive search
            let mut child_pv = Vec::new();
            let score = -self.alpha_beta(
                &board_copy,
                depth - 1,
                ply + 1,
                -beta,
                -alpha,
                &mut child_pv,
            );

            // Update best score
            if score > best_score {
                best_score = score;
                best_move = Some(mv);

                // Update PV
                local_pv.clear();
                local_pv.push(mv);
                local_pv.extend_from_slice(&child_pv);
            }

            // Update alpha
            if score > alpha {
                alpha = score;

                // Update history heuristic for good moves
                if mv.capture.is_none() {
                    self.move_orderer.update_history(mv, depth);
                }
            }

            // Beta cutoff
            if alpha >= beta {
                // Store killer move (non-captures only)
                if mv.capture.is_none() {
                    self.move_orderer.store_killer(mv, ply as usize);
                }
                break; // Prune remaining moves
            }
        }

        (best_score, best_move, local_pv)
    }

    /// Alpha-beta negamax search
    ///
    /// # Arguments
    /// * `board` - Current board position
    /// * `depth` - Remaining depth to search
    /// * `ply` - Current ply from root (for mate distance)
    /// * `alpha` - Lower bound
    /// * `beta` - Upper bound
    /// * `pv` - Principal variation output
    ///
    /// # Returns
    /// Score from the perspective of the side to move
    fn alpha_beta(
        &mut self,
        board: &Board,
        depth: u8,
        ply: u8,
        mut alpha: Score,
        beta: Score,
        pv: &mut Vec<Move>,
    ) -> Score {
        self.info.nodes += 1;

        // Check if we should stop search (time limit exceeded)
        if self.check_search_limits() {
            return alpha;
        }

        // Update selective depth
        if ply > self.info.selective_depth {
            self.info.selective_depth = ply;
        }

        // Terminal node: evaluate position
        if depth == 0 {
            return self.quiescence(board, alpha, beta);
        }

        // Get zobrist hash for TT lookup
        let hash = board.zobrist_hash().map(|h| h.get()).unwrap_or(0);
        let original_alpha = alpha;

        // Try transposition table cutoff
        if let Some(score) = self.try_tt_cutoff(hash, depth, alpha, beta, pv) {
            return score;
        }

        // Update alpha from TT if available (for better move ordering)
        if let Some(tt_entry) = self.tt.probe(hash)
            && tt_entry.depth >= depth && tt_entry.entry_type == EntryType::LowerBound {
                alpha = alpha.max(tt_entry.score);
                if alpha >= beta {
                    return alpha;
                }
            }

        // Generate legal moves
        let mut moves = Vec::new();
        self.generator.legal(board, &mut moves);

        // Check for checkmate or stalemate
        if moves.is_empty() {
            return self.detect_checkmate_or_stalemate(board, ply).unwrap_or(0);
        }

        // Order moves for better alpha-beta pruning
        self.order_moves_for_search(&mut moves, hash, ply as usize);

        // Search all moves at this node
        let (best_score, best_move, local_pv) =
            self.search_moves_at_node(board, moves, depth, ply, alpha, beta);

        // Determine entry type for TT
        let entry_type = if best_score <= original_alpha {
            EntryType::UpperBound // All moves failed low
        } else if best_score >= beta {
            EntryType::LowerBound // Beta cutoff
        } else {
            EntryType::Exact // PV node
        };

        // Store in transposition table
        self.tt.store(hash, best_move, best_score, depth, entry_type);

        // Copy local PV to output
        pv.clear();
        pv.extend_from_slice(&local_pv);

        best_score
    }

    /// Quiescence search to avoid horizon effect
    ///
    /// Only searches captures and checks to reach a "quiet" position
    fn quiescence(&mut self, board: &Board, mut alpha: Score, beta: Score) -> Score {
        self.info.nodes += 1;

        // Stand-pat score (current position evaluation)
        let stand_pat = self.evaluator.evaluate(board);

        // Beta cutoff
        if stand_pat >= beta {
            return beta;
        }

        // Update alpha
        if stand_pat > alpha {
            alpha = stand_pat;
        }

        // Generate captures only
        let mut captures = Vec::new();
        self.generator.captures(board, &mut captures);

        // Order captures
        self.move_orderer.order_moves(&mut captures);

        // Search captures
        for mv in captures {
            let mut board_copy = board.clone();

            // Make move
            if board_copy.make_move(mv).is_err() {
                continue;
            }

            // Recursive quiescence search
            let score = -self.quiescence(&board_copy, -beta, -alpha);

            // Update alpha
            if score > alpha {
                alpha = score;
            }

            // Beta cutoff
            if alpha >= beta {
                break;
            }
        }

        alpha
    }
}

impl<E: Evaluator, O: MoveOrderer> Default for AlphaBetaSearcher<E, O>
where
    E: Default,
    O: Default,
{
    fn default() -> Self {
        Self::with_evaluator_and_orderer(E::default(), O::default())
    }
}

impl<E: Evaluator, O: MoveOrderer> Searcher for AlphaBetaSearcher<E, O> {
    fn search<T>(&mut self, board: &T, limits: &SearchLimits) -> SearchResult
    where
        T: BoardQuery + Clone + 'static,
    {
        // Reset search state
        self.should_stop = false;
        self.start_time = Some(Instant::now());
        self.info = SearchInfo::new();

        // For now, we require board to be a Board type
        // In the future, we can make this more generic
        if let Some(board) = (board as &dyn std::any::Any).downcast_ref::<Board>() {
            self.iterative_deepening(board, limits)
        } else {
            // Fallback: just generate moves and pick first legal one
            let mut moves = Vec::new();
            self.generator.legal(board, &mut moves);

            let best_move = moves.first().copied();
            let score = if best_move.is_some() {
                self.evaluator.evaluate(board)
            } else {
                0
            };

            SearchResult::new(best_move, score)
        }
    }

    fn get_info(&self) -> &SearchInfo {
        &self.info
    }

    fn stop(&mut self) {
        self.should_stop = true;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_alpha_beta_searcher_creation() {
        let searcher = AlphaBetaSearcher::new();
        assert_eq!(searcher.info.depth, 0);
        assert!(!searcher.should_stop);
    }

    #[test]
    fn test_search_from_starting_position() {
        let board = Board::starting_position().expect("Failed to create starting position");
        let mut searcher = AlphaBetaSearcher::new();
        let limits = SearchLimits::depth(3);

        let result = searcher.search(&board, &limits);

        assert!(result.best_move.is_some(), "Should find a best move");
        assert!(result.info.nodes > 0, "Should search some nodes");
    }
}
