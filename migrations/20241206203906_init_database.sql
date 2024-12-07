-- Add migration script here
-- Pages
CREATE TABLE pages (
    -- page id, used to reference the page
    id SERIAL PRIMARY KEY,
    -- page path, last component uniquely identifies the page as a title
    path VARCHAR(255) UNIQUE NOT NULL,
    -- current page content
    content TEXT NOT NULL,
    -- timestamps
    inserted_at TIMESTAMP NOT NULL,
    updated_at TIMESTAMP NOT NULL
);

-- Users: Editors
-- Stores no login information.
CREATE TABLE users (
    id SERIAL PRIMARY KEY,
    -- A username that uniquely identifies the user.
    username VARCHAR(255) UNIQUE NOT NULL,
    -- timestamps
    inserted_at TIMESTAMP NOT NULL,
    updated_at TIMESTAMP NOT NULL
);

-- Page changes
CREATE TABLE changes (
    -- On what page the change happened
    page_id INTEGER NOT NULL REFERENCES pages(id),
    -- The author that made the change
    author_id INTEGER NOT NULL REFERENCES users(id),
    -- Hash of the changes
    hash CHAR(64) UNIQUE NOT NULL,
    -- The actual content of the changes
    content TEXT NOT NULL,
    -- timestamps
    -- no `updated_at` as this is an immutable record
    inserted_at TIMESTAMP NOT NULL
);
