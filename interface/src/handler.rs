//! UCI Handler - Main loop that connects UCI protocol with the chess engine

use crate::uci::{
    EngineInfo, InfoResponse, OptionInfo, OptionType, SearchParams, UciCommand, UciInput,
    UciResponse, send_response, send_responses,
};
use aether_core::{Color, Move, Piece, score_to_mate_moves};
use board::Board;
use engine::Engine;
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
}

impl Default for EngineOptions {
    fn default() -> Self {
        Self {
            hash_size: 16,
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
    /// Stop flag for search
    stop_flag: Arc<AtomicBool>,
}

impl UciHandler {
    /// Create a new UCI handler
    pub fn new() -> Self {
        let engine = Engine::new(16);
        let stop_flag = engine.stop_flag();

        Self {
            info: EngineInfo::default(),
            board: Board::starting_position().expect("Failed to create starting position"),
            engine,
            options: EngineOptions::default(),
            stop_flag,
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
    }

    fn parse_uci_move(&self, move_str: &str) -> Option<Move> {
        let parsed = Move::from_str(move_str).ok()?;

        // Generate legal moves and find matching one
        let legal_moves = self.engine.legal_moves(&self.board);

        legal_moves
            .into_iter()
            .find(|m| m.from == parsed.from && m.to == parsed.to && m.promotion == parsed.promotion)
    }

    fn cmd_go(&mut self, params: SearchParams) {
        let is_white = self.board.side_to_move() == Color::White;
        let time_limit = params.calculate_move_time(is_white);
        let hard_limit = params.calculate_hard_limit(is_white);
        let depth_limit = params.depth;
        let nodes_limit = params.nodes;
        let infinite = params.infinite;

        // Perform search with callback for UCI info
        let result = self.engine.search(
            &mut self.board,
            depth_limit,
            time_limit,
            hard_limit,
            nodes_limit,
            infinite,
            |info, best_move, score| {
                // Send UCI info for each completed depth
                if let Some(_mv) = best_move {
                    let pv: Vec<String> = info.pv.iter().map(|m| Self::move_to_uci(m)).collect();

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

    fn add_score_to_info(info: InfoResponse, score: i32) -> InfoResponse {
        match score_to_mate_moves(score) {
            Some(mate_moves) => info.with_score_mate(mate_moves),
            None => info.with_score_cp(score),
        }
    }

    fn cmd_stop(&mut self) {
        self.stop_flag.store(true, Ordering::SeqCst);
    }

    fn cmd_display(&self) {
        println!("{}", self.board.as_ascii());
        println!("Fen: {}", self.board);
        println!("Zobrist: 0x{:016x}", self.board.zobrist_hash_raw());

        let legal_moves = self.engine.legal_moves(&self.board);
        println!("Legal moves: {}", legal_moves.len());
    }

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
        assert_eq!(mv.from, Square::E2);
        assert_eq!(mv.to, Square::E4);
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

    #[test]
    fn test_perft_initial() {
        let mut handler = UciHandler::new();

        // Known perft values for starting position
        assert_eq!(handler.engine.perft(&mut handler.board, 1), 20);
        assert_eq!(handler.engine.perft(&mut handler.board, 2), 400);
        assert_eq!(handler.engine.perft(&mut handler.board, 3), 8902);
    }
}
