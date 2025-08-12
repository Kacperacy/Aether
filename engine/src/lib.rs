//! Engine crate
//!
//! Responsibilities:
//! - Coordinate search, evaluation, and time management to produce moves.
//! - Maintain engine state and provide a clean API for frontends (UCI/CLI).
//! - Keep clear layering: consume `search`/`eval` without leaking UI concerns.
