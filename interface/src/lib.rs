//! Interface crate - UCI protocol and engine communication
//!
//! This crate provides the UCI (Universal Chess Interface) implementation
//! for communicating with chess GUIs like Arena, CuteChess, etc.

pub mod benchmark;
pub mod epd;
pub mod handler;
pub mod uci;

pub use benchmark::*;
pub use handler::UciHandler;
pub use uci::*;
