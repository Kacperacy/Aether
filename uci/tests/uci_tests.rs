//! UCI protocol tests

use uci::{parse_uci_command, GoCommand, UciCommand};

#[test]
fn test_parse_uci_command() {
    assert_eq!(parse_uci_command("uci"), UciCommand::Uci);
    assert_eq!(parse_uci_command("isready"), UciCommand::IsReady);
    assert_eq!(parse_uci_command("ucinewgame"), UciCommand::UciNewGame);
    assert_eq!(parse_uci_command("quit"), UciCommand::Quit);
    assert_eq!(parse_uci_command("stop"), UciCommand::Stop);
}

#[test]
fn test_parse_position_startpos() {
    let cmd = parse_uci_command("position startpos");
    match cmd {
        UciCommand::Position { fen, moves } => {
            assert!(fen.is_none());
            assert!(moves.is_empty());
        }
        _ => panic!("Expected Position command"),
    }
}

#[test]
fn test_parse_position_startpos_moves() {
    let cmd = parse_uci_command("position startpos moves e2e4 e7e5");
    match cmd {
        UciCommand::Position { fen, moves } => {
            assert!(fen.is_none());
            assert_eq!(moves, vec!["e2e4", "e7e5"]);
        }
        _ => panic!("Expected Position command"),
    }
}

#[test]
fn test_parse_position_fen() {
    let cmd = parse_uci_command(
        "position fen rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1",
    );
    match cmd {
        UciCommand::Position { fen, moves } => {
            assert_eq!(
                fen,
                Some("rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1".to_string())
            );
            assert!(moves.is_empty());
        }
        _ => panic!("Expected Position command"),
    }
}

#[test]
fn test_parse_go_depth() {
    let cmd = parse_uci_command("go depth 10");
    match cmd {
        UciCommand::Go(go_cmd) => {
            assert_eq!(go_cmd.depth, Some(10));
        }
        _ => panic!("Expected Go command"),
    }
}

#[test]
fn test_parse_go_infinite() {
    let cmd = parse_uci_command("go infinite");
    match cmd {
        UciCommand::Go(go_cmd) => {
            assert!(go_cmd.infinite);
        }
        _ => panic!("Expected Go command"),
    }
}

#[test]
fn test_parse_go_movetime() {
    let cmd = parse_uci_command("go movetime 5000");
    match cmd {
        UciCommand::Go(go_cmd) => {
            assert_eq!(go_cmd.movetime, Some(5000));
        }
        _ => panic!("Expected Go command"),
    }
}

#[test]
fn test_parse_go_wtime_btime() {
    let cmd = parse_uci_command("go wtime 300000 btime 300000 winc 5000 binc 5000");
    match cmd {
        UciCommand::Go(go_cmd) => {
            assert_eq!(go_cmd.wtime, Some(300000));
            assert_eq!(go_cmd.btime, Some(300000));
            assert_eq!(go_cmd.winc, Some(5000));
            assert_eq!(go_cmd.binc, Some(5000));
        }
        _ => panic!("Expected Go command"),
    }
}

#[test]
fn test_parse_setoption_hash() {
    let cmd = parse_uci_command("setoption name Hash value 128");
    match cmd {
        UciCommand::SetOption { name, value } => {
            assert_eq!(name, "Hash");
            assert_eq!(value, Some("128".to_string()));
        }
        _ => panic!("Expected SetOption command"),
    }
}

#[test]
fn test_parse_setoption_move_overhead() {
    let cmd = parse_uci_command("setoption name Move Overhead value 100");
    match cmd {
        UciCommand::SetOption { name, value } => {
            assert_eq!(name, "Move Overhead");
            assert_eq!(value, Some("100".to_string()));
        }
        _ => panic!("Expected SetOption command"),
    }
}

#[test]
fn test_parse_setoption_moveoverhead_no_space() {
    let cmd = parse_uci_command("setoption name MoveOverhead value 200");
    match cmd {
        UciCommand::SetOption { name, value } => {
            assert_eq!(name, "MoveOverhead");
            assert_eq!(value, Some("200".to_string()));
        }
        _ => panic!("Expected SetOption command"),
    }
}

#[test]
fn test_parse_unknown_command() {
    let cmd = parse_uci_command("invalidcommand");
    match cmd {
        UciCommand::Unknown(s) => {
            assert_eq!(s, "invalidcommand");
        }
        _ => panic!("Expected Unknown command"),
    }
}

#[test]
fn test_go_command_calculate_time_white() {
    let go_cmd = GoCommand {
        wtime: Some(60000), // 1 minute
        btime: Some(60000),
        winc: Some(1000),  // 1 second increment
        binc: Some(1000),
        ..Default::default()
    };

    let time = go_cmd.calculate_time(true, 0).unwrap(); // No overhead
    // Should allocate some time based on remaining time and increment
    assert!(time.as_millis() > 0);
    assert!(time.as_millis() < 60000); // Less than total time
}

#[test]
fn test_go_command_calculate_time_movetime() {
    let go_cmd = GoCommand {
        movetime: Some(5000), // Exactly 5 seconds
        ..Default::default()
    };

    let time = go_cmd.calculate_time(true, 0).unwrap(); // No overhead
    assert_eq!(time.as_millis(), 5000);
}

#[test]
fn test_go_command_no_time_returns_none() {
    let go_cmd = GoCommand {
        depth: Some(10),
        ..Default::default()
    };

    assert!(go_cmd.calculate_time(true, 0).is_none());
}

#[test]
fn test_move_overhead_applied_to_movetime() {
    let go_cmd = GoCommand {
        movetime: Some(5000), // 5 seconds
        ..Default::default()
    };

    let time_no_overhead = go_cmd.calculate_time(true, 0).unwrap();
    let time_with_overhead = go_cmd.calculate_time(true, 100).unwrap();

    // With 100ms overhead, should get 100ms less search time
    assert_eq!(time_no_overhead.as_millis() - 100, time_with_overhead.as_millis());
}

#[test]
fn test_move_overhead_applied_to_time_control() {
    let go_cmd = GoCommand {
        wtime: Some(60000), // 1 minute
        btime: Some(60000),
        winc: Some(1000),  // 1 second increment
        binc: Some(1000),
        ..Default::default()
    };

    let time_no_overhead = go_cmd.calculate_time(true, 0).unwrap();
    let time_with_overhead = go_cmd.calculate_time(true, 200).unwrap();

    // With 200ms overhead, should get 200ms less search time
    assert_eq!(time_no_overhead.as_millis() - 200, time_with_overhead.as_millis());
}

#[test]
fn test_move_overhead_minimum_time() {
    let go_cmd = GoCommand {
        movetime: Some(50), // Very short time (50ms)
        ..Default::default()
    };

    // Even with large overhead, should get at least 1ms
    let time_with_large_overhead = go_cmd.calculate_time(true, 1000).unwrap();
    assert_eq!(time_with_large_overhead.as_millis(), 1);
}

#[test]
fn test_move_overhead_black_time() {
    let go_cmd = GoCommand {
        wtime: Some(60000),
        btime: Some(30000), // Black has half the time
        winc: Some(1000),
        binc: Some(500),   // Black has half the increment
        ..Default::default()
    };

    let white_time = go_cmd.calculate_time(true, 100).unwrap();
    let black_time = go_cmd.calculate_time(false, 100).unwrap();

    // Both should have overhead applied
    // Black should have less time due to less time and increment
    assert!(black_time.as_millis() < white_time.as_millis());
}
