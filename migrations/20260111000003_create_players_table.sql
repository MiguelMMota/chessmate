-- Create player role enum
CREATE TYPE player_role AS ENUM ('WHITE', 'BLACK');

-- Create players table
CREATE TABLE IF NOT EXISTS players (
    id SERIAL PRIMARY KEY,
    user_id INTEGER NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    match_id INTEGER NOT NULL REFERENCES matches(id) ON DELETE CASCADE,
    role player_role NOT NULL,

    -- Constraint: each user can only play once per match
    CONSTRAINT unique_user_per_match UNIQUE (user_id, match_id)
);

-- Indexes for faster joins
CREATE INDEX idx_players_user_id ON players(user_id);
CREATE INDEX idx_players_match_id ON players(match_id);
