-- Create user_owned_cards table
-- Tracks which cards each user has unlocked
-- Card definitions are stored in server code, not in the database
CREATE TABLE IF NOT EXISTS user_owned_cards (
    user_id INTEGER NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    card_id INTEGER NOT NULL,
    unlocked_on TIMESTAMP NOT NULL DEFAULT NOW(),

    PRIMARY KEY (user_id, card_id)
);

-- Index for faster lookups
CREATE INDEX idx_user_owned_cards_user_id ON user_owned_cards(user_id);
CREATE INDEX idx_user_owned_cards_card_id ON user_owned_cards(card_id);
