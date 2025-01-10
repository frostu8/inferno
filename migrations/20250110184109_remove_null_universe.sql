-- the global universe is always id=0
ALTER TABLE universes
ALTER COLUMN host DROP NOT NULL;

-- update tables after migrating data
--ALTER TABLE pages
--ALTER COLUMN universe_id SET NOT NULL;

ALTER TABLE pages
DROP CONSTRAINT pages_unique_key;

ALTER TABLE pages
ADD CONSTRAINT pages_unique_key UNIQUE (path, universe_id);
