CREATE TABLE users (
    id          INTEGER NOT NULL PRIMARY KEY,
    name        VARCHAR(64) NOT NULL,
    name_lower  VARCHAR(64) UNIQUE NOT NULL,
    password    TEXT NOT NULL,
    email       TEXT NOT NULL
);

CREATE TABLE counter (
    value       INTEGER NOT NULL
);

INSERT INTO counter (value) VALUES (1);
