// Network-compatible types for serialization
use crate::game::board::GameStatus;
use crate::game::piece::{Color, Piece, Position};
use serde::{Deserialize, Serialize};

/// Serializable version of game state for network transmission
/// Unlike the FFI GameState which uses raw pointers, this uses owned Strings
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SerializableGameState {
    pub game_id: String,
    pub current_turn: Color,
    pub status: GameStatus,
    pub white_time: Option<i32>,  // None if no clock
    pub black_time: Option<i32>,  // None if no clock
    pub board_state: Vec<Vec<Option<Piece>>>,  // 8x8 board
}

impl SerializableGameState {
    /// Create a new serializable game state
    pub fn new(
        game_id: String,
        current_turn: Color,
        status: GameStatus,
        white_time: Option<i32>,
        black_time: Option<i32>,
        board_state: Vec<Vec<Option<Piece>>>,
    ) -> Self {
        Self {
            game_id,
            current_turn,
            status,
            white_time,
            black_time,
            board_state,
        }
    }

    /// Convert board squares array to serializable format
    pub fn board_to_vec(squares: &[[Option<Piece>; 8]; 8]) -> Vec<Vec<Option<Piece>>> {
        squares.iter().map(|row| row.to_vec()).collect()
    }

    /// Get piece at position
    pub fn get_piece(&self, pos: Position) -> Option<Piece> {
        if !pos.is_valid() {
            return None;
        }
        self.board_state[pos.row as usize][pos.col as usize]
    }
}
