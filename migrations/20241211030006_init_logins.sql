-- Add migration script here
CREATE TABLE logins (
    -- The related user for the login
    user_id INTEGER NOT NULL UNIQUE REFERENCES users(id),
    -- The password for the user, hashed with the `salt`
    password_hash CHAR(64) NOT NULL,
    -- The salt of the user's password
    salt CHAR(32) NOT NULL
);
