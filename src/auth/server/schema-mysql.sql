
CREATE TABLE users (
    id          INTEGER NOT NULL PRIMARY KEY,
    -- UNIQUE index requires a length, so use VARCHAR instead of TEXT here
    name        VARCHAR(64) NOT NULL,
    name_lower  VARCHAR(64) UNIQUE NOT NULL,
    password    TEXT NOT NULL,
    email       TEXT NOT NULL
);

CREATE TABLE counter (
    value       INTEGER NOT NULL
);

INSERT INTO counter (value) VALUES (0);

DELIMITER //
CREATE FUNCTION next_counter()
    RETURNS INTEGER
BEGIN
    DECLARE last INTEGER;
    SELECT value INTO @last FROM counter LIMIT 1;
    UPDATE counter SET value = @last + 1 LIMIT 1;
    RETURN @last;
END //
DELIMITER ;

