//! UCI Handler - Main loop that connects UCI protocol with the chess engine

use crate::uci::{
    EngineInfo, InfoResponse, OptionInfo, OptionType, SearchParams, UciCommand, UciInput,
    UciResponse, send_response, send_responses,
};
use aether_core::{Color, Move, Piece, Square};
use board::{Board, BoardOps, BoardQuery, FenOps};
use movegen::{Generator, MoveGen};
use std::str::FromStr;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::time::{Duration, Instant};

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
    /// Move generator
    generator: Generator,
    /// Engine options
    options: EngineOptions,
    /// Stop flag for search
    stop_flag: Arc<AtomicBool>,
    /// Is engine currently searching
    searching: bool,
}

impl UciHandler {
    /// Create a new UCI handler
    pub fn new() -> Self {
        Self {
            info: EngineInfo::default(),
            board: Board::starting_position().expect("Failed to create starting position"),
            generator: Generator::new(),
            options: EngineOptions::default(),
            stop_flag: Arc::new(AtomicBool::new(false)),
            searching: false,
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
            UciCommand::Register => {} // Not implemented
            UciCommand::UciNewGame => self.cmd_ucinewgame(),
            UciCommand::Position { fen, moves } => self.cmd_position(fen, moves),
            UciCommand::Go(params) => self.cmd_go(params),
            UciCommand::Stop => self.cmd_stop(),
            UciCommand::PonderHit => {} // Not implemented
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

    /// Handle "ucinewgame" command
    fn cmd_ucinewgame(&mut self) {
        self.board = Board::starting_position().expect("Failed to create starting position");
        // TODO: Clear hash tables when implemented
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
        let mut legal_moves = Vec::new();
        self.generator.legal(&self.board, &mut legal_moves);

        legal_moves
            .into_iter()
            .find(|m| m.from == from && m.to == to && m.promotion == promotion)
    }

    /// Handle "go" command
    fn cmd_go(&mut self, params: SearchParams) {
        self.stop_flag.store(false, Ordering::SeqCst);
        self.searching = true;

        let is_white = self.board.side_to_move() == Color::White;
        let time_limit = params.calculate_move_time(is_white);
        let depth_limit = params.depth.unwrap_or(64);

        // Simple iterative deepening search
        let start_time = Instant::now();
        let mut best_move: Option<Move> = None;
        let mut best_score = i32::MIN;

        let mut legal_moves = Vec::new();
        self.generator.legal(&self.board, &mut legal_moves);

        if legal_moves.is_empty() {
            send_response(&UciResponse::BestMove {
                best: "0000".to_string(),
                ponder: None,
            });
            self.searching = false;
            return;
        }

        // If only one legal move, play it immediately
        if legal_moves.len() == 1 {
            let mv = &legal_moves[0];
            send_response(&UciResponse::BestMove {
                best: self.move_to_uci(mv),
                ponder: None,
            });
            self.searching = false;
            return;
        }

        for depth in 1..=depth_limit {
            if self.stop_flag.load(Ordering::SeqCst) {
                break;
            }

            if let Some(limit) = time_limit {
                if start_time.elapsed() >= limit {
                    break;
                }
            }

            let mut nodes = 0u64;
            let mut current_best: Option<Move> = None;
            let mut current_score = i32::MIN;

            for mv in &legal_moves {
                if self.stop_flag.load(Ordering::SeqCst) {
                    break;
                }

                self.board.make_move(mv).ok();
                let score = -self.alpha_beta(
                    depth - 1,
                    i32::MIN + 1,
                    i32::MAX,
                    &mut nodes,
                    &start_time,
                    time_limit,
                );
                self.board.unmake_move(mv).ok();

                if score > current_score {
                    current_score = score;
                    current_best = Some(*mv);
                }
            }

            if let Some(mv) = current_best {
                best_move = Some(mv);
                best_score = current_score;

                // Send info about this depth
                let elapsed = start_time.elapsed();
                let nps = if elapsed.as_millis() > 0 {
                    (nodes as u128 * 1000 / elapsed.as_millis()) as u64
                } else {
                    0
                };

                send_response(&UciResponse::Info(
                    InfoResponse::new()
                        .with_depth(depth)
                        .with_score_cp(best_score)
                        .with_nodes(nodes)
                        .with_time(elapsed.as_millis() as u64)
                        .with_nps(nps)
                        .with_pv(vec![self.move_to_uci(&mv)]),
                ));
            }
        }

        // Send best move
        let best_move_str = best_move
            .map(|m| self.move_to_uci(&m))
            .unwrap_or_else(|| self.move_to_uci(&legal_moves[0]));

        send_response(&UciResponse::BestMove {
            best: best_move_str,
            ponder: None,
        });

        self.searching = false;
    }

    /// Simple alpha-beta search
    fn alpha_beta(
        &mut self,
        depth: u8,
        mut alpha: i32,
        beta: i32,
        nodes: &mut u64,
        start_time: &Instant,
        time_limit: Option<Duration>,
    ) -> i32 {
        *nodes += 1;

        // Time check every 1024 nodes
        if *nodes % 1024 == 0 {
            if self.stop_flag.load(Ordering::SeqCst) {
                return 0;
            }
            if let Some(limit) = time_limit {
                if start_time.elapsed() >= limit {
                    self.stop_flag.store(true, Ordering::SeqCst);
                    return 0;
                }
            }
        }

        if depth == 0 {
            return self.evaluate();
        }

        let mut legal_moves = Vec::new();
        self.generator.legal(&self.board, &mut legal_moves);

        if legal_moves.is_empty() {
            // Checkmate or stalemate
            if self.board.is_in_check(self.board.side_to_move()) {
                return -100000 + (64 - depth as i32); // Prefer faster mates
            } else {
                return 0; // Stalemate
            }
        }

        for mv in legal_moves {
            self.board.make_move(&mv).ok();
            let score = -self.alpha_beta(depth - 1, -beta, -alpha, nodes, start_time, time_limit);
            self.board.unmake_move(&mv).ok();

            if score >= beta {
                return beta;
            }
            if score > alpha {
                alpha = score;
            }
        }

        alpha
    }

    /// Simple material evaluation
    fn evaluate(&self) -> i32 {
        let mut score = 0i32;

        let piece_values = [100, 320, 330, 500, 900, 20000]; // P, N, B, R, Q, K

        for color in [Color::White, Color::Black] {
            let sign = if color == Color::White { 1 } else { -1 };

            for (i, &value) in piece_values.iter().enumerate() {
                let piece = match i {
                    0 => Piece::Pawn,
                    1 => Piece::Knight,
                    2 => Piece::Bishop,
                    3 => Piece::Rook,
                    4 => Piece::Queen,
                    5 => Piece::King,
                    _ => continue,
                };

                let count = self.board.piece_count(piece, color) as i32;
                score += sign * value * count;
            }
        }

        // Return from side to move's perspective
        if self.board.side_to_move() == Color::White {
            score
        } else {
            -score
        }
    }

    /// Convert a Move to UCI notation
    fn move_to_uci(&self, mv: &Move) -> String {
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

        let mut legal_moves = Vec::new();
        self.generator.legal(&self.board, &mut legal_moves);
        println!("Legal moves: {}", legal_moves.len());
    }

    /// Handle "perft" command - count nodes at given depth
    fn cmd_perft(&mut self, depth: u8) {
        let start = Instant::now();
        let nodes = self.perft(depth);
        let elapsed = start.elapsed();

        let nps = if elapsed.as_millis() > 0 {
            (nodes as u128 * 1000 / elapsed.as_millis()) as u64
        } else {
            0
        };

        println!();
        println!("Nodes: {}", nodes);
        println!("Time: {:?}", elapsed);
        println!("NPS: {}", nps);
    }

    /// Perft - count nodes at given depth
    fn perft(&mut self, depth: u8) -> u64 {
        if depth == 0 {
            return 1;
        }

        let mut legal_moves = Vec::new();
        self.generator.legal(&self.board, &mut legal_moves);

        if depth == 1 {
            return legal_moves.len() as u64;
        }

        let mut nodes = 0u64;

        for mv in legal_moves {
            self.board.make_move(&mv).ok();
            let child_nodes = self.perft(depth - 1);
            self.board.unmake_move(&mv).ok();

            if depth >= 2 {
                println!("{}: {}", self.move_to_uci(&mv), child_nodes);
            }
            nodes += child_nodes;
        }

        nodes
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

        // After e4 e5, the FEN should reflect this position
        let fen = handler.board.to_fen();
        assert!(fen.contains("e6")); // En passant square after e4
    }

    #[test]
    fn test_perft_initial() {
        let mut handler = UciHandler::new();

        // Known perft values for starting position
        assert_eq!(handler.perft(1), 20);
        assert_eq!(handler.perft(2), 400);
        assert_eq!(handler.perft(3), 8902);
    }
}
