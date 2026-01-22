use std::fmt;
use std::fmt::{Display, Formatter};
use std::io::{self, BufRead, Write};
use std::time::Duration;

pub struct EngineInfo {
    pub name: String,
    pub author: String,
}

impl Default for EngineInfo {
    fn default() -> Self {
        Self {
            name: "Aether".to_string(),
            author: "Kacper Maciołek".to_string(),
        }
    }
}

#[derive(Debug, Clone, Default)]
pub struct SearchParams {
    pub searchmoves: Vec<String>,
    pub ponder: bool,
    pub wtime: Option<u64>,
    pub btime: Option<u64>,
    pub winc: Option<u64>,
    pub binc: Option<u64>,
    pub movestogo: Option<u32>,
    pub depth: Option<u8>,
    pub nodes: Option<u64>,
    pub mate: Option<u32>,
    pub movetime: Option<u64>,
    pub infinite: bool,
}

impl SearchParams {
    pub fn calculate_move_time(&self, is_white: bool) -> Option<Duration> {
        if self.infinite {
            return None;
        }

        if let Some(movetime) = self.movetime {
            return Some(Duration::from_millis(movetime.saturating_sub(50).max(10)));
        }

        let time = if is_white { self.wtime } else { self.btime };
        let inc = if is_white { self.winc } else { self.binc };

        if let Some(time_left) = time {
            let increment = inc.unwrap_or(0);
            let moves_to_go = self.movestogo.unwrap_or(30) as u64;
            let base_time = time_left / moves_to_go.max(1);
            let inc_bonus = (increment * 7) / 10;
            let target_time = base_time + inc_bonus;

            let max_fraction = time_left / 10;
            let min_remaining = if time_left > 1000 { 100 } else { 50 };
            let safety_buffer = time_left.saturating_sub(min_remaining);
            let final_time = target_time.min(max_fraction).min(safety_buffer).max(5);

            Some(Duration::from_millis(final_time))
        } else {
            None
        }
    }

    pub fn calculate_hard_limit(&self, is_white: bool) -> Option<Duration> {
        if self.infinite {
            return None;
        }

        if let Some(movetime) = self.movetime {
            return Some(Duration::from_millis(movetime.saturating_sub(10).max(5)));
        }

        let time = if is_white { self.wtime } else { self.btime };

        if let Some(time_left) = time {
            let hard_limit = (time_left / 4).saturating_sub(30).max(5);
            Some(Duration::from_millis(hard_limit))
        } else {
            None
        }
    }
}

#[derive(Debug, Clone)]
pub enum UciCommand {
    Uci,
    Debug(bool),
    IsReady,
    SetOption {
        name: String,
        value: Option<String>,
    },
    Register,
    UciNewGame,
    Position {
        fen: Option<String>,
        moves: Vec<String>,
    },
    Go(SearchParams),
    Stop,
    PonderHit,
    Quit,
    Display,
    Perft(u8),
    Unknown(String),
}

pub fn parse_command(input: &str) -> UciCommand {
    let input = input.trim();
    let mut parts = input.split_whitespace();

    match parts.next() {
        Some("uci") => UciCommand::Uci,
        Some("debug") => {
            let on = parts.next() == Some("on");
            UciCommand::Debug(on)
        }
        Some("isready") => UciCommand::IsReady,
        Some("setoption") => parse_setoption(&mut parts),
        Some("register") => UciCommand::Register,
        Some("ucinewgame") => UciCommand::UciNewGame,
        Some("position") => parse_position(&mut parts),
        Some("go") => parse_go(&mut parts),
        Some("stop") => UciCommand::Stop,
        Some("ponderhit") => UciCommand::PonderHit,
        Some("quit") => UciCommand::Quit,
        Some("d") => UciCommand::Display,
        Some("perft") => {
            let depth = parts.next().and_then(|s| s.parse().ok()).unwrap_or(1);
            UciCommand::Perft(depth)
        }
        _ => UciCommand::Unknown(input.to_string()),
    }
}

