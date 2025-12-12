//! UCI Handler - Main loop that connects UCI protocol with the chess engine

use crate::uci::{
    EngineInfo, InfoResponse, OptionInfo, OptionType, SearchParams, UciCommand, UciInput,
    UciResponse, send_response, send_responses,
};
use aether_core::{Color, Move, Piece, Square};
use board::{Board, BoardOps, BoardQuery, FenOps, polyglot_hash};
use engine::Engine;
use opening::{OpeningBook, convert_polyglot_castling};
use std::str::FromStr;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};

/// Engine options
#[derive(Debug, Clone)]
pub struct EngineOptions {
    /// Hash table size in MB
    pub hash_size: usize,
    /// Number of threads (for future multi-threading)
    pub threads: usize,
    /// Whether to show debug output
    pub debug: bool,
    /// Whether to use an own book
    pub own_book: bool,
    /// Path to the opening book file
    pub book_path: Option<String>,
    /// If true, always play the best book move, otherwise play a random book move
    pub book_best_move: bool,
}

impl Default for EngineOptions {
    fn default() -> Self {
        Self {
            hash_size: 16,
            threads: 1,
            debug: false,
            own_book: false,
            book_path: None,
            book_best_move: true,
        }
    }
}

/// UCI Handler - manages the main UCI loop
pub struct UciHandler {
    /// Engine information
    info: EngineInfo,
    /// Current board position
    board: Board,
    /// Chess engine (search + evaluation)
    engine: Engine,
    /// Engine options
    options: EngineOptions,
    /// Stop flag for search
    stop_flag: Arc<AtomicBool>,
    /// Opening book
    book: Option<OpeningBook>,
}

impl UciHandler {
    /// Create a new UCI handler
    pub fn new() -> Self {
        let engine = Engine::new(16);
        let stop_flag = engine.stop_flag();

        // Load the embedded default opening book
        let book = OpeningBook::default_book().ok();

        Self {
            info: EngineInfo::default(),
            board: Board::starting_position().expect("Failed to create starting position"),
            engine,
            options: EngineOptions::default(),
            stop_flag,
            book,
        }
    }

    /// Run the main UCI loop
    pub fn run(&mut self) {
        let mut input = UciInput::new();

        loop {
            if let Some(cmd) = input.read_command() {
                match cmd {
                    UciCommand::Quit => break,
                    _ => self.handle_command(cmd),
                }
            } else {
                // EOF or error
                break;
            }
        }
    }

    /// Handle a single UCI command
    fn handle_command(&mut self, cmd: UciCommand) {
        match cmd {
            UciCommand::Uci => self.cmd_uci(),
            UciCommand::Debug(on) => self.cmd_debug(on),
            UciCommand::IsReady => self.cmd_isready(),
            UciCommand::SetOption { name, value } => self.cmd_setoption(&name, value),
            UciCommand::Register => {} // Not implemented - optional UCI feature
            UciCommand::UciNewGame => self.cmd_ucinewgame(),
            UciCommand::Position { fen, moves } => self.cmd_position(fen, moves),
            UciCommand::Go(params) => self.cmd_go(params),
            UciCommand::Stop => self.cmd_stop(),
            UciCommand::PonderHit => {} // Not implemented - requires pondering support
            UciCommand::Quit => {}      // Handled in main loop
            UciCommand::Display => self.cmd_display(),
            UciCommand::Perft(depth) => self.cmd_perft(depth),
            UciCommand::Unknown(s) => {
                if self.options.debug {
                    send_response(&UciResponse::Info(
                        InfoResponse::new().with_string(format!("Unknown command: {}", s)),
                    ));
                }
            }
        }
    }

    /// Handle "uci" command
    fn cmd_uci(&self) {
        send_responses(&[
            UciResponse::IdName(self.info.name.clone()),
            UciResponse::IdAuthor(self.info.author.clone()),
        ]);

        // Send available options
        send_response(&UciResponse::Option(OptionInfo {
            name: "Hash".to_string(),
            option_type: OptionType::Spin {
                default: 16,
                min: 1,
                max: 1024,
            },
        }));

        send_response(&UciResponse::Option(OptionInfo {
            name: "Threads".to_string(),
            option_type: OptionType::Spin {
                default: 1,
                min: 1,
                max: 1,
            },
        }));

        send_response(&UciResponse::Option(OptionInfo {
            name: "OwnBook".to_string(),
            option_type: OptionType::Check { default: false },
        }));

        send_response(&UciResponse::Option(OptionInfo {
            name: "BookFile".to_string(),
            option_type: OptionType::String {
                default: String::new(),
            },
        }));

        send_response(&UciResponse::Option(OptionInfo {
            name: "BookBestMove".to_string(),
            option_type: OptionType::Check { default: true },
        }));

        send_response(&UciResponse::UciOk);
    }

    /// Handle "debug" command
    fn cmd_debug(&mut self, on: bool) {
        self.options.debug = on;
    }

    /// Handle "isready" command
    fn cmd_isready(&self) {
        send_response(&UciResponse::ReadyOk);
    }

