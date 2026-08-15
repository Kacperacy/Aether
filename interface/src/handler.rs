//! UCI Handler - Main loop that connects UCI protocol with the chess engine

use crate::uci::{
    EngineInfo, InfoResponse, OptionInfo, OptionType, SearchParams, UciCommand, UciInput,
    UciResponse, send_response, send_responses,
};
use aether_core::{Color, Move};
use board::Board;
use engine::eval::score_to_mate_moves;
use engine::search::SearchLimits;
use engine::{DEFAULT_HASH_MB, Engine, MAX_HASH_MB, MIN_HASH_MB};

/// Depth used for a `go` with no depth, node or time bound (and no `infinite`).
const DEFAULT_GO_DEPTH: u8 = 8;

/// Engine options
#[derive(Debug, Clone)]
pub struct EngineOptions {
    /// Hash table size in MB
    pub hash_size: usize,
    /// Number of threads (for future multi-threading)
    pub threads: usize,
    /// Whether to show debug output
    pub debug: bool,
}

impl Default for EngineOptions {
    fn default() -> Self {
        Self {
            hash_size: DEFAULT_HASH_MB,
            threads: 1,
            debug: false,
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
}

impl UciHandler {
    /// Create a new UCI handler
    pub fn new() -> Self {
        let engine = Engine::new(DEFAULT_HASH_MB);

        Self {
            info: EngineInfo::default(),
            board: Board::starting_position().expect("Failed to create starting position"),
            engine,
            options: EngineOptions::default(),
        }
    }

    /// Run the main UCI loop
    pub fn run(&mut self) {
        let mut input = UciInput::new();

        while let Some(cmd) = input.read_command() {
            match cmd {
                UciCommand::Quit => break,
                _ => self.handle_command(cmd),
            }
        }
    }

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
            UciCommand::Bench(depth) => Self::cmd_bench(depth),
            UciCommand::Unknown(s) => {
                if self.options.debug {
                    send_response(&UciResponse::Info(
                        InfoResponse::new().with_string(format!("Unknown command: {}", s)),
                    ));
                }
            }
        }
    }

