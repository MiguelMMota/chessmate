// Matchmaking queue for pairing players
use std::time::Instant;
use tokio::sync::mpsc;
use uuid::Uuid;

use crate::networking::protocol::ServerMessage;

/// A player waiting in the matchmaking queue
#[derive(Debug, Clone)]
pub struct WaitingPlayer {
    pub player_id: String,
    pub joined_at: Instant,
    pub sender: mpsc::UnboundedSender<ServerMessage>,
}

impl WaitingPlayer {
    pub fn new(player_id: String, sender: mpsc::UnboundedSender<ServerMessage>) -> Self {
        Self {
            player_id,
            joined_at: Instant::now(),
            sender,
        }
    }
}

/// A matched pair of players ready to start a game
#[derive(Debug, Clone)]
pub struct Match {
    pub game_id: String,
    pub white_player: WaitingPlayer,
    pub black_player: WaitingPlayer,
}

impl Match {
    pub fn new(white_player: WaitingPlayer, black_player: WaitingPlayer) -> Self {
        Self {
            game_id: Uuid::new_v4().to_string(),
            white_player,
            black_player,
        }
    }
}

/// Matchmaking queue that pairs players
#[derive(Debug)]
pub struct MatchmakingQueue {
    waiting_players: Vec<WaitingPlayer>,
}

impl MatchmakingQueue {
    pub fn new() -> Self {
        Self {
            waiting_players: Vec::new(),
        }
    }

    /// Add a player to the matchmaking queue
    pub fn add_player(&mut self, player: WaitingPlayer) {
        self.waiting_players.push(player);
    }

    /// Remove a player from the queue by player_id
    pub fn remove_player(&mut self, player_id: &str) -> bool {
        if let Some(index) = self
            .waiting_players
            .iter()
            .position(|p| p.player_id == player_id)
        {
            self.waiting_players.remove(index);
            true
        } else {
            false
        }
    }

    /// Try to create matches from waiting players
    /// Simple algorithm: pair the first two players in the queue
    /// Returns a vector of matched pairs
    pub fn try_create_matches(&mut self) -> Vec<Match> {
        let mut matches = Vec::new();

        while self.waiting_players.len() >= 2 {
            // Take the first two players
            let player1 = self.waiting_players.remove(0);
            let player2 = self.waiting_players.remove(0);

            // Randomly assign colors (50/50)
            let (white_player, black_player) = if rand::random::<bool>() {
                (player1, player2)
            } else {
                (player2, player1)
            };

            matches.push(Match::new(white_player, black_player));
        }

        matches
    }

    /// Get the number of players waiting
    pub fn player_count(&self) -> usize {
        self.waiting_players.len()
    }

    /// Check if a player is in the queue
    pub fn contains_player(&self, player_id: &str) -> bool {
        self.waiting_players
            .iter()
            .any(|p| p.player_id == player_id)
    }
}

impl Default for MatchmakingQueue {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_matchmaking_queue_creation() {
        let queue = MatchmakingQueue::new();
        assert_eq!(queue.player_count(), 0);
    }

    #[test]
    fn test_add_player() {
        let mut queue = MatchmakingQueue::new();
        let (tx, _rx) = mpsc::unbounded_channel();
        let player = WaitingPlayer::new("player1".to_string(), tx);

        queue.add_player(player);
        assert_eq!(queue.player_count(), 1);
        assert!(queue.contains_player("player1"));
    }

    #[test]
    fn test_remove_player() {
        let mut queue = MatchmakingQueue::new();
        let (tx, _rx) = mpsc::unbounded_channel();
        let player = WaitingPlayer::new("player1".to_string(), tx);

        queue.add_player(player);
        assert_eq!(queue.player_count(), 1);

        let removed = queue.remove_player("player1");
        assert!(removed);
        assert_eq!(queue.player_count(), 0);
    }

    #[test]
    fn test_create_matches() {
        let mut queue = MatchmakingQueue::new();
        let (tx1, _rx1) = mpsc::unbounded_channel();
        let (tx2, _rx2) = mpsc::unbounded_channel();

        let player1 = WaitingPlayer::new("player1".to_string(), tx1);
        let player2 = WaitingPlayer::new("player2".to_string(), tx2);

        queue.add_player(player1);
        queue.add_player(player2);

        let matches = queue.try_create_matches();
        assert_eq!(matches.len(), 1);
        assert_eq!(queue.player_count(), 0);

        let m = &matches[0];
        assert!(m.white_player.player_id == "player1" || m.white_player.player_id == "player2");
        assert!(m.black_player.player_id == "player1" || m.black_player.player_id == "player2");
        assert_ne!(m.white_player.player_id, m.black_player.player_id);
    }

    #[test]
    fn test_no_match_with_single_player() {
        let mut queue = MatchmakingQueue::new();
        let (tx, _rx) = mpsc::unbounded_channel();
        let player = WaitingPlayer::new("player1".to_string(), tx);

        queue.add_player(player);

        let matches = queue.try_create_matches();
        assert_eq!(matches.len(), 0);
        assert_eq!(queue.player_count(), 1);
    }
}
