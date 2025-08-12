//! Evaluation crate
//!
//! Responsibilities:
//! - Provide position evaluation heuristics and feature extraction used by the search.
//! - Remain decoupled from policy/state management; operate on data exposed via `aether_types`.
//! - Avoid heavy dependencies to keep compile times reasonable and layering clean.
//!
//! Typical consumers: `search`, `engine`, benchmarking tools.
