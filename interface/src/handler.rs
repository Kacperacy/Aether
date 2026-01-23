//! UCI Handler - Main loop that connects UCI protocol with the chess engine

use crate::uci::{
    EngineInfo, InfoResponse, OptionInfo, OptionType, SearchParams, UciCommand, UciInput,
    UciResponse, send_response, send_responses,
};
use aether_core::{Color, Move, Piece, score_to_mate_moves};
use board::Board;
use engine::search::SearcherType;
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
    /// Search algorithm
    pub algorithm: SearcherType,
}

impl Default for EngineOptions {
    fn default() -> Self {
        Self {
            hash_size: 16,
            threads: 1,
            debug: false,
            algorithm: SearcherType::default(),
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
    /// Create a new UCI handler with default settings
    pub fn new() -> Self {
        Self::with_engine(Engine::new(16))
    }

    /// Create a new UCI handler with a custom engine
    pub fn with_engine(engine: Engine) -> Self {
        let stop_flag = engine.stop_flag();
        let algorithm = engine.searcher_type();

        Self {
            info: EngineInfo::default(),
            board: Board::starting_position().expect("Failed to create starting position"),
            engine,
            options: EngineOptions {
                algorithm,
                ..EngineOptions::default()
            },
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
            UciCommand::Bench(depth) => self.cmd_bench(depth),
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

        send_response(&UciResponse::Option(OptionInfo {
            name: "Algorithm".to_string(),
            option_type: OptionType::Combo {
                default: "FullAlphaBeta".to_string(),
                options: vec![
                    "PureAlphaBeta".to_string(),
                    "FullAlphaBeta".to_string(),
                    "Mtdf".to_string(),
                    "NegaScout".to_string(),
                    "MCTS".to_string(),
                ],
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
            "algorithm" => {
                if let Some(v) = value {
                    if let Ok(algo) = v.parse::<SearcherType>() {
                        self.options.algorithm = algo;
                        self.engine.set_searcher_type(algo);
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

    fn cmd_bench(&mut self, depth: Option<u8>) {
        use std::time::{Duration, Instant};

        // Standard benchmark positions (16 positions)
        const BENCH_POSITIONS: &[&str] = &[
            // Starting position
            "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1",
            // Kiwipete - complex tactical position
            "r3k2r/p1ppqpb1/bn2pnp1/3PN3/1p2P3/2N2Q1p/PPPBBPPP/R3K2R w KQkq - 0 1",
            // Endgame position
            "8/2p5/3p4/KP5r/1R3p1k/8/4P1P1/8 w - - 0 1",
            // Complex position with promotions
            "r3k2r/Pppp1ppp/1b3nbN/nP6/BBP1P3/q4N2/Pp1P2PP/R2Q1RK1 w kq - 0 1",
            // Another promotion test
            "rnbq1k1r/pp1Pbppp/2p5/8/2B5/8/PPP1NnPP/RNBQK2R w KQ - 1 8",
            // Position from a real game
            "r4rk1/1pp1qppp/p1np1n2/2b1p1B1/2B1P1b1/P1NP1N2/1PP1QPPP/R4RK1 w - - 0 10",
            // Middlegame with tension
            "r1bqkb1r/pppp1ppp/2n2n2/4p3/2B1P3/5N2/PPPP1PPP/RNBQK2R w KQkq - 4 4",
            // Rook endgame
            "8/8/3p4/1p1Pp2p/pP2Pp1P/P4P1K/8/4k3 w - - 0 1",
            // Queen vs pieces
            "r2q1rk1/ppp2ppp/2n1bn2/2b1p3/3pP3/3P1NPP/PPP1NPB1/R1BQ1RK1 b - - 0 9",
            // Closed position
            "r1bq1rk1/pp3ppp/2nbpn2/3p4/3P4/1PN1PN2/PBP2PPP/R2QKB1R w KQ - 0 8",
            // Open position
            "r1bqr1k1/pp1n1pbp/2p2np1/3p4/3P4/2N1PN2/PP2BPPP/R1BQ1RK1 w - - 0 10",
            // Tactical position - pins
            "r2qk2r/pb1nbppp/1pp1pn2/3p4/2PP4/1PN1PN2/PB2BPPP/R2QK2R w KQkq - 0 9",
            // Knight outpost
            "r1bq1rk1/ppp2ppp/2np1n2/4p1B1/1bB1P3/2NP1N2/PPP2PPP/R2QK2R w KQ - 0 7",
            // Pawn structure
            "r2qkb1r/pp2pppp/2n2n2/3p4/3P4/2N2N2/PPP2PPP/R1BQKB1R w KQkq - 0 6",
            // Complex tactics
            "r1b1k2r/ppppnppp/2n2q2/2b5/3NP3/2P1B3/PP3PPP/RN1QKB1R w KQkq - 0 7",
            // Endgame with passed pawn
            "8/5pk1/5p2/3P4/1p5p/1P3P1P/5KP1/8 w - - 0 1",
        ];

        let depth = depth.unwrap_or(12);
        let mut total_nodes = 0u64;
        let mut total_time = Duration::ZERO;
        let start = Instant::now();

        println!("Running benchmark at depth {}...", depth);
        println!();

        for (i, fen) in BENCH_POSITIONS.iter().enumerate() {
            let mut board: Board = fen.parse().expect("Invalid benchmark FEN");
            self.engine.new_game();

            let pos_start = Instant::now();
            let result = self.engine.search(
                &mut board,
                Some(depth),
                None,
                None,
                None,
                false,
                |_, _, _| {},
            );

            let pos_time = pos_start.elapsed();
            total_nodes += result.info.nodes;
            total_time += pos_time;

            let best_move = result
                .best_move
                .map(|m| Self::move_to_uci(&m))
                .unwrap_or_else(|| "none".to_string());

            println!(
                "Position {:2}: {} nodes, {:?}, best: {}",
                i + 1,
                result.info.nodes,
                pos_time,
                best_move
            );
        }

        let elapsed = start.elapsed();
        let nps = if elapsed.as_millis() > 0 {
            total_nodes * 1000 / elapsed.as_millis() as u64
        } else {
            0
        };

        println!();
        println!("{} nodes {:.3}s nps {}", total_nodes, elapsed.as_secs_f64(), nps);
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
