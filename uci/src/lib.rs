//! UCI crate
//!
//! Responsibilities:
//! - Implement the UCI protocol frontend to communicate with GUIs.
//! - Translate between engine API and UCI commands/responses.
//! - Avoid engine policy; delegate computation to `engine`.

mod commands;
mod engine_handler;
mod protocol;

pub use commands::{parse_uci_command, GoCommand, UciCommand};
pub use engine_handler::UciEngine;
pub use protocol::run_uci_loop;

use std::io::{self, Write};

/// UCI protocol version
pub const UCI_VERSION: &str = "1.0";

/// Engine name
pub const ENGINE_NAME: &str = "Aether";

/// Engine version
pub const ENGINE_VERSION: &str = env!("CARGO_PKG_VERSION");

/// Engine author
pub const ENGINE_AUTHOR: &str = "Kacper Maciołek";

/// Send a UCI response to stdout
pub fn send_response(response: &str) {
    println!("{}", response);
    io::stdout().flush().ok();
}

/// Send engine info
pub fn send_info(message: &str) {
    send_response(&format!("info string {}", message));
}

/// Send best move
pub fn send_bestmove(best_move: &str, ponder: Option<&str>) {
    if let Some(ponder_move) = ponder {
        send_response(&format!("bestmove {} ponder {}", best_move, ponder_move));
    } else {
        send_response(&format!("bestmove {}", best_move));
    }
}

/// Send search info during search
pub fn send_search_info(
    depth: u8,
    seldepth: u8,
    score_cp: i32,
    nodes: u64,
    nps: u64,
    time_ms: u64,
    pv: &[String],
) {
    let mut info = format!(
        "info depth {} seldepth {} score cp {} nodes {} nps {} time {}",
        depth, seldepth, score_cp, nodes, nps, time_ms
    );

    if !pv.is_empty() {
        info.push_str(" pv");
        for mv in pv {
            info.push(' ');
            info.push_str(mv);
        }
    }

    send_response(&info);
}

/// Send hash full info
pub fn send_hashfull(per_mille: u16) {
    send_response(&format!("info hashfull {}", per_mille));
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_constants() {
        assert_eq!(ENGINE_NAME, "Aether");
        assert_eq!(ENGINE_AUTHOR, "Kacper Maciołek");
        // ENGINE_VERSION comes from CARGO_PKG_VERSION at compile time
        assert!(ENGINE_VERSION.contains('.'), "Version should have dots");
    }
}
