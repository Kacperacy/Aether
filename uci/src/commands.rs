//! UCI command parsing

use std::time::Duration;

/// UCI commands that can be received from the GUI
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum UciCommand {
    /// Initialize UCI mode
    Uci,
    
    /// Check if engine is ready
    IsReady,
    
    /// Start a new game
    UciNewGame,
    
    /// Set position
    Position {
        fen: Option<String>,
        moves: Vec<String>,
    },
    
    /// Start searching
    Go(GoCommand),
    
    /// Stop searching
    Stop,
    
    /// Quit the engine
    Quit,
    
    /// Set option
    SetOption {
        name: String,
        value: Option<String>,
    },
    
    /// Unknown command (for debugging)
    Unknown(String),
}

/// Parameters for the "go" command
#[derive(Debug, Clone, PartialEq, Eq)]
#[derive(Default)]
pub struct GoCommand {
    /// Search for this many moves
    pub searchmoves: Vec<String>,
    
    /// Search in ponder mode
    pub ponder: bool,
    
    /// White has this much time left (ms)
    pub wtime: Option<u64>,
    
    /// Black has this much time left (ms)
    pub btime: Option<u64>,
    
    /// White increment per move (ms)
    pub winc: Option<u64>,
    
    /// Black increment per move (ms)
    pub binc: Option<u64>,
    
    /// Number of moves to next time control
    pub movestogo: Option<u32>,
    
    /// Search to this depth
    pub depth: Option<u8>,
    
    /// Search this many nodes
    pub nodes: Option<u64>,
    
    /// Search for mate in this many moves
    pub mate: Option<u32>,
    
    /// Search for this much time (ms)
    pub movetime: Option<u64>,
    
    /// Search infinitely
    pub infinite: bool,
}


impl GoCommand {
    /// Calculate search time for the given side
    ///
    /// # Arguments
    /// * `is_white` - Whether white is to move
    /// * `move_overhead_ms` - Time to reserve for GUI/network latency (milliseconds)
    pub fn calculate_time(&self, is_white: bool, move_overhead_ms: u64) -> Option<Duration> {
        let time_left = if is_white {
            self.wtime
        } else {
            self.btime
        };

        let increment = if is_white {
            self.winc.unwrap_or(0)
        } else {
            self.binc.unwrap_or(0)
        };

        if let Some(movetime) = self.movetime {
            // Apply move overhead to movetime
            let adjusted_time = movetime.saturating_sub(move_overhead_ms);
            return Some(Duration::from_millis(adjusted_time.max(1)));
        }

        if let Some(time) = time_left {
            // Simple time management: use 1/30 of remaining time + increment
            let moves_to_go = self.movestogo.unwrap_or(30);
            let time_per_move = (time / moves_to_go as u64) + increment;

            // Apply move overhead
            let adjusted_time = time_per_move.saturating_sub(move_overhead_ms);
            Some(Duration::from_millis(adjusted_time.max(1)))
        } else {
            None
        }
    }
}

/// Parse a UCI command from a string
pub fn parse_uci_command(line: &str) -> UciCommand {
    let parts: Vec<&str> = line.split_whitespace().collect();
    
    if parts.is_empty() {
        return UciCommand::Unknown(String::new());
    }
    
    match parts[0] {
        "uci" => UciCommand::Uci,
        "isready" => UciCommand::IsReady,
        "ucinewgame" => UciCommand::UciNewGame,
        "position" => parse_position(&parts[1..]),
        "go" => UciCommand::Go(parse_go(&parts[1..])),
        "stop" => UciCommand::Stop,
        "quit" => UciCommand::Quit,
        "setoption" => parse_setoption(&parts[1..]),
        _ => UciCommand::Unknown(line.to_string()),
    }
}

fn parse_position(parts: &[&str]) -> UciCommand {
    if parts.is_empty() {
        return UciCommand::Unknown("position".to_string());
    }
    
    let mut fen = None;
    let mut moves = Vec::new();
    let mut i = 0;
    
    if parts[0] == "startpos" {
        i = 1;
    } else if parts[0] == "fen" {
        // Collect FEN parts until "moves" or end
        let mut fen_parts = Vec::new();
        i = 1;
        while i < parts.len() && parts[i] != "moves" {
            fen_parts.push(parts[i]);
            i += 1;
        }
        fen = Some(fen_parts.join(" "));
    }
    
    // Parse moves if present
    if i < parts.len() && parts[i] == "moves" {
        i += 1;
        while i < parts.len() {
            moves.push(parts[i].to_string());
            i += 1;
        }
    }
    
    UciCommand::Position { fen, moves }
}