fn parse_setoption<'a>(parts: &mut impl Iterator<Item = &'a str>) -> UciCommand {
    let mut name = String::new();
    let mut value = None;
    let mut in_name = false;
    let mut in_value = false;

    for part in parts {
        match part {
            "name" => {
                in_name = true;
                in_value = false;
            }
            "value" => {
                in_name = false;
                in_value = true;
            }
            _ => {
                if in_name {
                    if !name.is_empty() {
                        name.push(' ');
                    }
                    name.push_str(part);
                } else if in_value {
                    let v = value.get_or_insert_with(String::new);
                    if !v.is_empty() {
                        v.push(' ');
                    }
                    v.push_str(part);
                }
            }
        }
    }

    UciCommand::SetOption { name, value }
}

fn parse_position<'a>(parts: &mut impl Iterator<Item = &'a str>) -> UciCommand {
    let mut fen = None;
    let mut moves = Vec::new();
    let mut in_moves = false;
    let mut fen_parts = Vec::new();

    for part in parts {
        match part {
            "startpos" => {
                fen = None;
            }
            "fen" => {
                // FEN will be collected in subsequent parts
            }
            "moves" => {
                if !fen_parts.is_empty() {
                    fen = Some(fen_parts.join(" "));
                }
                in_moves = true;
            }
            _ => {
                if in_moves {
                    moves.push(part.to_string());
                } else {
                    fen_parts.push(part);
                }
            }
        }
    }

    // If we never hit "moves", the FEN parts might still be there
    if !in_moves && !fen_parts.is_empty() {
        fen = Some(fen_parts.join(" "));
    }

    UciCommand::Position { fen, moves }
}

fn parse_go<'a>(parts: &mut impl Iterator<Item = &'a str>) -> UciCommand {
    let mut params = SearchParams::default();
    let mut current_key: Option<&str> = None;

    for part in parts {
        match part {
            "searchmoves" | "wtime" | "btime" | "winc" | "binc" | "movestogo" | "depth"
            | "nodes" | "mate" | "movetime" => {
                current_key = Some(part);
            }
            "ponder" => {
                params.ponder = true;
                current_key = None;
            }
            "infinite" => {
                params.infinite = true;
                current_key = None;
            }
            value => {
                if let Some(key) = current_key {
                    match key {
                        "searchmoves" => params.searchmoves.push(value.to_string()),
                        "wtime" => params.wtime = value.parse().ok(),
                        "btime" => params.btime = value.parse().ok(),
                        "winc" => params.winc = value.parse().ok(),
                        "binc" => params.binc = value.parse().ok(),
                        "movestogo" => params.movestogo = value.parse().ok(),
                        "depth" => params.depth = value.parse().ok(),
                        "nodes" => params.nodes = value.parse().ok(),
                        "mate" => params.mate = value.parse().ok(),
                        "movetime" => params.movetime = value.parse().ok(),
                        _ => {}
                    }
                    if key != "searchmoves" {
                        current_key = None;
                    }
                }
            }
        }
    }

    UciCommand::Go(params)
}

#[derive(Debug, Clone)]
pub enum UciResponse {
    IdName(String),
    IdAuthor(String),
    UciOk,
    ReadyOk,
    BestMove {
        best: String,
        ponder: Option<String>,
    },
    CopyProtection(String),
    Registration(String),
    Info(InfoResponse),
    Option(OptionInfo),
}

#[derive(Debug, Clone, Default)]
pub struct InfoResponse {
    pub depth: Option<u8>,
    pub seldepth: Option<u8>,
    pub time: Option<u64>,
    pub nodes: Option<u64>,
    pub pv: Vec<String>,
    pub multipv: Option<u8>,
    pub score_cp: Option<i32>,
    pub score_mate: Option<i32>,
    pub currmove: Option<String>,
    pub currmovenumber: Option<u32>,
    pub hashfull: Option<u16>,
    pub nps: Option<u64>,
    pub tbhits: Option<u64>,
    pub string: Option<String>,
}

impl InfoResponse {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_depth(mut self, depth: u8) -> Self {
        self.depth = Some(depth);
        self
    }

    pub fn with_seldepth(mut self, seldepth: u8) -> Self {
        self.seldepth = Some(seldepth);
        self
    }

    pub fn with_score_cp(mut self, score: i32) -> Self {
        self.score_cp = Some(score);
        self
    }

    pub fn with_score_mate(mut self, mate: i32) -> Self {
        self.score_mate = Some(mate);
        self
    }

