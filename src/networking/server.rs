// Game server that manages active games and player connections
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::{mpsc, RwLock};

use crate::game::board::GameStatus;
use crate::game::game_state::ChessGame;
use crate::game::piece::{Color, Position};
use crate::game::rules;
use crate::networking::matchmaking::{Match, MatchmakingQueue, WaitingPlayer};
use crate::networking::protocol::{ClientMessage, GameAction, ServerMessage};
use crate::networking::types::SerializableGameState;

/// A game session on the server
#[derive(Debug)]
pub struct ServerGame {
    pub game_id: String,
    pub game: ChessGame,
    pub white_player_id: String,
    pub black_player_id: String,
    pub white_sender: mpsc::UnboundedSender<ServerMessage>,
    pub black_sender: mpsc::UnboundedSender<ServerMessage>,
}

impl ServerGame {
    pub fn new(
        game_id: String,
        white_player_id: String,
        black_player_id: String,
        white_sender: mpsc::UnboundedSender<ServerMessage>,
        black_sender: mpsc::UnboundedSender<ServerMessage>,
    ) -> Self {
        Self {
            game_id,
            game: ChessGame::new(),
            white_player_id,
            black_player_id,
            white_sender,
            black_sender,
        }
    }

    /// Get the color for a given player ID
    pub fn get_player_color(&self, player_id: &str) -> Option<Color> {
        if player_id == self.white_player_id {
            Some(Color::White)
        } else if player_id == self.black_player_id {
            Some(Color::Black)
        } else {
            None
        }
    }

    /// Check if it's a player's turn
    pub fn is_player_turn(&self, player_id: &str) -> bool {
        if let Some(color) = self.get_player_color(player_id) {
            self.game.board().current_turn() == color
        } else {
            false
        }
    }

    /// Convert internal game state to serializable format
    pub fn to_serializable_state(&self) -> SerializableGameState {
        let board = self.game.board();
        let status = rules::get_game_status(board);

        SerializableGameState::new(
            self.game_id.clone(),
            self.white_player_id.clone(),
            self.black_player_id.clone(),
            board.current_turn(),
            status,
            board.get_remaining_time(Color::White),
            board.get_remaining_time(Color::Black),
            &self.game.board_squares(),
            board.last_action(),
        )
    }

    /// Broadcast game state to both players
    pub fn broadcast_state(&self) {
        let state = self.to_serializable_state();
        let msg = ServerMessage::game_state_update(state);

        let _ = self.white_sender.send(msg.clone());
        let _ = self.black_sender.send(msg);
    }

    /// Send message to opponent
    pub fn send_to_opponent(&self, player_id: &str, msg: ServerMessage) {
        let sender = if player_id == self.white_player_id {
            &self.black_sender
        } else {
            &self.white_sender
        };

        let _ = sender.send(msg);
    }
}

/// Main game server managing all games and matchmaking
#[derive(Clone)]
pub struct GameServer {
    active_games: Arc<RwLock<HashMap<String, ServerGame>>>,
    matchmaking: Arc<RwLock<MatchmakingQueue>>,
    player_to_game: Arc<RwLock<HashMap<String, String>>>, // player_id -> game_id
}

