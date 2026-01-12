use std::collections::HashMap;

/// Trigger types for time increment events
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TriggerType {
    TotalMoves,
    TotalActionPoints,
}

/// Trigger configuration for adding time during the game
#[derive(Debug, Clone)]
pub struct TimeIncrementTrigger {
    pub trigger_type: TriggerType,
    pub threshold: f32,
    pub increment: i32,
    pub targets: Vec<usize>,
}

/// Clock settings for a chess game
#[derive(Debug, Clone)]
pub struct ChessClockSettings {
    /// Initial time for each player in seconds (indexed by player ID)
    pub initial_times: HashMap<usize, i32>,
    /// Time increment added after each move (indexed by player ID)
    pub move_increments: HashMap<usize, i32>,
    /// Triggers for adding time based on game events
    pub triggers: Vec<TimeIncrementTrigger>,
}

/// Chess clock state tracking time for each player
#[derive(Debug, Clone)]
pub struct ChessClock {
    settings: ChessClockSettings,
    /// Remaining time for each player in seconds (indexed by player ID)
    remaining_times: HashMap<usize, i32>,
    /// Track moves and action points for trigger evaluation
    total_moves: u32,
    total_action_points: u32,
    /// Which player's clock is currently running (None if game hasn't started)
    active_player: Option<usize>,
}

impl ChessClock {
    /// Create a new chess clock from settings
    pub fn new(settings: ChessClockSettings) -> Self {
        let remaining_times = settings.initial_times.clone();

        ChessClock {
            settings,
            remaining_times,
            total_moves: 0,
            total_action_points: 0,
            active_player: None,
        }
    }

    /// Start the clock for a specific player
    pub fn start_player_clock(&mut self, player_id: usize) {
        self.active_player = Some(player_id);
    }

    /// Stop the current player's clock
    pub fn stop_clock(&mut self) {
        self.active_player = None;
    }

    /// Get the currently active player (whose clock is running)
    pub fn active_player(&self) -> Option<usize> {
        self.active_player
    }

    /// Get remaining time for a player
    pub fn get_remaining_time(&self, player_id: usize) -> Option<i32> {
        self.remaining_times.get(&player_id).copied()
    }

    /// Decrement the active player's time by one second
    /// Returns true if the player still has time, false if time ran out
    pub fn tick(&mut self) -> bool {
        if let Some(player_id) = self.active_player {
            if let Some(time) = self.remaining_times.get_mut(&player_id) {
                *time -= 1;
                return *time > 0;
            }
        }
        true
    }

    /// Called when a player completes their move
    /// Applies move increment and checks triggers
    pub fn end_turn(&mut self, player_id: usize) {
        // Apply move increment for this player
        if let Some(increment) = self.settings.move_increments.get(&player_id) {
            if let Some(time) = self.remaining_times.get_mut(&player_id) {
                *time += increment;
            }
        }

        // Increment move counter
        self.total_moves += 1;

        // Check and apply triggers
        self.check_triggers();

        // Stop this player's clock (caller will start the next player's)
        self.stop_clock();
    }

    /// Check if any triggers should fire and apply them
    fn check_triggers(&mut self) {
        for trigger in &self.settings.triggers {
            let should_fire = match trigger.trigger_type {
                TriggerType::TotalMoves => self.total_moves as f32 >= trigger.threshold,
                TriggerType::TotalActionPoints => {
                    self.total_action_points as f32 >= trigger.threshold
                }
            };

            if should_fire {
                // Apply increment to target players
                for &player_id in &trigger.targets {
                    if let Some(time) = self.remaining_times.get_mut(&player_id) {
                        *time += trigger.increment;
                    }
                }
            }
        }
    }

    /// Check if any player has run out of time
    /// Returns Some(player_id) if a player lost on time, None otherwise
    pub fn get_player_out_of_time(&self) -> Option<usize> {
        for (&player_id, &time) in &self.remaining_times {
            if time <= 0 {
                return Some(player_id);
            }
        }
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_clock_creation() {
        let mut initial_times = HashMap::new();
        initial_times.insert(0, 300); // 5 minutes
        initial_times.insert(1, 300);

        let mut increments = HashMap::new();
        increments.insert(0, 5);
        increments.insert(1, 5);

        let settings = ChessClockSettings {
            initial_times,
            move_increments: increments,
            triggers: vec![],
        };

        let clock = ChessClock::new(settings);
        assert_eq!(clock.get_remaining_time(0), Some(300));
        assert_eq!(clock.get_remaining_time(1), Some(300));
    }

    #[test]
    fn test_clock_tick() {
        let mut initial_times = HashMap::new();
        initial_times.insert(0, 10);

        let settings = ChessClockSettings {
            initial_times,
            move_increments: HashMap::new(),
            triggers: vec![],
        };

        let mut clock = ChessClock::new(settings);
        clock.start_player_clock(0);

        for _ in 0..5 {
            assert!(clock.tick());
        }
        assert_eq!(clock.get_remaining_time(0), Some(5));
    }

    #[test]
    fn test_clock_timeout() {
        let mut initial_times = HashMap::new();
        initial_times.insert(0, 2);

        let settings = ChessClockSettings {
            initial_times,
            move_increments: HashMap::new(),
            triggers: vec![],
        };

        let mut clock = ChessClock::new(settings);
        clock.start_player_clock(0);

        assert!(clock.tick()); // 1 second left
        assert!(!clock.tick()); // 0 seconds - timeout
        assert_eq!(clock.get_player_out_of_time(), Some(0));
    }

    #[test]
    fn test_move_increment() {
        let mut initial_times = HashMap::new();
        initial_times.insert(0, 60);

        let mut increments = HashMap::new();
        increments.insert(0, 10);

        let settings = ChessClockSettings {
            initial_times,
            move_increments: increments,
            triggers: vec![],
        };

        let mut clock = ChessClock::new(settings);
        clock.start_player_clock(0);

        // Simulate 5 seconds passing
        for _ in 0..5 {
            clock.tick();
        }

        clock.end_turn(0);

        // Should have 60 - 5 + 10 = 65 seconds
        assert_eq!(clock.get_remaining_time(0), Some(65));
    }
}
