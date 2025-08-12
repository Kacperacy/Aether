//! CLI crate
//!
//! Responsibilities:
//! - Provide command-line utilities and developer tooling around the engine.
//! - Parse arguments and delegate work to `engine`/`search` as needed.
//! - Keep binaries thin; avoid embedding core logic here.
