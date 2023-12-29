-- Add migration script here
PRAGMA foreign_keys = ON;
CREATE TABLE IF NOT EXISTS sessions (
    `id` TEXT PRIMARY KEY,
    `value` INTEGER NOT NULL
);