impl GameServer {
    pub fn new() -> Self {
        Self {
            active_games: Arc::new(RwLock::new(HashMap::new())),
            matchmaking: Arc::new(RwLock::new(MatchmakingQueue::new())),
            player_to_game: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Get a clone of the Arc pointers for use in async tasks
    pub fn clone_refs(&self) -> Self {
        Self {
            active_games: Arc::clone(&self.active_games),
            matchmaking: Arc::clone(&self.matchmaking),
            player_to_game: Arc::clone(&self.player_to_game),
        }
    }

    /// Handle a client message
    pub async fn handle_message(
        &self,
        player_id: &str,
        message: ClientMessage,
    ) -> Result<(), String> {
        match message {
            ClientMessage::JoinMatchmaking { player_id } => {
                self.handle_join_matchmaking(player_id).await
            }
            ClientMessage::SubmitAction { game_id, action } => {
                self.handle_submit_action(player_id, &game_id, action).await
            }
            ClientMessage::LeaveGame { game_id } => {
                self.handle_leave_game(player_id, &game_id).await
            }
            ClientMessage::RequestState { game_id } => {
                self.handle_request_state(player_id, &game_id).await
            }
        }
    }

    /// Handle player joining matchmaking queue
    async fn handle_join_matchmaking(&self, _player_id: String) -> Result<(), String> {
        // Note: The actual adding to queue happens in the WebSocket handler
        // This is just for validation
        Ok(())
    }

    /// Add a player to the matchmaking queue (called from WebSocket handler)
    pub async fn add_to_matchmaking(&self, player: WaitingPlayer) -> Result<(), String> {
        let mut queue = self.matchmaking.write().await;
        queue.add_player(player);
        Ok(())
    }

    /// Try to create matches from the queue
    pub async fn try_matchmaking(&self) -> Vec<Match> {
        let mut queue = self.matchmaking.write().await;
        queue.try_create_matches()
    }

    /// Create a game from a match
    pub async fn create_game_from_match(&self, m: Match) {
        let game = ServerGame::new(
            m.game_id.clone(),
            m.white_player.player_id.clone(),
            m.black_player.player_id.clone(),
            m.white_player.sender.clone(),
            m.black_player.sender.clone(),
        );

        // Notify players that match was found
        let _ = m.white_player.sender.send(ServerMessage::match_found(
            m.game_id.clone(),
            m.black_player.player_id.clone(),
            Color::White,
        ));

        let _ = m.black_player.sender.send(ServerMessage::match_found(
            m.game_id.clone(),
            m.white_player.player_id.clone(),
            Color::Black,
        ));

        // Send initial game state
        game.broadcast_state();

        // Store game and player mappings
        let mut games = self.active_games.write().await;
        let mut player_map = self.player_to_game.write().await;

        player_map.insert(m.white_player.player_id.clone(), m.game_id.clone());
        player_map.insert(m.black_player.player_id.clone(), m.game_id.clone());
        games.insert(m.game_id, game);
    }

    /// Handle player submitting an action
    async fn handle_submit_action(
        &self,
        player_id: &str,
        game_id: &str,
        action: GameAction,
    ) -> Result<(), String> {
        let mut games = self.active_games.write().await;

        let game = games
            .get_mut(game_id)
            .ok_or_else(|| "Game not found".to_string())?;

        // Verify it's the player's turn
        if !game.is_player_turn(player_id) {
            let msg = ServerMessage::invalid_action("Not your turn".to_string());
            if let Some(color) = game.get_player_color(player_id) {
                let sender = if color == Color::White {
                    &game.white_sender
                } else {
                    &game.black_sender
                };
                let _ = sender.send(msg);
            }
            return Err("Not your turn".to_string());
        }

        // Process the action
        match action {
            GameAction::MovePiece {
                from,
                to,
                promotion,
            } => {
                self.process_move(game, player_id, from, to, promotion)
                    .await
            }
            GameAction::Resign => self.process_resign(game, player_id).await,
            GameAction::OfferDraw | GameAction::AcceptDraw | GameAction::DeclineDraw => {
                // TODO: Implement draw offers
                Ok(())
            }
        }
    }

    /// Process a move action
    async fn process_move(
        &self,
        game: &mut ServerGame,
        player_id: &str,
        from: Position,
        to: Position,
        promotion: Option<crate::game::piece::PieceType>,
    ) -> Result<(), String> {
        // Select the piece first
        game.game.select_piece(from.row, from.col);

        // Try to move (with or without promotion)
        let success = if let Some(promo) = promotion {
            game.game
                .try_move_selected_with_promotion(to.row, to.col, promo)
        } else {
            game.game.try_move_selected(to.row, to.col)
        };

        if !success {
            let msg = ServerMessage::invalid_action("Illegal move".to_string());
            if let Some(color) = game.get_player_color(player_id) {
                let sender = if color == Color::White {
                    &game.white_sender
                } else {
                    &game.black_sender
                };
                let _ = sender.send(msg);
            }
            return Err("Illegal move".to_string());
        }

        // Notify opponent of the move
        let action = GameAction::move_piece(from, to, promotion);
        game.send_to_opponent(player_id, ServerMessage::opponent_action(action));

        // Broadcast updated game state
        game.broadcast_state();

        // Check if game is over
        let status = rules::get_game_status(game.game.board());
        if !matches!(status, GameStatus::Ongoing | GameStatus::Check) {
            let (winner, reason) = match status {
                GameStatus::Checkmate(color) => (Some(color), "Checkmate".to_string()),
                GameStatus::Stalemate => (None, "Stalemate".to_string()),
                GameStatus::DrawInsufficientMaterial => (None, "Insufficient material".to_string()),
                GameStatus::TimeLoss(color) => (Some(color.opposite()), "Time out".to_string()),
                _ => (None, "Game over".to_string()),
            };

            let msg = ServerMessage::game_over(winner, reason);
            let _ = game.white_sender.send(msg.clone());
            let _ = game.black_sender.send(msg);
        }

        Ok(())
    }

    /// Process a resign action
    async fn process_resign(&self, game: &mut ServerGame, player_id: &str) -> Result<(), String> {
        let winner = if player_id == game.white_player_id {
            Some(Color::Black)
        } else {
            Some(Color::White)
        };

        let msg = ServerMessage::game_over(winner, "Resignation".to_string());
        let _ = game.white_sender.send(msg.clone());
        let _ = game.black_sender.send(msg);

        Ok(())
    }

    /// Handle player leaving a game
    async fn handle_leave_game(&self, player_id: &str, game_id: &str) -> Result<(), String> {
        let mut games = self.active_games.write().await;
        let mut player_map = self.player_to_game.write().await;

        // Extract player IDs before removing the game
        if let Some(game) = games.get(game_id) {
            let white_id = game.white_player_id.clone();
            let black_id = game.black_player_id.clone();

            // Notify opponent
            let winner = if player_id == white_id {
                Some(Color::Black)
            } else {
                Some(Color::White)
            };

            let msg = ServerMessage::game_over(winner, "Opponent left".to_string());
            game.send_to_opponent(player_id, msg);

            // Now remove game (after we're done with references to it)
            games.remove(game_id);
            player_map.remove(&white_id);
            player_map.remove(&black_id);
        }

        Ok(())
    }

    /// Handle request for game state
    async fn handle_request_state(&self, player_id: &str, game_id: &str) -> Result<(), String> {
        let games = self.active_games.read().await;

        let game = games
            .get(game_id)
            .ok_or_else(|| "Game not found".to_string())?;

        // Verify player is in this game
        if player_id != game.white_player_id && player_id != game.black_player_id {
            return Err("Not your game".to_string());
        }

        // Send current state
        let state = game.to_serializable_state();
        let msg = ServerMessage::game_state_update(state);

        let sender = if player_id == game.white_player_id {
            &game.white_sender
        } else {
            &game.black_sender
        };

        let _ = sender.send(msg);

        Ok(())
    }

    /// Get the number of active games
    pub async fn active_game_count(&self) -> usize {
        self.active_games.read().await.len()
    }

    /// Get the number of players in matchmaking
    pub async fn matchmaking_count(&self) -> usize {
        self.matchmaking.read().await.player_count()
    }
}

impl Default for GameServer {
    fn default() -> Self {
        Self::new()
    }
}