    /// Handle "setoption" command
    fn cmd_setoption(&mut self, name: &str, value: Option<String>) {
        match name.to_lowercase().as_str() {
            "hash" => {
                if let Some(v) = value {
                    if let Ok(size) = v.parse::<usize>() {
                        self.options.hash_size = size.clamp(1, 1024);
                        self.engine.resize_tt(self.options.hash_size);
                    }
                }
            }
            "threads" => {
                if let Some(v) = value {
                    if let Ok(t) = v.parse::<usize>() {
                        self.options.threads = t.clamp(1, 1);
                    }
                }
            }
            "ownbook" => {
                self.options.own_book =
                    value.as_ref().map_or(false, |v| v.to_lowercase() == "true");
            }
            "bookfile" => {
                if let Some(path) = value {
                    if path.is_empty() {
                        self.options.book_path = None;
                        self.book = None;
                    } else {
                        self.options.book_path = Some(path.clone());
                        match OpeningBook::open(&path) {
                            Ok(book) => {
                                if self.options.debug {
                                    send_response(&UciResponse::Info(
                                        InfoResponse::new().with_string(format!(
                                            "Loaded opening book: {} ({} entries)",
                                            path,
                                            book.len()
                                        )),
                                    ));
                                }
                                self.book = Some(book);
                            }
                            Err(e) => {
                                if self.options.debug {
                                    send_response(&UciResponse::Info(
                                        InfoResponse::new()
                                            .with_string(format!("Failed to load book: {}", e)),
                                    ));
                                }
                                self.book = None;
                            }
                        }
                    }
                }
            }
            "bookbestmove" => {
                self.options.book_best_move =
                    value.as_ref().map_or(true, |v| v.to_lowercase() == "true");
            }
            _ => {}
        }
    }

    /// Handle "ucinewgame" command
    fn cmd_ucinewgame(&mut self) {
        self.board = Board::starting_position().expect("Failed to create starting position");
        self.engine.new_game();
    }

    /// Handle "position" command
    fn cmd_position(&mut self, fen: Option<String>, moves: Vec<String>) {
        // Set up the position
        if let Some(fen_str) = fen {
            match Board::from_fen(&fen_str) {
                Ok(board) => self.board = board,
                Err(e) => {
                    if self.options.debug {
                        send_response(&UciResponse::Info(
                            InfoResponse::new().with_string(format!("Invalid FEN: {}", e)),
                        ));
                    }
                    return;
                }
            }
        } else {
            // startpos
            self.board = Board::starting_position().expect("Failed to create starting position");
        }

        // Apply moves
        for move_str in moves {
            if let Some(mv) = self.parse_uci_move(&move_str) {
                if let Err(e) = self.board.make_move(&mv) {
                    if self.options.debug {
                        send_response(&UciResponse::Info(
                            InfoResponse::new()
                                .with_string(format!("Invalid move {}: {}", move_str, e)),
                        ));
                    }
                    return;
                }
            } else {
                if self.options.debug {
                    send_response(&UciResponse::Info(
                        InfoResponse::new().with_string(format!("Cannot parse move: {}", move_str)),
                    ));
                }
                return;
            }
        }
    }

    /// Parse a UCI move string (e.g., "e2e4", "e7e8q")
    fn parse_uci_move(&self, move_str: &str) -> Option<Move> {
        if move_str.len() < 4 {
            return None;
        }

        let from = Square::from_str(&move_str[0..2]).ok()?;
        let to = Square::from_str(&move_str[2..4]).ok()?;

        // Check for promotion
        let promotion = if move_str.len() > 4 {
            match move_str.chars().nth(4)? {
                'q' => Some(Piece::Queen),
                'r' => Some(Piece::Rook),
                'b' => Some(Piece::Bishop),
                'n' => Some(Piece::Knight),
                _ => None,
            }
        } else {
            None
        };

        // Generate legal moves and find matching one
        let legal_moves = self.engine.legal_moves(&self.board);

        legal_moves
            .into_iter()
            .find(|m| m.from == from && m.to == to && m.promotion == promotion)
    }

    /// Handle "go" command
    fn cmd_go(&mut self, params: SearchParams) {
        // 1. Try opening book first (if enabled)
        if let Some(book_move) = self.probe_book() {
            send_response(&UciResponse::BestMove {
                best: book_move,
                ponder: None,
            });
            return;
        }

        // 2. Standard search (existing code below)
        let is_white = self.board.side_to_move() == Color::White;
        let time_limit = params.calculate_move_time(is_white);
        let depth_limit = params.depth;

        // Perform search with callback for UCI info
        let result = self.engine.search(
            &mut self.board,
            depth_limit,
            time_limit,
            |info, best_move, score| {
                // Send UCI info for each completed depth
                if let Some(_mv) = best_move {
                    let pv: Vec<String> = info.pv.iter().map(|m| Self::move_to_uci(m)).collect();

                    send_response(&UciResponse::Info(
                        InfoResponse::new()
                            .with_depth(info.depth)
                            .with_score_cp(score)
                            .with_nodes(info.nodes)
                            .with_time(info.time_elapsed.as_millis() as u64)
                            .with_nps(info.nps)
                            .with_hashfull(info.hash_full)
                            .with_pv(pv),
                    ));
                }
            },
        );

        // Send best move
        let best_move_str = result
            .best_move
            .map(|m| Self::move_to_uci(&m))
            .unwrap_or_else(|| "0000".to_string());

        send_response(&UciResponse::BestMove {
            best: best_move_str,
            ponder: None,
        });
    }

