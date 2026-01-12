-- Create users table
CREATE TABLE IF NOT EXISTS users (
    id SERIAL PRIMARY KEY,
    name VARCHAR(255) NOT NULL,
    email VARCHAR(255) NOT NULL UNIQUE,

    -- Authentication fields
    -- For email/password: password_hash is set, oauth fields are NULL
    -- For OAuth: oauth_provider and oauth_id are set, password_hash is NULL
    password_hash VARCHAR(255),
    oauth_provider VARCHAR(50),  -- 'google', 'apple', etc.
    oauth_id VARCHAR(255),

    -- Game stats
    rating INTEGER NOT NULL DEFAULT 1000,

    -- Timestamps
    created_on TIMESTAMP NOT NULL DEFAULT NOW(),
    updated_on TIMESTAMP NOT NULL DEFAULT NOW(),

    -- Constraints
    CONSTRAINT check_auth_method CHECK (
        (password_hash IS NOT NULL AND oauth_provider IS NULL AND oauth_id IS NULL) OR
        (password_hash IS NULL AND oauth_provider IS NOT NULL AND oauth_id IS NOT NULL)
    ),
    CONSTRAINT unique_oauth_account UNIQUE (oauth_provider, oauth_id)
);

-- Index for faster lookups
CREATE INDEX idx_users_email ON users(email);
CREATE INDEX idx_users_oauth ON users(oauth_provider, oauth_id);

-- Trigger to automatically update updated_on timestamp
CREATE OR REPLACE FUNCTION update_updated_on_column()
RETURNS TRIGGER AS $$
BEGIN
    NEW.updated_on = NOW();
    RETURN NEW;
END;
$$ language 'plpgsql';

CREATE TRIGGER update_users_updated_on BEFORE UPDATE ON users
    FOR EACH ROW EXECUTE FUNCTION update_updated_on_column();