    pub fn with_nodes(mut self, nodes: u64) -> Self {
        self.nodes = Some(nodes);
        self
    }

    pub fn with_time(mut self, time_ms: u64) -> Self {
        self.time = Some(time_ms);
        self
    }

    pub fn with_pv(mut self, pv: Vec<String>) -> Self {
        self.pv = pv;
        self
    }

    pub fn with_nps(mut self, nps: u64) -> Self {
        self.nps = Some(nps);
        self
    }

    pub fn with_hashfull(mut self, hashfull: u16) -> Self {
        self.hashfull = Some(hashfull);
        self
    }

    pub fn with_string(mut self, s: String) -> Self {
        self.string = Some(s);
        self
    }
}

#[derive(Debug, Clone)]
pub struct OptionInfo {
    pub name: String,
    pub option_type: OptionType,
}

impl Display for OptionInfo {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "option name {} type ", self.name)?;
        match &self.option_type {
            OptionType::Check { default } => {
                write!(f, "check default {}", default)
            }
            OptionType::Spin { default, min, max } => {
                write!(f, "spin default {} min {} max {}", default, min, max)
            }
            OptionType::Combo { default, options } => {
                write!(f, "combo default {}", default)?;
                for opt in options {
                    write!(f, " var {}", opt)?;
                }
                Ok(())
            }
            OptionType::Button => write!(f, "button"),
            OptionType::String { default } => {
                write!(f, "string default {}", default)
            }
        }
    }
}

#[derive(Debug, Clone)]
pub enum OptionType {
    Check {
        default: bool,
    },
    Spin {
        default: i64,
        min: i64,
        max: i64,
    },
    Combo {
        default: String,
        options: Vec<String>,
    },
    Button,
    String {
        default: String,
    },
}

impl Display for UciResponse {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            UciResponse::IdName(name) => write!(f, "id name {}", name),
            UciResponse::IdAuthor(author) => write!(f, "id author {}", author),
            UciResponse::UciOk => write!(f, "uciok"),
            UciResponse::ReadyOk => write!(f, "readyok"),
            UciResponse::BestMove { best, ponder } => {
                write!(f, "bestmove {}", best)?;
                if let Some(p) = ponder {
                    write!(f, " ponder {}", p)?;
                }
                Ok(())
            }
            UciResponse::CopyProtection(status) => write!(f, "copyprotection {}", status),
            UciResponse::Registration(status) => write!(f, "registration {}", status),
            UciResponse::Info(info) => {
                write!(f, "info")?;
                if let Some(d) = info.depth {
                    write!(f, " depth {}", d)?;
                }
                if let Some(sd) = info.seldepth {
                    write!(f, " seldepth {}", sd)?;
                }
                if let Some(mp) = info.multipv {
                    write!(f, " multipv {}", mp)?;
                }
                if let Some(cp) = info.score_cp {
                    write!(f, " score cp {}", cp)?;
                }
                if let Some(mate) = info.score_mate {
                    write!(f, " score mate {}", mate)?;
                }
                if let Some(nodes) = info.nodes {
                    write!(f, " nodes {}", nodes)?;
                }
                if let Some(nps) = info.nps {
                    write!(f, " nps {}", nps)?;
                }
                if let Some(time) = info.time {
                    write!(f, " time {}", time)?;
                }
                if let Some(hf) = info.hashfull {
                    write!(f, " hashfull {}", hf)?;
                }
                if let Some(cm) = &info.currmove {
                    write!(f, " currmove {}", cm)?;
                }
                if let Some(cmn) = info.currmovenumber {
                    write!(f, " currmovenumber {}", cmn)?;
                }
                if !info.pv.is_empty() {
                    write!(f, " pv {}", info.pv.join(" "))?;
                }
                if let Some(s) = &info.string {
                    write!(f, " string {}", s)?;
                }
                Ok(())
            }
            UciResponse::Option(opt) => {
                write!(f, "{}", opt)
            }
        }
    }
}

pub fn send_response(response: &UciResponse) {
    println!("{}", response);
    io::stdout().flush().ok();
}

pub fn send_responses(responses: &[UciResponse]) {
    for response in responses {
        send_response(response);
    }
}

