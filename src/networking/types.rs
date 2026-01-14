// Network-compatible types for serialization
use crate::game::board::GameStatus;
use crate::game::piece::{Color, GameAction, Piece, PieceType, Position};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Represents a single piece's state on the board
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PieceState {
    pub id: u8,
    pub position: String, // algebraic notation (e.g., "e4")
    pub piece_type: String, // "pawn", "knight", "bishop", "rook", "queen", "king"
}

/// ID-based board representation: list of all pieces with their IDs and positions
/// Color can be inferred from ID: 0-15 = White, 16-31 = Black
pub type BoardState = Vec<PieceState>;

/// Time representation: player_id -> seconds_remaining
pub type TimeState = HashMap<String, i32>;

/// Serializable version of game state for network transmission
/// Uses ID-based piece tracking for client-side animation and reconciliation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SerializableGameState {
    pub board_state: BoardState,
    pub next_player_id: String,
    pub time: TimeState,
    pub status: GameStatus,
    pub game_id: String,
    pub last_action: Option<GameAction>, // The action that led to this state (for animation)
}

impl SerializableGameState {
    /// Create a new serializable game state with ID-based representation
    pub fn new(
        game_id: String,
        white_player_id: String,
        black_player_id: String,
        current_turn: Color,
        status: GameStatus,
        white_time: Option<i32>,
        black_time: Option<i32>,
        squares: &[[Option<Piece>; 8]; 8],
        last_action: Option<GameAction>,
    ) -> Self {
        let board_state = Self::squares_to_id_based(squares);

        let next_player_id = match current_turn {
            Color::White => white_player_id.clone(),
            Color::Black => black_player_id.clone(),
        };

        let mut time = HashMap::new();
        if let Some(wt) = white_time {
            time.insert(white_player_id, wt);
        }
        if let Some(bt) = black_time {
            time.insert(black_player_id, bt);
        }

        Self {
            board_state,
            next_player_id,
            time,
            status,
            game_id,
            last_action,
        }
    }

    /// Convert board squares to ID-based format
    /// Returns a list of all pieces with their IDs, positions, and types
    fn squares_to_id_based(squares: &[[Option<Piece>; 8]; 8]) -> BoardState {
        let mut pieces = Vec::new();

        for row in 0..8 {
            for col in 0..8 {
                if let Some(piece) = squares[row][col] {
                    let pos = Position::new(row as i8, col as i8);
                    pieces.push(PieceState {
                        id: piece.id,
                        position: pos.to_algebraic(),
                        piece_type: Self::piece_type_to_string(piece.piece_type),
                    });
                }
            }
        }

        pieces
    }

    /// Convert piece type to string name
    fn piece_type_to_string(piece_type: PieceType) -> String {
        match piece_type {
            PieceType::King => "king".to_string(),
            PieceType::Queen => "queen".to_string(),
            PieceType::Rook => "rook".to_string(),
            PieceType::Bishop => "bishop".to_string(),
            PieceType::Knight => "knight".to_string(),
            PieceType::Pawn => "pawn".to_string(),
        }
    }
}
