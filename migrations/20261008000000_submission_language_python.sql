-- SQLite cannot alter a CHECK constraint, so replace the column to allow Python.
ALTER TABLE submissions ADD COLUMN language_next TEXT NOT NULL DEFAULT 'cpp'
    CHECK (language_next IN ('cpp', 'rust', 'python'));
UPDATE submissions SET language_next = language;
ALTER TABLE submissions DROP COLUMN language;
ALTER TABLE submissions RENAME COLUMN language_next TO language;