pub struct UciInput {
    reader: io::BufReader<io::Stdin>,
}

impl UciInput {
    pub fn new() -> Self {
        Self {
            reader: io::BufReader::new(io::stdin()),
        }
    }

    pub fn read_command(&mut self) -> Option<UciCommand> {
        let mut line = String::new();
        match self.reader.read_line(&mut line) {
            Ok(0) => None, // EOF
            Ok(_) => Some(parse_command(&line)),
            Err(_) => None,
        }
    }
}

impl Default for UciInput {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_uci() {
        assert!(matches!(parse_command("uci"), UciCommand::Uci));
    }

    #[test]
    fn test_parse_isready() {
        assert!(matches!(parse_command("isready"), UciCommand::IsReady));
    }

    #[test]
    fn test_parse_position_startpos() {
        match parse_command("position startpos") {
            UciCommand::Position { fen, moves } => {
                assert!(fen.is_none());
                assert!(moves.is_empty());
            }
            _ => panic!("Expected Position command"),
        }
    }

    #[test]
    fn test_parse_position_startpos_moves() {
        match parse_command("position startpos moves e2e4 e7e5") {
            UciCommand::Position { fen, moves } => {
                assert!(fen.is_none());
                assert_eq!(moves, vec!["e2e4", "e7e5"]);
            }
            _ => panic!("Expected Position command"),
        }
    }

    #[test]
    fn test_parse_position_fen() {
        let cmd = "position fen rnbqkbnr/pppppppp/8/8/4P3/8/PPPP1PPP/RNBQKBNR b KQkq e3 0 1";
        match parse_command(cmd) {
            UciCommand::Position { fen, moves } => {
                assert!(fen.is_some());
                assert!(moves.is_empty());
            }
            _ => panic!("Expected Position command"),
        }
    }

    #[test]
    fn test_parse_go_depth() {
        match parse_command("go depth 6") {
            UciCommand::Go(params) => {
                assert_eq!(params.depth, Some(6));
            }
            _ => panic!("Expected Go command"),
        }
    }

    #[test]
    fn test_parse_go_time() {
        match parse_command("go wtime 300000 btime 300000 winc 2000 binc 2000") {
            UciCommand::Go(params) => {
                assert_eq!(params.wtime, Some(300000));
                assert_eq!(params.btime, Some(300000));
                assert_eq!(params.winc, Some(2000));
                assert_eq!(params.binc, Some(2000));
            }
            _ => panic!("Expected Go command"),
        }
    }

    #[test]
    fn test_parse_go_infinite() {
        match parse_command("go infinite") {
            UciCommand::Go(params) => {
                assert!(params.infinite);
            }
            _ => panic!("Expected Go command"),
        }
    }

    #[test]
    fn test_parse_setoption() {
        match parse_command("setoption name Hash value 128") {
            UciCommand::SetOption { name, value } => {
                assert_eq!(name, "Hash");
                assert_eq!(value, Some("128".to_string()));
            }
            _ => panic!("Expected SetOption command"),
        }
    }

    #[test]
    fn test_response_format() {
        let resp = UciResponse::BestMove {
            best: "e2e4".to_string(),
            ponder: Some("e7e5".to_string()),
        };
        assert_eq!(format!("{}", resp), "bestmove e2e4 ponder e7e5");
    }

    #[test]
    fn test_info_response_format() {
        let info = InfoResponse::new()
            .with_depth(10)
            .with_score_cp(50)
            .with_nodes(12345)
            .with_pv(vec!["e2e4".to_string(), "e7e5".to_string()]);

        let resp = UciResponse::Info(info);
        let output = format!("{}", resp);

        assert!(output.contains("depth 10"));
        assert!(output.contains("score cp 50"));
        assert!(output.contains("nodes 12345"));
        assert!(output.contains("pv e2e4 e7e5"));
    }

    #[test]
    fn test_calculate_move_time() {
        let params = SearchParams {
            wtime: Some(60000),
            btime: Some(60000),
            winc: Some(1000),
            binc: Some(1000),
            ..Default::default()
        };

        let time = params.calculate_move_time(true);
        assert!(time.is_some());
        // Should be roughly 2-3 seconds with default 30 moves to go
    }
}
