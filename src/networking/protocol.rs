// Network protocol message types
use crate::game::piece::{Color, PieceType, Position};
use crate::networking::types::SerializableGameState;
use serde::{Deserialize, Serialize};

/// Messages sent from client to server
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum ClientMessage {
    /// Join the matchmaking queue
    JoinMatchmaking { player_id: String },

    /// Submit a game action (move, resign, etc.)
    SubmitAction { game_id: String, action: GameAction },

    /// Leave a game
    LeaveGame { game_id: String },

    /// Request current game state
    RequestState { game_id: String },
}

/// Messages sent from server to client
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum ServerMessage {
    /// Match found, game starting
    MatchFound {
        game_id: String,
        opponent_id: String,
        your_color: Color,
    },

    /// Full game state update
    GameStateUpdate { state: SerializableGameState },

    /// Opponent performed an action
    OpponentAction { action: GameAction },

    /// Game ended
    GameOver {
        winner: Option<Color>,
        reason: String,
    },

    /// Action was invalid
    InvalidAction { reason: String },

    /// Generic error
    Error { message: String },

    /// Acknowledgment that player joined matchmaking queue
    MatchmakingJoined,

    /// Specific error: Invalid move attempted
    InvalidMove { from: Position, to: Position },

    /// Specific error: Game not found
    GameNotFound { game_id: String },

    /// Specific error: Not the player's turn
    NotYourTurn,

    /// Specific error: Player not in the specified game
    NotYourGame { game_id: String },

    /// Specific error: Must join matchmaking before performing actions
    MustJoinMatchmaking,

    /// Specific error: Invalid message format
    InvalidMessageFormat { details: String },
}

/// Actions that can be performed during a game
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "action_type")]
pub enum GameAction {
    /// Move a piece from one position to another
    MovePiece {
        from: Position,
        to: Position,
        promotion: Option<PieceType>,
    },

    /// Resign from the game
    Resign,

    /// Offer a draw (future)
    OfferDraw,

    /// Accept a draw offer (future)
    AcceptDraw,

    /// Decline a draw offer (future)
    DeclineDraw,
}

impl ClientMessage {
    /// Create a join matchmaking message
    pub fn join_matchmaking(player_id: String) -> Self {
        ClientMessage::JoinMatchmaking { player_id }
    }

    /// Create a submit action message
    pub fn submit_action(game_id: String, action: GameAction) -> Self {
        ClientMessage::SubmitAction { game_id, action }
    }

    /// Create a leave game message
    pub fn leave_game(game_id: String) -> Self {
        ClientMessage::LeaveGame { game_id }
    }

    /// Create a request state message
    pub fn request_state(game_id: String) -> Self {
        ClientMessage::RequestState { game_id }
    }
}

impl ServerMessage {
    /// Create a match found message
    pub fn match_found(game_id: String, opponent_id: String, your_color: Color) -> Self {
        ServerMessage::MatchFound {
            game_id,
            opponent_id,
            your_color,
        }
    }

    /// Create a game state update message
    pub fn game_state_update(state: SerializableGameState) -> Self {
        ServerMessage::GameStateUpdate { state }
    }

    /// Create an opponent action message
    pub fn opponent_action(action: GameAction) -> Self {
        ServerMessage::OpponentAction { action }
    }

    /// Create a game over message
    pub fn game_over(winner: Option<Color>, reason: String) -> Self {
        ServerMessage::GameOver { winner, reason }
    }

    /// Create an invalid action message
    pub fn invalid_action(reason: String) -> Self {
        ServerMessage::InvalidAction { reason }
    }

    /// Create an error message
    pub fn error(message: String) -> Self {
        ServerMessage::Error { message }
    }

    /// Create a matchmaking joined acknowledgment
    pub fn matchmaking_joined() -> Self {
        ServerMessage::MatchmakingJoined
    }

    /// Create an invalid move error message
    pub fn invalid_move(from: Position, to: Position) -> Self {
        ServerMessage::InvalidMove { from, to }
    }

    /// Create a game not found error message
    pub fn game_not_found(game_id: String) -> Self {
        ServerMessage::GameNotFound { game_id }
    }

    /// Create a not your turn error message
    pub fn not_your_turn() -> Self {
        ServerMessage::NotYourTurn
    }

    /// Create a not your game error message
    pub fn not_your_game(game_id: String) -> Self {
        ServerMessage::NotYourGame { game_id }
    }

    /// Create a must join matchmaking error message
    pub fn must_join_matchmaking() -> Self {
        ServerMessage::MustJoinMatchmaking
    }

    /// Create an invalid message format error message
    pub fn invalid_message_format(details: String) -> Self {
        ServerMessage::InvalidMessageFormat { details }
    }
}

impl GameAction {
    /// Create a move piece action
    pub fn move_piece(from: Position, to: Position, promotion: Option<PieceType>) -> Self {
        GameAction::MovePiece {
            from,
            to,
            promotion,
        }
    }

    /// Create a resign action
    pub fn resign() -> Self {
        GameAction::Resign
    }
}
