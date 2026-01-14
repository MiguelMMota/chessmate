// Network client for connecting to ChessMate server
use futures_util::{SinkExt, StreamExt};
use std::error::Error;
use tokio::sync::mpsc;
use tokio_tungstenite::{connect_async, tungstenite::Message};

use crate::networking::protocol::{ClientMessage, GameAction, ServerMessage};
use crate::networking::types::SerializableGameState;

/// Network client for connecting to the game server
pub struct NetworkClient {
    player_id: String,
    server_url: String,
    tx: Option<mpsc::UnboundedSender<ClientMessage>>,
    rx: Option<mpsc::UnboundedReceiver<ServerMessage>>,
}

impl NetworkClient {
    /// Create a new network client
    pub fn new(player_id: String, server_url: String) -> Self {
        Self {
            player_id,
            server_url,
            tx: None,
            rx: None,
        }
    }

    /// Connect to the server and start message handling
    pub async fn connect(&mut self) -> Result<(), Box<dyn Error>> {
        let (ws_stream, _) = connect_async(&self.server_url).await?;
        let (mut ws_tx, mut ws_rx) = ws_stream.split();

        // Create channels for communication with the application
        let (tx_to_server, mut rx_from_app) = mpsc::unbounded_channel::<ClientMessage>();
        let (tx_to_app, rx_to_app) = mpsc::unbounded_channel::<ServerMessage>();

        // Store channels
        self.tx = Some(tx_to_server);
        self.rx = Some(rx_to_app);

        // Spawn task to send messages to server
        tokio::spawn(async move {
            while let Some(msg) = rx_from_app.recv().await {
                if let Ok(json) = serde_json::to_string(&msg) {
                    if ws_tx.send(Message::Text(json)).await.is_err() {
                        break;
                    }
                }
            }
        });

        // Spawn task to receive messages from server
        tokio::spawn(async move {
            while let Some(result) = ws_rx.next().await {
                match result {
                    Ok(Message::Text(text)) => {
                        if let Ok(msg) = serde_json::from_str::<ServerMessage>(&text) {
                            if tx_to_app.send(msg).is_err() {
                                break;
                            }
                        }
                    }
                    Ok(Message::Close(_)) => break,
                    Err(_) => break,
                    _ => {}
                }
            }
        });

        Ok(())
    }

    /// Join the matchmaking queue
    pub async fn join_matchmaking(&self) -> Result<(), Box<dyn Error>> {
        if let Some(tx) = &self.tx {
            let msg = ClientMessage::join_matchmaking(self.player_id.clone());
            tx.send(msg)?;
        }
        Ok(())
    }

    /// Submit a game action
    pub async fn submit_action(
        &self,
        game_id: &str,
        action: GameAction,
    ) -> Result<(), Box<dyn Error>> {
        if let Some(tx) = &self.tx {
            let msg = ClientMessage::submit_action(game_id.to_string(), action);
            tx.send(msg)?;
        }
        Ok(())
    }

    /// Leave a game
    pub async fn leave_game(&self, game_id: &str) -> Result<(), Box<dyn Error>> {
        if let Some(tx) = &self.tx {
            let msg = ClientMessage::leave_game(game_id.to_string());
            tx.send(msg)?;
        }
        Ok(())
    }

    /// Request current game state
    pub async fn request_state(&self, game_id: &str) -> Result<(), Box<dyn Error>> {
        if let Some(tx) = &self.tx {
            let msg = ClientMessage::request_state(game_id.to_string());
            tx.send(msg)?;
        }
        Ok(())
    }

    /// Try to receive a message from the server (non-blocking)
    pub async fn try_recv(&mut self) -> Option<ServerMessage> {
        if let Some(rx) = &mut self.rx {
            rx.try_recv().ok()
        } else {
            None
        }
    }

    /// Wait for the next message from the server (blocking)
    pub async fn recv(&mut self) -> Option<ServerMessage> {
        if let Some(rx) = &mut self.rx {
            rx.recv().await
        } else {
            None
        }
    }

    /// Get the player ID
    pub fn player_id(&self) -> &str {
        &self.player_id
    }

    /// Check if connected
    pub fn is_connected(&self) -> bool {
        self.tx.is_some() && self.rx.is_some()
    }
}