    fn cmd_uci(&self) {
        send_responses(&[
            UciResponse::IdName(self.info.name.clone()),
            UciResponse::IdAuthor(self.info.author.clone()),
        ]);

        // Send available options
        send_response(&UciResponse::Option(OptionInfo {
            name: "Hash".to_string(),
            option_type: OptionType::Spin {
                default: DEFAULT_HASH_MB as i64,
                min: MIN_HASH_MB as i64,
                max: MAX_HASH_MB as i64,
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

        // Hidden tuning knobs. Present only in a `tune` build, so the shipping
        // engine advertises exactly the options it advertised before.
        #[cfg(feature = "tune")]
        for t in engine::params::TUNABLES {
            send_response(&UciResponse::Option(OptionInfo {
                name: t.name.to_string(),
                option_type: OptionType::Spin {
                    default: t.default as i64,
                    min: t.min as i64,
                    max: t.max as i64,
                },
            }));
        }

        send_response(&UciResponse::UciOk);
    }

    fn cmd_debug(&mut self, on: bool) {
        self.options.debug = on;
    }

    fn cmd_isready(&self) {
        send_response(&UciResponse::ReadyOk);
    }

    fn cmd_setoption(&mut self, name: &str, value: Option<String>) {
        match name.to_lowercase().as_str() {
            "hash" => {
                if let Some(v) = value
                    && let Ok(size) = v.parse::<usize>()
                {
                    self.options.hash_size = size.clamp(MIN_HASH_MB, MAX_HASH_MB);
                    self.engine.resize_tt(self.options.hash_size);
                }
            }
            "threads" => {
                if let Some(v) = value
                    && let Ok(t) = v.parse::<usize>()
                {
                    self.options.threads = t.clamp(1, 1);
                }
            }
            // A tuner drives the search parameters through this path. The name
            // is matched case-insensitively and the value clamped to the
            // parameter's declared range by `set_by_name`.
            #[cfg(feature = "tune")]
            other => {
                if let Some(v) = value
                    && let Ok(parsed) = v.parse::<i32>()
                {
                    engine::params::set_by_name(other, parsed);
                }
            }
            #[cfg(not(feature = "tune"))]
            _ => {}
        }
    }

    fn cmd_ucinewgame(&mut self) {
        self.board = Board::starting_position().expect("Failed to create starting position");
        self.engine.new_game();
    }

    fn cmd_position(&mut self, fen: Option<String>, moves: Vec<String>) {
        // Set up the position
        if let Some(fen_str) = fen {
            match fen_str.parse::<Board>() {
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

        // These moves are part of the game, not of a search — they will never be
        // unmade, so they must not occupy the unmake stack.
        self.board.commit_history();
    }

    fn parse_uci_move(&self, move_str: &str) -> Option<Move> {
        movegen::parse_uci_move(&self.board, move_str)
    }

    fn cmd_go(&mut self, params: SearchParams) {
        let is_white = self.board.side_to_move() == Color::White;

        // Every limit the GUI supplied applies at once; the search stops at
        // whichever fires first. `infinite` simply means no limits at all.
        let mut limits = SearchLimits {
            depth: params.depth,
            nodes: params.nodes,
            time: params.calculate_move_time(is_white),
            hard_time: params.calculate_hard_limit(is_white),
        };

        // A bare `go` carries no bounds at all. Only honour that as "search
        // forever" when the GUI actually asked for `go infinite` — otherwise fall
        // back to a fixed depth, because the search runs on this thread and
        // cannot observe a `stop` command until it returns.
        if limits.is_unbounded() && !params.infinite {
            limits.depth = Some(DEFAULT_GO_DEPTH);
        }

        // Perform search with callback for UCI info
        let result = self
            .engine
            .search(&mut self.board, &limits, |info, best_move, score| {
                // Send UCI info for each completed depth
                if let Some(_mv) = best_move {
                    let pv: Vec<String> = info.pv.iter().map(Self::move_to_uci).collect();

                    let mut response = InfoResponse::new()
                        .with_depth(info.depth)
                        .with_seldepth(info.selective_depth)
                        .with_nodes(info.nodes)
                        .with_time(info.time_elapsed.as_millis() as u64)
                        .with_nps(info.nps)
                        .with_hashfull(info.hash_full)
                        .with_pv(pv);

                    // Handle mate scores vs centipawn scores
                    response = Self::add_score_to_info(response, score);

                    send_response(&UciResponse::Info(response));
                }
            });

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

    fn move_to_uci(mv: &Move) -> String {
        mv.to_string()
    }

    fn add_score_to_info(info: InfoResponse, score: i32) -> InfoResponse {
        match score_to_mate_moves(score) {
            Some(mate_moves) => info.with_score_mate(mate_moves),
            None => info.with_score_cp(score),
        }
    }

    fn cmd_stop(&mut self) {
        self.engine.stop();
    }

    fn cmd_display(&self) {
        println!("{}", self.board.as_ascii());
        println!("Fen: {}", self.board);
        println!("Zobrist: 0x{:016x}", self.board.zobrist_hash_raw());

        let legal_moves = self.engine.legal_moves(&self.board);
        println!("Legal moves: {}", legal_moves.len());
    }

    /// Fixed-workload benchmark. Prints the node count that serves as the
    /// project's search-regression signal.
    fn cmd_bench(depth: Option<u8>) {
        let depth = depth.unwrap_or(engine::bench::DEFAULT_BENCH_DEPTH);
        let result = engine::bench::run(depth);

        println!();
        println!("Positions: {}", result.positions);
        println!("Depth: {}", result.depth);
        println!("Nodes: {}", result.nodes);
        println!("Time: {:?}", result.elapsed);
        println!("NPS: {}", result.nps());
    }

    fn cmd_perft(&mut self, depth: u8) {
        let report = movegen::perft_report(&mut self.board, u32::from(depth));

        for (mv, nodes) in &report.moves {
            println!("{}: {}", Self::move_to_uci(mv), nodes);
        }

        println!();
        println!("Nodes: {}", report.nodes);
        println!("Time: {:?}", report.elapsed);
        println!("NPS: {}", report.nps());
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
    use aether_core::Square;
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
        assert_eq!(mv.from_sq(), Square::E2);
        assert_eq!(mv.to_sq(), Square::E4);
    }

    #[test]
    fn test_position_startpos() {
        let mut handler = UciHandler::new();
        handler.cmd_position(None, vec![]);
        assert_eq!(handler.board.to_string(), STARTING_POSITION_FEN);
    }

    #[test]
    fn test_position_with_moves() {
        let mut handler = UciHandler::new();
        handler.cmd_position(None, vec!["e2e4".to_string(), "e7e5".to_string()]);

        // After e4 e5, the position should reflect this
        let legal_moves = handler.engine.legal_moves(&handler.board);
        assert!(!legal_moves.is_empty());
    }
}
