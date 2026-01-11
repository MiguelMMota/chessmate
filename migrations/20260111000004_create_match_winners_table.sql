-- Create match_winners junction table
-- This allows for multiple winners (e.g., draws, team games in future)
CREATE TABLE IF NOT EXISTS match_winners (
    match_id INTEGER NOT NULL REFERENCES matches(id) ON DELETE CASCADE,
    player_id INTEGER NOT NULL REFERENCES players(id) ON DELETE CASCADE,

    PRIMARY KEY (match_id, player_id)
);

-- Index for faster lookups
CREATE INDEX idx_match_winners_match_id ON match_winners(match_id);
CREATE INDEX idx_match_winners_player_id ON match_winners(player_id);