fn parse_go(parts: &[&str]) -> GoCommand {
    let mut cmd = GoCommand::default();
    let mut i = 0;
    
    while i < parts.len() {
        match parts[i] {
            "searchmoves" => {
                i += 1;
                while i < parts.len() && !is_go_keyword(parts[i]) {
                    cmd.searchmoves.push(parts[i].to_string());
                    i += 1;
                }
                continue;
            }
            "ponder" => cmd.ponder = true,
            "wtime" => {
                i += 1;
                if i < parts.len() {
                    cmd.wtime = parts[i].parse().ok();
                }
            }
            "btime" => {
                i += 1;
                if i < parts.len() {
                    cmd.btime = parts[i].parse().ok();
                }
            }
            "winc" => {
                i += 1;
                if i < parts.len() {
                    cmd.winc = parts[i].parse().ok();
                }
            }
            "binc" => {
                i += 1;
                if i < parts.len() {
                    cmd.binc = parts[i].parse().ok();
                }
            }
            "movestogo" => {
                i += 1;
                if i < parts.len() {
                    cmd.movestogo = parts[i].parse().ok();
                }
            }
            "depth" => {
                i += 1;
                if i < parts.len() {
                    cmd.depth = parts[i].parse().ok();
                }
            }
            "nodes" => {
                i += 1;
                if i < parts.len() {
                    cmd.nodes = parts[i].parse().ok();
                }
            }
            "mate" => {
                i += 1;
                if i < parts.len() {
                    cmd.mate = parts[i].parse().ok();
                }
            }
            "movetime" => {
                i += 1;
                if i < parts.len() {
                    cmd.movetime = parts[i].parse().ok();
                }
            }
            "infinite" => cmd.infinite = true,
            _ => {}
        }
        i += 1;
    }
    
    cmd
}

fn is_go_keyword(s: &str) -> bool {
    matches!(
        s,
        "wtime" | "btime" | "winc" | "binc" | "movestogo" | "depth" | "nodes" | "mate"
            | "movetime" | "infinite" | "ponder"
    )
}

fn parse_setoption(parts: &[&str]) -> UciCommand {
    if parts.len() < 2 || parts[0] != "name" {
        return UciCommand::Unknown("setoption".to_string());
    }
    
    let mut name_parts = Vec::new();
    let mut i = 1;
    
    while i < parts.len() && parts[i] != "value" {
        name_parts.push(parts[i]);
        i += 1;
    }
    
    let name = name_parts.join(" ");
    let mut value = None;
    
    if i < parts.len() && parts[i] == "value" {
        i += 1;
        if i < parts.len() {
            value = Some(parts[i..].join(" "));
        }
    }
    
    UciCommand::SetOption { name, value }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_parse_uci() {
        assert_eq!(parse_uci_command("uci"), UciCommand::Uci);
        assert_eq!(parse_uci_command("isready"), UciCommand::IsReady);
        assert_eq!(parse_uci_command("quit"), UciCommand::Quit);
    }
    
    #[test]
    fn test_parse_position_startpos() {
        match parse_uci_command("position startpos") {
            UciCommand::Position { fen, moves } => {
                assert_eq!(fen, None);
                assert_eq!(moves.len(), 0);
            }
            _ => panic!("Expected Position command"),
        }
    }
    
    #[test]
    fn test_parse_position_with_moves() {
        match parse_uci_command("position startpos moves e2e4 e7e5") {
            UciCommand::Position { fen, moves } => {
                assert_eq!(fen, None);
                assert_eq!(moves, vec!["e2e4", "e7e5"]);
            }
            _ => panic!("Expected Position command"),
        }
    }
    
    #[test]
    fn test_parse_go_depth() {
        match parse_uci_command("go depth 10") {
            UciCommand::Go(cmd) => {
                assert_eq!(cmd.depth, Some(10));
            }
            _ => panic!("Expected Go command"),
        }
    }
    
    #[test]
    fn test_parse_go_time() {
        match parse_uci_command("go wtime 300000 btime 300000 winc 5000 binc 5000") {
            UciCommand::Go(cmd) => {
                assert_eq!(cmd.wtime, Some(300000));
                assert_eq!(cmd.btime, Some(300000));
                assert_eq!(cmd.winc, Some(5000));
                assert_eq!(cmd.binc, Some(5000));
            }
            _ => panic!("Expected Go command"),
        }
    }
}
