//! Search crate
//!
//! Responsibilities:
//! - Implement game-tree search (e.g., minimax, alpha-beta, iterative deepening, quiescence).
//! - Consume move generation (`movegen`) and evaluation (`eval`) without embedding policy into lower layers.
//! - Expose a clean API for the engine to request best moves and principal variations.
//!
//! This crate should avoid direct UI/CLI dependencies and remain compute-focused.
