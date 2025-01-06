-- Add migration script here
CREATE TABLE links (
    -- The page id the link is from
    source_id INTEGER NOT NULL REFERENCES pages(id),
    -- The destination path of the link
    dest_path VARCHAR(255) NOT NULL,

    UNIQUE (source_id, dest_path)
);