    /// Convert a Move to UCI notation
    fn move_to_uci(mv: &Move) -> String {
        let mut s = format!("{}{}", mv.from, mv.to);
        if let Some(promo) = mv.promotion {
            s.push(match promo {
                Piece::Queen => 'q',
                Piece::Rook => 'r',
                Piece::Bishop => 'b',
                Piece::Knight => 'n',
                _ => 'q',
            });
        }
        s
    }

    /// Handle "stop" command
    fn cmd_stop(&mut self) {
        self.stop_flag.store(true, Ordering::SeqCst);
    }

    /// Handle "d" (display) command - non-standard debug command
    fn cmd_display(&self) {
        println!("{}", self.board.as_ascii());
        println!("Fen: {}", self.board.to_fen());
        println!("Zobrist: 0x{:016x}", self.board.zobrist_hash_raw());

        let legal_moves = self.engine.legal_moves(&self.board);
        println!("Legal moves: {}", legal_moves.len());
    }

    /// Handle "perft" command - count nodes at given depth
    fn cmd_perft(&mut self, depth: u8) {
        use std::time::Instant;

        let start = Instant::now();

        // Use perft_divide for detailed output
        let results = self.engine.perft_divide(&mut self.board, depth);

        let mut total = 0u64;
        for (mv, nodes) in &results {
            println!("{}: {}", Self::move_to_uci(mv), nodes);
            total += nodes;
        }

        let elapsed = start.elapsed();
        let nps = if elapsed.as_millis() > 0 {
            (total as u128 * 1000 / elapsed.as_millis()) as u64
        } else {
            0
        };

        println!();
        println!("Nodes: {}", total);
        println!("Time: {:?}", elapsed);
        println!("NPS: {}", nps);
    }

    /// Probe the opening book for a move in the current position
    /// Returns UCI move string if found and legal, None otherwise
    fn probe_book(&mut self) -> Option<String> {
        // Check if book is enabled and loaded
        if !self.options.own_book {
            return None;
        }

        let book = self.book.as_mut()?;

        // Use Polyglot-compatible hash (NOT the engine's internal Zobrist hash)
        let key = polyglot_hash(&self.board);

        // Select move based on configuration
        let poly_move = if self.options.book_best_move {
            book.select_move(key).ok()?
        } else {
            // Weighted random selection
            let random: f64 = rand::random();
            book.select_move_weighted(key, random).ok()?
        }?;

        // Convert Polyglot castling notation (e1h1 -> e1g1) to standard UCI
        let uci_str = convert_polyglot_castling(&poly_move.to_uci());

        // Validate the move is legal in the current position
        // This handles edge cases where book hash collides or book is outdated
        if self.parse_uci_move(&uci_str).is_some() {
            if self.options.debug {
                send_response(&UciResponse::Info(
                    InfoResponse::new().with_string(format!("Book move: {}", uci_str)),
                ));
            }
            Some(uci_str)
        } else {
            if self.options.debug {
                send_response(&UciResponse::Info(InfoResponse::new().with_string(
                    format!("Book move {} is illegal, searching...", uci_str),
                )));
            }
            None
        }
    }
}

impl Default for UciHandler {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use board::STARTING_POSITION_FEN;

    #[test]
    fn test_handler_creation() {
        let handler = UciHandler::new();
        assert_eq!(handler.info.name, "Aether");
    }

    #[test]
    fn test_parse_move() {
        let handler = UciHandler::new();
        let mv = handler.parse_uci_move("e2e4");
        assert!(mv.is_some());
        let mv = mv.unwrap();
        assert_eq!(mv.from, Square::E2);
        assert_eq!(mv.to, Square::E4);
    }

    #[test]
    fn test_position_startpos() {
        let mut handler = UciHandler::new();
        handler.cmd_position(None, vec![]);
        assert_eq!(handler.board.to_fen(), STARTING_POSITION_FEN);
    }

    #[test]
    fn test_position_with_moves() {
        let mut handler = UciHandler::new();
        handler.cmd_position(None, vec!["e2e4".to_string(), "e7e5".to_string()]);

        // After e4 e5, the position should reflect this
        let legal_moves = handler.engine.legal_moves(&handler.board);
        assert!(!legal_moves.is_empty());
    }

    #[test]
    fn test_perft_initial() {
        let mut handler = UciHandler::new();

        // Known perft values for starting position
        assert_eq!(handler.engine.perft(&mut handler.board, 1), 20);
        assert_eq!(handler.engine.perft(&mut handler.board, 2), 400);
        assert_eq!(handler.engine.perft(&mut handler.board, 3), 8902);
    }
}
