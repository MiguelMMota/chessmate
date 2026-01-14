// ChessMate CLI client for testing network multiplayer
use std::io::{self, Write};
use tokio::time::{sleep, Duration};

use chessmate::networking::client::SimpleGameClient;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🎮 ChessMate CLI Client");
    println!("======================\n");

    // Get player ID from command line or generate one
    let player_id = std::env::args()
        .nth(1)
        .unwrap_or_else(|| format!("player_{}", rand::random::<u16>()));

    // Get server URL from environment or use default
    let server_url =
        std::env::var("SERVER_URL").unwrap_or_else(|_| "ws://localhost:3000/ws".to_string());

    println!("Player ID: {}", player_id);
    println!("Server: {}\n", server_url);

    // Create and connect client
    let mut client = SimpleGameClient::new(player_id.clone(), server_url);

    println!("Connecting to server...");
    client.connect_and_join().await?;
    println!("✓ Connected and joined matchmaking queue");
    println!("Waiting for opponent...\n");

    // Main game loop
    loop {
        // Process server messages
        let events = client.update().await?;
        for event in events {
            println!("📬 {}", event);
        }

        // If in a game, show board and prompt for move
        if client.in_game() {
            if let Some(state) = client.current_state() {
                // Print board
                print_board_compact(&state.board_state);

                // Check whose turn it is
                println!("\nNext player: {}", state.next_player_id);
                println!("Game status: {:?}", state.status);

                // Print time for each player
                for (player_id, time) in &state.time {
                    println!("{}: {}s", player_id, time);
                }

                println!("\nCommands:");
                println!("  move <from> <to>  - Make a move (e.g., 'move e2 e4')");
                println!("  resign           - Resign from the game");
                println!("  quit             - Disconnect and exit");
                print!("\n> ");
                io::stdout().flush()?;

                // Non-blocking input handling
                // For simplicity in this demo, we'll just sleep and check for messages
                sleep(Duration::from_millis(100)).await;
            }
        } else {
            // Not in a game, just wait for matchmaking
            sleep(Duration::from_millis(500)).await;
        }
    }
}

/// Print the chess board from ID-based representation
fn print_board_compact(board_state: &Vec<chessmate::networking::types::PieceState>) {
    use chessmate::game::piece::Position;

    // Create empty board
    let mut display_board: [[Option<(String, bool)>; 8]; 8] = Default::default();

    // Fill board from ID-based representation
    for piece_state in board_state {
        // Color is inferred from ID: 0-15 = White, 16-31 = Black
        let is_white = piece_state.id < 16;

        if let Some(pos) = Position::from_algebraic(&piece_state.position) {
            if pos.is_valid() {
                display_board[pos.row as usize][pos.col as usize] =
                    Some((piece_state.piece_type.clone(), is_white));
            }
        }
    }

    println!("\n  +---+---+---+---+---+---+---+---+");
    for row_idx in (0..8).rev() {
        print!("{} |", row_idx + 1);
        for col_idx in 0..8 {
            let symbol = if let Some((piece_code, is_white)) = &display_board[row_idx][col_idx] {
                format!(" {} ", piece_code_to_symbol(piece_code, *is_white))
            } else {
                "   ".to_string()
            };
            print!("{}|", symbol);
        }
        println!();
        println!("  +---+---+---+---+---+---+---+---+");
    }
    println!("    a   b   c   d   e   f   g   h");
}

/// Convert piece type to symbol
fn piece_code_to_symbol(piece_type: &str, is_white: bool) -> char {
    match piece_type.to_lowercase().as_str() {
        "king" => if is_white { '♔' } else { '♚' },
        "queen" => if is_white { '♕' } else { '♛' },
        "rook" => if is_white { '♖' } else { '♜' },
        "bishop" => if is_white { '♗' } else { '♝' },
        "knight" => if is_white { '♘' } else { '♞' },
        "pawn" => if is_white { '♙' } else { '♟' },
        _ => '?',
    }
}

// Note: This is a simplified CLI client for demonstration
// A production version would use proper async input handling
// or a TUI library like crossterm/tui-rs
