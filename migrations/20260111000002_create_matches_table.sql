-- Create matches table
CREATE TABLE IF NOT EXISTS matches (
    id SERIAL PRIMARY KEY,
    started_on TIMESTAMP NOT NULL DEFAULT NOW(),
    ended_on TIMESTAMP
);

-- Index for faster time-based queries
CREATE INDEX idx_matches_started_on ON matches(started_on);
CREATE INDEX idx_matches_ended_on ON matches(ended_on);
