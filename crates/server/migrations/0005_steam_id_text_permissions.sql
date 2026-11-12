-- SQLite cannot change a column type, so the table is rebuilt
CREATE TABLE steam_accounts_new (
    user_id       BLOB PRIMARY KEY REFERENCES users(id) ON DELETE CASCADE,
    steam_id      TEXT NOT NULL,
    persona_name  TEXT NOT NULL
) STRICT;

INSERT INTO steam_accounts_new
SELECT user_id, CAST(steam_id AS TEXT), persona_name FROM steam_accounts;

DROP TABLE steam_accounts;
ALTER TABLE steam_accounts_new RENAME TO steam_accounts;

-- Remove permissions that no longer exist
UPDATE rooms SET default_permissions = (
    SELECT json_group_array(value)
    FROM json_each(rooms.default_permissions)
    WHERE value NOT IN ('commands', 'screenshare')
);

UPDATE room_access SET permissions = (
    SELECT json_group_array(value)
    FROM json_each(room_access.permissions)
    WHERE value NOT IN ('commands', 'screenshare')
);
