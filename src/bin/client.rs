// ChessMate CLI client for testing network multiplayer
use std::io::{self, Write};
use tokio::time::{sleep, Duration};

use chessmate::game::piece::{Color, Piece, PieceType};
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
                print_board(&state.board_state);

                // Check whose turn it is
                println!("\nCurrent turn: {:?}", state.current_turn);
                println!("Game status: {:?}", state.status);

                if let Some(white_time) = state.white_time {
                    println!("White time: {}s", white_time);
                }
                if let Some(black_time) = state.black_time {
                    println!("Black time: {}s", black_time);
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

/// Print the chess board
fn print_board(board: &[Vec<Option<Piece>>]) {
    println!("\n  +---+---+---+---+---+---+---+---+");
    for (row_idx, row) in board.iter().enumerate().rev() {
        print!("{} |", row_idx + 1);
        for piece_opt in row {
            let symbol = if let Some(piece) = piece_opt {
                format!(" {} ", piece_symbol(piece))
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

/// Get a colored symbol for a piece
fn piece_symbol(piece: &Piece) -> char {
    match (piece.piece_type, piece.color) {
        (PieceType::King, Color::White) => '♔',
        (PieceType::Queen, Color::White) => '♕',
        (PieceType::Rook, Color::White) => '♖',
        (PieceType::Bishop, Color::White) => '♗',
        (PieceType::Knight, Color::White) => '♘',
        (PieceType::Pawn, Color::White) => '♙',
        (PieceType::King, Color::Black) => '♚',
        (PieceType::Queen, Color::Black) => '♛',
        (PieceType::Rook, Color::Black) => '♜',
        (PieceType::Bishop, Color::Black) => '♝',
        (PieceType::Knight, Color::Black) => '♞',
        (PieceType::Pawn, Color::Black) => '♟',
    }
}

// Note: This is a simplified CLI client for demonstration
// A production version would use proper async input handling
// or a TUI library like crossterm/tui-rs
