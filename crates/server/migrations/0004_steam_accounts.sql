CREATE TABLE steam_accounts (
    user_id       BLOB PRIMARY KEY REFERENCES users(id) ON DELETE CASCADE,
    steam_id      INTEGER NOT NULL,
    persona_name  TEXT NOT NULL
) STRICT;
