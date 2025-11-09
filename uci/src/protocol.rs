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
                send_response("option name Move Overhead type spin default 10 min 0 max 5000");
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
                engine.stop();
                send_info("Stop command received");
            }
            
            UciCommand::Quit => {
                break;
            }
            
            UciCommand::SetOption { name, value } => {
                match name.to_lowercase().replace(" ", "").as_str() {
                    "hash" => {
                        if let Some(val_str) = value {
                            match val_str.parse::<usize>() {
                                Ok(size) => {
                                    engine.set_hash_size(size);
                                    send_info(&format!("Hash size set to {} MB", size));
                                }
                                Err(_) => send_info(&format!("Invalid hash size: {}", val_str)),
                            }
                        } else {
                            send_info("Hash option requires a value");
                        }
                    }
                    "moveoverhead" => {
                        if let Some(val_str) = value {
                            match val_str.parse::<u64>() {
                                Ok(overhead) => {
                                    engine.set_move_overhead(overhead);
                                    send_info(&format!("Move overhead set to {} ms", overhead));
                                }
                                Err(_) => send_info(&format!("Invalid move overhead: {}", val_str)),
                            }
                        } else {
                            send_info("Move Overhead option requires a value");
                        }
                    }
                    "threads" => {
                        send_info("Threads option not yet implemented (single-threaded engine)");
                    }
                    _ => {
                        send_info(&format!("Unknown option: {}", name));
                    }
                }
            }
            
            UciCommand::Unknown(cmd) => {
                send_info(&format!("Unknown command: {}", cmd));
            }
        }
    }
    
    Ok(())
}
