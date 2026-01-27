//! EPD (Extended Position Description) file parser
//!
//! EPD format: FEN [operations]
//! Example: "rnbqkbnr/pppppppp/8/8/4P3/8/PPPP1PPP/RNBQKBNR b KQkq e3 bm e5; id \"pos1\";"

use std::fs::File;
use std::io::{self, BufRead, BufReader};
use std::path::Path;

/// Represents a single EPD position with optional operations
#[derive(Debug, Clone)]
pub struct EpdPosition {
    /// The FEN string (position only, without halfmove/fullmove clocks)
    pub fen: String,
    /// Best move (bm operation)
    pub best_move: Option<String>,
    /// Avoid moves (am operation)
    pub avoid_moves: Vec<String>,
    /// Position ID (id operation)
    pub id: Option<String>,
}

impl EpdPosition {
    /// Parse an EPD line into an EpdPosition
    pub fn parse(line: &str) -> Option<Self> {
        let line = line.trim();
        if line.is_empty() || line.starts_with('#') {
            return None;
        }

        // EPD format: piece_placement side castling en_passant [operations]
        // FEN has 6 fields, EPD has 4 fields + operations
        let parts: Vec<&str> = line.split_whitespace().collect();
        if parts.len() < 4 {
            return None;
        }

        // Extract the 4 mandatory FEN fields
        let fen_base = format!("{} {} {} {}", parts[0], parts[1], parts[2], parts[3]);

        // Add default halfmove and fullmove clocks to make it a valid FEN
        let fen = format!("{} 0 1", fen_base);

        // Parse operations (everything after the 4 FEN fields)
        let mut best_move = None;
        let mut avoid_moves = Vec::new();
        let mut id = None;

        if parts.len() > 4 {
            let operations_str = parts[4..].join(" ");
            let operations: Vec<&str> = operations_str.split(';').collect();

            for op in operations {
                let op = op.trim();
                if op.is_empty() {
                    continue;
                }

                if let Some(rest) = op.strip_prefix("bm ") {
                    // Best move - take first move only
                    best_move = rest.split_whitespace().next().map(String::from);
                } else if let Some(rest) = op.strip_prefix("am ") {
                    // Avoid moves
                    avoid_moves = rest.split_whitespace().map(String::from).collect();
                } else if let Some(rest) = op.strip_prefix("id ") {
                    // ID - remove quotes if present
                    id = Some(rest.trim_matches('"').to_string());
                }
            }
        }

        Some(EpdPosition {
            fen,
            best_move,
            avoid_moves,
            id,
        })
    }
}

/// Load positions from an EPD file
///
/// # Arguments
/// * `path` - Path to the EPD file
/// * `limit` - Optional maximum number of positions to load
///
/// # Returns
/// Vector of parsed EPD positions
pub fn load_epd_file<P: AsRef<Path>>(path: P, limit: Option<usize>) -> io::Result<Vec<EpdPosition>> {
    let file = File::open(path)?;
    let reader = BufReader::new(file);

    let mut positions = Vec::new();
    let max_positions = limit.unwrap_or(usize::MAX);

    for line in reader.lines() {
        let line = line?;
        if let Some(pos) = EpdPosition::parse(&line) {
            positions.push(pos);
            if positions.len() >= max_positions {
                break;
            }
        }
    }

    Ok(positions)
}

/// Load positions from a simple FEN file (one FEN per line)
pub fn load_fen_file<P: AsRef<Path>>(path: P, limit: Option<usize>) -> io::Result<Vec<String>> {
    let file = File::open(path)?;
    let reader = BufReader::new(file);

    let mut positions = Vec::new();
    let max_positions = limit.unwrap_or(usize::MAX);

    for line in reader.lines() {
        let line = line?;
        let line = line.trim();
        if !line.is_empty() && !line.starts_with('#') {
            positions.push(line.to_string());
            if positions.len() >= max_positions {
                break;
            }
        }
    }

    Ok(positions)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_simple_epd() {
        let line = "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq -";
        let pos = EpdPosition::parse(line).unwrap();
        assert_eq!(
            pos.fen,
            "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1"
        );
    }

    #[test]
    fn test_parse_epd_with_bm() {
        let line = "rnbqkbnr/pppppppp/8/8/4P3/8/PPPP1PPP/RNBQKBNR b KQkq e3 bm e5;";
        let pos = EpdPosition::parse(line).unwrap();
        assert_eq!(pos.best_move, Some("e5".to_string()));
    }

    #[test]
    fn test_parse_epd_with_id() {
        let line = "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - bm e4; id \"starting\";";
        let pos = EpdPosition::parse(line).unwrap();
        assert_eq!(pos.best_move, Some("e4".to_string()));
        assert_eq!(pos.id, Some("starting".to_string()));
    }

    #[test]
    fn test_skip_empty_and_comments() {
        assert!(EpdPosition::parse("").is_none());
        assert!(EpdPosition::parse("# comment").is_none());
        assert!(EpdPosition::parse("   ").is_none());
    }

    #[test]
    fn test_invalid_epd() {
        // Too few fields
        assert!(EpdPosition::parse("rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq").is_none());
    }
}
