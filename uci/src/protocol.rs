//! UCI protocol main loop

use crate::{
    commands::{parse_uci_command, UciCommand},
    engine_handler::UciEngine,
    send_info, send_response, ENGINE_AUTHOR, ENGINE_NAME, ENGINE_VERSION,
};
use std::io::{self, BufRead};

/// Run the UCI protocol loop
pub fn run_uci_loop() -> io::Result<()> {
    let mut engine = UciEngine::new().expect("Failed to create engine");
    let stdin = io::stdin();
    let mut lines = stdin.lock().lines();
    
    while let Some(Ok(line)) = lines.next() {
        let line = line.trim();
        if line.is_empty() {
            continue;
        }
        
        let command = parse_uci_command(line);
        
        match command {
            UciCommand::Uci => {
                send_response(&format!("id name {} {}", ENGINE_NAME, ENGINE_VERSION));
                send_response(&format!("id author {}", ENGINE_AUTHOR));
                
                // Send options
                send_response("option name Hash type spin default 64 min 1 max 1024");
                send_response("option name Threads type spin default 1 min 1 max 128");
                
                send_response("uciok");
            }
            
            UciCommand::IsReady => {
                send_response("readyok");
            }
            
            UciCommand::UciNewGame => {
                engine.new_game();
                send_info("New game started");
            }
            
            UciCommand::Position { fen, moves } => {
                match engine.set_position(fen, moves) {
                    Ok(()) => {}
                    Err(e) => send_info(&format!("Error setting position: {}", e)),
                }
            }
            
            UciCommand::Go(go_cmd) => {
                engine.go(go_cmd);
            }
            
            UciCommand::Stop => {
                // For now, we don't support stopping mid-search
                // In a full implementation, this would signal the searcher to stop
                send_info("Stop command received (not implemented)");
            }
            
            UciCommand::Quit => {
                break;
            }
            
            UciCommand::SetOption { name, value } => {
                send_info(&format!("Setting option {} = {:?} (not implemented)", name, value));
            }
            
            UciCommand::Unknown(cmd) => {
                send_info(&format!("Unknown command: {}", cmd));
            }
        }
    }
    
    Ok(())
}
