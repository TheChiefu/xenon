-- Initial schema.
--
-- Connection settings live in SqliteConnectOptions.
-- Timestamps are Unix milliseconds, set in Rust by now_ms().


-- Content addressed: sha256 is both the dedup key and the path on disk.
-- filename is the first uploader's name. mime is sniffed from the bytes.
CREATE TABLE files (
    id          BLOB PRIMARY KEY,
    sha256      BLOB NOT NULL UNIQUE,
    filename    TEXT NOT NULL,
    mime        TEXT NOT NULL,
    byte_size   INTEGER NOT NULL,
    created_at  INTEGER NOT NULL
) STRICT;

-- username is restricted to lowercase, so UNIQUE rejects 'Alice' while 'alice'
-- exists.
CREATE TABLE users (
    id              BLOB PRIMARY KEY,
    username        TEXT NOT NULL UNIQUE,
    display_name    TEXT NOT NULL,
    description     TEXT NOT NULL DEFAULT '',
    avatar_file_id  BLOB REFERENCES files(id),
    banner_file_id  BLOB REFERENCES files(id),
    password_hash   TEXT,
    global_role     TEXT NOT NULL,
    created_at      INTEGER NOT NULL,
    deleted_at      INTEGER
) STRICT;

CREATE UNIQUE INDEX one_owner ON users(global_role) WHERE global_role = 'owner' AND deleted_at IS NULL;

-- One row per user, created alongside them
CREATE TABLE user_preferences (
    user_id     BLOB PRIMARY KEY REFERENCES users(id) ON DELETE CASCADE,
    room_layout TEXT NOT NULL DEFAULT '[]'
) STRICT;

-- Not an auth path: identity and display only. No platform_user_id, no
-- credential columns, no uniqueness constraint over a platform id — this
-- table holds only "this user is linked to this platform, show this
-- handle for them," nothing more.
CREATE TABLE linked_accounts (
    user_id         BLOB NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    platform        TEXT NOT NULL,
    platform_handle TEXT NOT NULL,
    PRIMARY KEY (user_id, platform)
) STRICT;

CREATE TABLE sessions (
    token_hash  BLOB PRIMARY KEY,
    user_id     BLOB NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    created_at  INTEGER NOT NULL,
    expires_at  INTEGER NOT NULL,
    revoked_at  INTEGER
) STRICT;

CREATE INDEX sessions_user ON sessions(user_id);
CREATE INDEX sessions_expiry ON sessions(expires_at) WHERE revoked_at IS NULL;

-- default_permissions has no DEFAULT: creation must state it. The value is
-- copied into room_access.permissions when a member joins.
CREATE TABLE rooms (
    id                  BLOB PRIMARY KEY,
    name                TEXT NOT NULL,
    visibility          TEXT NOT NULL,
    default_permissions TEXT NOT NULL,
    created_at          INTEGER NOT NULL,
    -- Incremented on edit and tombstone
    mutation_seq        INTEGER NOT NULL DEFAULT 0
) STRICT;

-- The directory query:
-- Hidden rooms are absent from the index and id is a UUIDv7 (byte order is creation order)
-- of which the directory pages on with `id > ?`.
--
-- Queries must filter with `visibility IN ('public', 'locked')` to match this predicate.
CREATE INDEX rooms_directory ON rooms(id) WHERE visibility IN ('public', 'locked');

-- A row grants read access to the room
--
-- granted_at holds join time
CREATE TABLE room_access (
    room_id     BLOB NOT NULL REFERENCES rooms(id) ON DELETE CASCADE,
    user_id     BLOB NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    permissions TEXT NOT NULL,
    notify      TEXT NOT NULL DEFAULT 'none',
    granted_at  INTEGER NOT NULL,
    PRIMARY KEY (room_id, user_id)
) STRICT;

CREATE INDEX room_access_user ON room_access(user_id);

-- Pending invitations to Locked and Hidden rooms
-- A row means invited, not joined
CREATE TABLE room_invites (
    room_id     BLOB NOT NULL REFERENCES rooms(id) ON DELETE CASCADE,
    user_id     BLOB NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    invited_by  BLOB NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    created_at  INTEGER NOT NULL,
    expires_at  INTEGER,
    PRIMARY KEY (room_id, user_id)
) STRICT;

CREATE INDEX room_invites_user ON room_invites(user_id);

