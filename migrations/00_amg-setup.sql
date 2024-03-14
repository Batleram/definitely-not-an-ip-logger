-- Add migration script here
CREATE table user_visits (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    user INTEGER NOT NULL
);
