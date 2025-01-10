-- Add migration script here
CREATE TABLE universes (
    id SERIAL PRIMARY KEY,
    host VARCHAR(255) UNIQUE NOT NULL
);

ALTER TABLE pages
ADD COLUMN universe_id INTEGER REFERENCES universes(id);

ALTER TABLE pages
DROP CONSTRAINT pages_path_key;

ALTER TABLE pages
ADD CONSTRAINT pages_unique_key UNIQUE NULLS NOT DISTINCT (path, universe_id);
