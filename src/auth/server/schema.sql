
CREATE TABLE users (
    id          SERIAL PRIMARY KEY,
    name        TEXT NOT NULL,
    name_lower  TEXT UNIQUE NOT NULL,
    password    TEXT NOT NULL,
    email       TEXT NOT NULL
);
