-- Client addresses refused before any route runs
-- IPv6 addresses are stored as their /64 network
CREATE TABLE blocked_addresses (
    address     TEXT PRIMARY KEY,
    reason      TEXT NOT NULL,
    created_at  INTEGER NOT NULL
) STRICT;
