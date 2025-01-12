-- Sessions management table
-- Used to refresh the tokens
CREATE TABLE sessions (
    -- The id of the session.
    -- This is distinct from the actual session key.
    id SERIAL PRIMARY KEY,
    -- The related user for the session
    user_id INTEGER NOT NULL REFERENCES users(id),
    -- The hash of the session token
    hash CHAR(64) NOT NULL
);
