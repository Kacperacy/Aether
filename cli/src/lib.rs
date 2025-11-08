//! CLI crate
//!
//! Responsibilities:
//! - Provide command-line utilities and developer tooling around the engine.
//! - Parse arguments and delegate work to `engine`/`search` as needed.
//! - Keep binaries thin; avoid embedding core logic here.

use aether_types::{BoardQuery, MoveGen};
use board::{Board, BoardOps, FenOps};
use eval::{Evaluator, SimpleEvaluator};
use movegen::Generator;
use search::{AlphaBetaSearcher, SearchLimits, Searcher};
use std::io::{self, Write};

/// Simple chess CLI for testing the engine
pub struct ChessCLI {
    board: Board,
    generator: Generator,
    searcher: AlphaBetaSearcher,
}

impl ChessCLI {
    /// Create a new CLI with starting position
    pub fn new() -> Result<Self, String> {
        let board = Board::starting_position()
            .map_err(|e| format!("Failed to create starting position: {}", e))?;

        Ok(Self {
            board,
            generator: Generator,
            searcher: AlphaBetaSearcher::new(),
        })
    }

    /// Create a new CLI from FEN
    pub fn from_fen(fen: &str) -> Result<Self, String> {
        let board = Board::from_fen(fen)
            .map_err(|e| format!("Failed to parse FEN: {}", e))?;

        Ok(Self {
            board,
            generator: Generator,
            searcher: AlphaBetaSearcher::new(),
        })
    }

    /// Display the current board
    pub fn display_board(&self) {
        println!("{}", self.board.as_ascii());
        println!("FEN: {}", self.board.to_fen());
        println!("Side to move: {:?}", self.board.side_to_move());
    }

    /// List all legal moves in current position
    pub fn list_moves(&mut self) {
        let mut moves = Vec::new();
        self.generator.legal(&self.board, &mut moves);

        println!("Legal moves ({}):", moves.len());
        for (i, mv) in moves.iter().enumerate() {
            print!("{} ", mv);
            if (i + 1) % 10 == 0 {
                println!();
            }
        }
        if !moves.is_empty() && moves.len() % 10 != 0 {
            println!();
        }
    }

    /// Search for best move
    pub fn search(&mut self, depth: u8) {
        println!("Searching to depth {}...", depth);

        let limits = SearchLimits::depth(depth);
        let result = self.searcher.search(&self.board, &limits);

        if let Some(best_move) = result.best_move {
            println!("\nBest move: {}", best_move);
            println!("Score: {} centipawns", result.score);
            println!("Nodes: {}", result.info.nodes);
            println!("Time: {:?}", result.info.time_elapsed);
            println!("NPS: {}", result.info.nps);
            println!("Hash full: {}‰", result.info.hash_full);

            if !result.pv.is_empty() {
                print!("PV: ");
                for mv in &result.pv {
                    print!("{} ", mv);
                }
                println!();
            }
        } else {
            println!("No legal moves available!");
        }
    }

    /// Make a move (in UCI notation: e2e4, e7e8q for promotion)
    pub fn make_move(&mut self, move_str: &str) -> Result<(), String> {
        let mut moves = Vec::new();
        self.generator.legal(&self.board, &mut moves);

        // Find matching move
        let matching_move = moves.iter()
            .find(|m| m.to_string() == move_str)
            .copied()
            .ok_or_else(|| format!("Illegal move: {}", move_str))?;

        self.board.make_move(matching_move)
            .map_err(|e| format!("Failed to make move: {}", e))?;

        Ok(())
    }

    /// Evaluate current position
    pub fn evaluate(&self) {
        let evaluator = SimpleEvaluator::new();
        let score = evaluator.evaluate(&self.board);

        println!("Position evaluation: {} centipawns", score);
        println!("(Positive = advantage for {:?})", self.board.side_to_move());
    }

    /// Run interactive REPL
    pub fn run_repl(&mut self) -> io::Result<()> {
        println!("Aether Chess Engine CLI");
        println!("Type 'help' for commands\n");

        self.display_board();

        loop {
            print!("\n> ");
            io::stdout().flush()?;

            let mut input = String::new();
            io::stdin().read_line(&mut input)?;

            let input = input.trim();
            if input.is_empty() {
                continue;
            }

            let parts: Vec<&str> = input.split_whitespace().collect();
            let command = parts[0];

            match command {
                "help" | "h" => {
                    self.print_help();
                }
                "display" | "d" => {
                    self.display_board();
                }
                "moves" | "m" => {
                    self.list_moves();
                }
                "search" | "s" => {
                    let depth = parts.get(1)
                        .and_then(|s| s.parse::<u8>().ok())
                        .unwrap_or(5);
                    self.search(depth);
                }
                "eval" | "e" => {
                    self.evaluate();
                }
                "move" | "mv" => {
                    if parts.len() < 2 {
                        println!("Usage: move <move> (e.g., 'move e2e4')");
                        continue;
                    }
                    match self.make_move(parts[1]) {
                        Ok(()) => {
                            println!("Move {} played", parts[1]);
                            self.display_board();
                        }
                        Err(e) => println!("Error: {}", e),
                    }
                }
                "fen" => {
                    if parts.len() < 2 {
                        println!("Current FEN: {}", self.board.to_fen());
                    } else {
                        let fen = parts[1..].join(" ");
                        match Board::from_fen(&fen) {
                            Ok(board) => {
                                self.board = board;
                                println!("Position loaded from FEN");
                                self.display_board();
                            }
                            Err(e) => println!("Error parsing FEN: {}", e),
                        }
                    }
                }
                "new" => {
                    match Board::starting_position() {
                        Ok(board) => {
                            self.board = board;
                            println!("New game started");
                            self.display_board();
                        }
                        Err(e) => println!("Error: {}", e),
                    }
                }
                "quit" | "q" | "exit" => {
                    println!("Goodbye!");
                    break;
                }
                _ => {
                    println!("Unknown command: {}. Type 'help' for commands.", command);
                }
            }
        }

        Ok(())
    }

    fn print_help(&self) {
        println!("Available commands:");
        println!("  help, h                - Show this help message");
        println!("  display, d             - Display the current board");
        println!("  moves, m               - List all legal moves");
        println!("  search [depth], s [d]  - Search for best move (default depth: 5)");
        println!("  eval, e                - Evaluate current position");
        println!("  move <move>, mv <m>    - Make a move (e.g., 'move e2e4')");
        println!("  fen [fen_string]       - Load/show FEN position");
        println!("  new                    - Start a new game");
        println!("  quit, q, exit          - Exit the program");
    }
}

impl Default for ChessCLI {
    fn default() -> Self {
        Self::new().expect("Failed to create CLI")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cli_creation() {
        let cli = ChessCLI::new();
        assert!(cli.is_ok());
    }

    #[test]
    fn test_cli_from_fen() {
        let cli = ChessCLI::from_fen("rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1");
        assert!(cli.is_ok());
    }

    #[test]
    fn test_cli_invalid_fen() {
        let cli = ChessCLI::from_fen("invalid fen");
        assert!(cli.is_err());
    }
}
