
CREATE TABLE users (
    id          SERIAL PRIMARY KEY,
    -- UNIQUE index requires a length, so use VARCHAR instead of TEXT here
    name        VARCHAR(64) NOT NULL,
    name_lower  VARCHAR(64) UNIQUE NOT NULL,
    password    TEXT NOT NULL,
    email       TEXT NOT NULL
);
