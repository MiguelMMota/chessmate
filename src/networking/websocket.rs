// WebSocket handler for client connections
use futures_util::{SinkExt, StreamExt};
use tokio::sync::mpsc;
use tokio_tungstenite::tungstenite::Message;
use warp::ws::{WebSocket, Ws};
use warp::Filter;

use crate::networking::matchmaking::WaitingPlayer;
use crate::networking::protocol::{ClientMessage, ServerMessage};
use crate::networking::server::GameServer;

/// Handle a WebSocket connection from a client
pub async fn handle_websocket(ws: WebSocket, server: GameServer) {
    let (mut ws_tx, mut ws_rx) = ws.split();

    // Create a channel for sending messages to this client
    let (tx, mut rx) = mpsc::unbounded_channel::<ServerMessage>();

    // Spawn a task to forward messages from the channel to the WebSocket
    tokio::spawn(async move {
        while let Some(msg) = rx.recv().await {
            if let Ok(json) = serde_json::to_string(&msg) {
                if ws_tx.send(Message::text(json)).await.is_err() {
                    break;
                }
            }
        }
    });

    // Variable to store player ID once they join matchmaking
    let mut player_id: Option<String> = None;

    // Process incoming messages from the WebSocket
    while let Some(result) = ws_rx.next().await {
        match result {
            Ok(msg) => {
                if let Ok(text) = msg.to_text() {
                    // Try to deserialize the message
                    match serde_json::from_str::<ClientMessage>(text) {
                        Ok(client_msg) => {
                            // Extract player_id from the message if we don't have it yet
                            if player_id.is_none() {
                                if let ClientMessage::JoinMatchmaking {
                                    player_id: ref pid,
                                } = client_msg
                                {
                                    player_id = Some(pid.clone());

                                    // Add player to matchmaking queue
                                    let player = WaitingPlayer::new(pid.clone(), tx.clone());
                                    if let Err(e) = server.add_to_matchmaking(player).await {
                                        eprintln!("Failed to add player to matchmaking: {}", e);
                                        // Use generic error for matchmaking failures
                                        let _ = tx.send(ServerMessage::error(e));
                                        continue;
                                    }

                                    // Send acknowledgment
                                    let _ = tx.send(ServerMessage::matchmaking_joined());
                                }
                            }

                            // Handle the message
                            if let Some(ref pid) = player_id {
                                if let Err(e) = server.handle_message(pid, client_msg).await {
                                    eprintln!("Error handling message from {}: {}", pid, e);
                                    // Check if error is specific and already sent by server
                                    // Otherwise send as generic error
                                    if e.contains("Game not found") {
                                        // Specific error already sent by server
                                    } else {
                                        let _ = tx.send(ServerMessage::error(e));
                                    }
                                }
                            } else {
                                let _ = tx.send(ServerMessage::must_join_matchmaking());
                            }
                        }
                        Err(e) => {
                            eprintln!("Failed to deserialize message: {}", e);
                            let _ = tx.send(ServerMessage::invalid_message_format(e.to_string()));
                        }
                    }
                }
            }
            Err(e) => {
                eprintln!("WebSocket error: {}", e);
                break;
            }
        }
    }

    // Client disconnected
    if let Some(pid) = player_id {
        println!("Player {} disconnected", pid);
    }
}

/// Create the WebSocket route
pub fn websocket_route(
    server: GameServer,
) -> impl Filter<Extract = impl warp::Reply, Error = warp::Rejection> + Clone {
    warp::path("ws")
        .and(warp::ws())
        .map(move |ws: Ws| {
            let server = server.clone_refs();
            ws.on_upgrade(move |websocket| handle_websocket(websocket, server))
        })
}

/// Create health check endpoint
pub fn health_route() -> impl Filter<Extract = impl warp::Reply, Error = warp::Rejection> + Clone {
    warp::path("health").and(warp::get()).map(|| {
        warp::reply::json(&serde_json::json!({
            "status": "ok",
            "service": "chessmate-server"
        }))
    })
}

/// Create stats endpoint
pub fn stats_route(
    server: GameServer,
) -> impl Filter<Extract = impl warp::Reply, Error = warp::Rejection> + Clone {
    warp::path("stats").and(warp::get()).then(move || {
        let server = server.clone_refs();
        async move {
            let active_games = server.active_game_count().await;
            let matchmaking_players = server.matchmaking_count().await;

            warp::reply::json(&serde_json::json!({
                "active_games": active_games,
                "matchmaking_players": matchmaking_players,
            }))
        }
    })
}