-- Room scoped bans:
-- created_by is a plain REFERENCES, never CASCADE (a cascade lifts every ban an issuer made)
CREATE TABLE room_bans (
    room_id     BLOB NOT NULL REFERENCES rooms(id) ON DELETE CASCADE,
    user_id     BLOB NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    created_by  BLOB REFERENCES users(id),
    reason      TEXT,
    created_at  INTEGER NOT NULL,
    expires_at  INTEGER,
    PRIMARY KEY (room_id, user_id)
) STRICT;

CREATE TABLE read_state (
    user_id       BLOB NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    room_id       BLOB NOT NULL REFERENCES rooms(id) ON DELETE CASCADE,
    last_read_seq INTEGER NOT NULL,
    PRIMARY KEY (user_id, room_id)
) STRICT;

CREATE TABLE messages (
    seq           INTEGER PRIMARY KEY AUTOINCREMENT,
    id            BLOB NOT NULL UNIQUE,
    room_id       BLOB NOT NULL REFERENCES rooms(id) ON DELETE CASCADE,
    author_id     BLOB NOT NULL REFERENCES users(id),
    body          TEXT,
    client_nonce  BLOB NOT NULL,
    created_at    INTEGER NOT NULL,
    edited_at     INTEGER,
    deleted_at    INTEGER
) STRICT;

CREATE INDEX messages_room ON messages(room_id, seq);
CREATE INDEX messages_room_live ON messages(room_id, seq) WHERE deleted_at IS NULL;
CREATE UNIQUE INDEX message_nonce ON messages(author_id, client_nonce);

CREATE TRIGGER messages_author_is_member BEFORE INSERT ON messages
WHEN NOT EXISTS (
    SELECT 1 FROM room_access a
    WHERE a.room_id = new.room_id AND a.user_id = new.author_id
)
BEGIN SELECT RAISE(ABORT, 'author is not a member of this room');
END;

CREATE TRIGGER messages_author_live BEFORE INSERT ON messages
WHEN (SELECT deleted_at FROM users WHERE id = new.author_id) IS NOT NULL
BEGIN SELECT RAISE(ABORT, 'author is deleted');
END;

CREATE VIRTUAL TABLE messages_fts USING fts5(
    body, content = 'messages', content_rowid = 'seq', tokenize = 'trigram'
);

CREATE TRIGGER messages_fts_insert AFTER INSERT ON messages BEGIN
    INSERT INTO messages_fts(rowid, body) VALUES (new.seq, new.body);
END;

CREATE TRIGGER messages_fts_delete AFTER DELETE ON messages BEGIN
    INSERT INTO messages_fts(messages_fts, rowid, body) VALUES ('delete', old.seq, old.body);
END;

CREATE TRIGGER messages_fts_update AFTER UPDATE OF body ON messages BEGIN
    INSERT INTO messages_fts(messages_fts, rowid, body) VALUES ('delete', old.seq, old.body);
    INSERT INTO messages_fts(rowid, body) VALUES (new.seq, new.body);
END;

-- ordinal is the display order
CREATE TABLE message_attachments (
    message_id  BLOB NOT NULL REFERENCES messages(id) ON DELETE CASCADE,
    file_id     BLOB NOT NULL REFERENCES files(id),
    ordinal     INTEGER NOT NULL,
    spoiler     INTEGER NOT NULL DEFAULT 0,
    PRIMARY KEY (message_id, ordinal),
    UNIQUE (message_id, file_id)
) STRICT;

CREATE INDEX message_attachments_file ON message_attachments(file_id);

-- A user's library of files
CREATE TABLE user_files (
    user_id   BLOB NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    file_id   BLOB NOT NULL REFERENCES files(id),
    added_at  INTEGER NOT NULL,
    PRIMARY KEY (user_id, file_id)
) STRICT;

CREATE INDEX user_files_file ON user_files(file_id);

-- Server registration codes
CREATE TABLE registration_codes (
    code        TEXT PRIMARY KEY,
    created_by  BLOB NOT NULL REFERENCES users(id),
    created_at  INTEGER NOT NULL,
    expires_at  INTEGER,
    max_uses    INTEGER,
    uses        INTEGER NOT NULL DEFAULT 0
) STRICT;

-- The VAPID public key browsers subscribe against. Browser subscriptions
-- themselves (push_subscriptions) are not Xenon's to keep: only the push
-- sidecar reads them, so they live in its own store instead.
CREATE TABLE push_keys (
    id          INTEGER PRIMARY KEY CHECK (id = 1),
    public_key  BLOB NOT NULL,
    created_at  INTEGER NOT NULL
) STRICT;