/// Simple client that handles common game flow
pub struct SimpleGameClient {
    client: NetworkClient,
    current_game_id: Option<String>,
    current_state: Option<SerializableGameState>,
}

impl SimpleGameClient {
    /// Create a new simple game client
    pub fn new(player_id: String, server_url: String) -> Self {
        Self {
            client: NetworkClient::new(player_id, server_url),
            current_game_id: None,
            current_state: None,
        }
    }

    /// Connect and join matchmaking
    pub async fn connect_and_join(&mut self) -> Result<(), Box<dyn Error>> {
        self.client.connect().await?;
        self.client.join_matchmaking().await?;
        Ok(())
    }

    /// Process incoming messages and update state
    pub async fn update(&mut self) -> Result<Vec<String>, Box<dyn Error>> {
        let mut events = Vec::new();

        while let Some(msg) = self.client.try_recv().await {
            match msg {
                ServerMessage::MatchmakingJoined => {
                    events.push("Joined matchmaking queue".to_string());
                }
                ServerMessage::MatchFound {
                    game_id,
                    opponent_id,
                    your_color,
                } => {
                    self.current_game_id = Some(game_id.clone());
                    events.push(format!(
                        "Match found! Game ID: {}, Opponent: {}, You are: {:?}",
                        game_id, opponent_id, your_color
                    ));
                }
                ServerMessage::GameStateUpdate { state } => {
                    self.current_state = Some(state.clone());
                    events.push(format!(
                        "Game state updated. Next player: {}",
                        state.next_player_id
                    ));
                }
                ServerMessage::OpponentAction { action } => {
                    events.push(format!("Opponent action: {:?}", action));
                }
                ServerMessage::GameOver { winner, reason } => {
                    events.push(format!(
                        "Game over! Winner: {:?}, Reason: {}",
                        winner, reason
                    ));
                    self.current_game_id = None;
                    self.current_state = None;
                }
                ServerMessage::InvalidAction { reason } => {
                    events.push(format!("Invalid action: {}", reason));
                }
                ServerMessage::Error { message } => {
                    events.push(format!("Error: {}", message));
                }
                ServerMessage::InvalidMove { from, to } => {
                    events.push(format!(
                        "Invalid move: cannot move from {:?} to {:?}",
                        from, to
                    ));
                }
                ServerMessage::GameNotFound { game_id } => {
                    events.push(format!("Game not found: {}", game_id));
                }
                ServerMessage::NotYourTurn => {
                    events.push("Not your turn".to_string());
                }
                ServerMessage::NotYourGame { game_id } => {
                    events.push(format!("Not your game: {}", game_id));
                }
                ServerMessage::MustJoinMatchmaking => {
                    events.push("Must join matchmaking first".to_string());
                }
                ServerMessage::InvalidMessageFormat { details } => {
                    events.push(format!("Invalid message format: {}", details));
                }
            }
        }

        Ok(events)
    }

    /// Submit a move
    pub async fn submit_move(
        &self,
        from_row: i8,
        from_col: i8,
        to_row: i8,
        to_col: i8,
        promotion: Option<crate::game::piece::PieceType>,
    ) -> Result<(), Box<dyn Error>> {
        if let Some(game_id) = &self.current_game_id {
            let from = crate::game::piece::Position::new(from_row, from_col);
            let to = crate::game::piece::Position::new(to_row, to_col);
            let action = GameAction::move_piece(from, to, promotion);
            self.client.submit_action(game_id, action).await?;
        }
        Ok(())
    }

    /// Resign from current game
    pub async fn resign(&self) -> Result<(), Box<dyn Error>> {
        if let Some(game_id) = &self.current_game_id {
            let action = GameAction::resign();
            self.client.submit_action(game_id, action).await?;
        }
        Ok(())
    }

    /// Get current game state
    pub fn current_state(&self) -> Option<&SerializableGameState> {
        self.current_state.as_ref()
    }

    /// Get current game ID
    pub fn current_game_id(&self) -> Option<&String> {
        self.current_game_id.as_ref()
    }

    /// Check if in a game
    pub fn in_game(&self) -> bool {
        self.current_game_id.is_some()
    }
}
