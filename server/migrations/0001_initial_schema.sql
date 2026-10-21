-- Initial schema.
--
-- Connection settings live in SqliteConnectOptions.
-- Timestamps are Unix milliseconds, set in Rust by now_ms().


--- Tables ---


-- GENERIC --
CREATE TABLE files (
    seq         INTEGER PRIMARY KEY,
    id          BLOB NOT NULL UNIQUE,
    sha256      BLOB NOT NULL UNIQUE,   -- used as dedupe key (ie. multiple uploads of same file)
    filename    TEXT NOT NULL,          -- given by first upload (deduped)
    mime        TEXT NOT NULL,          -- sniffed from the bytes
    byte_size   INTEGER NOT NULL,
    created_at  INTEGER NOT NULL
) STRICT;

CREATE TABLE registration_codes (
    code        TEXT PRIMARY KEY,
    created_at  INTEGER NOT NULL,
    expires_at  INTEGER,
    max_uses    INTEGER,
    uses        INTEGER NOT NULL DEFAULT 0
) STRICT;

-- Client addresses refused before any route runs
-- IPv6 addresses are stored as their /64 network
CREATE TABLE blocked_addresses (
    address     TEXT PRIMARY KEY,
    reason      TEXT NOT NULL,
    created_at  INTEGER NOT NULL
) STRICT;

-- USER --
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
    locked          INTEGER NOT NULL DEFAULT 0 -- Refuses new login on TRUE (bool)
) STRICT;

CREATE TABLE user_preferences (
    user_id     BLOB PRIMARY KEY REFERENCES users(id) ON DELETE CASCADE,
    room_layout TEXT NOT NULL DEFAULT '[]'
) STRICT;

-- Personal file storage for reuse (without new uploads)
CREATE TABLE user_files (
    user_id   BLOB NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    file_id   BLOB NOT NULL REFERENCES files(id),
    added_at  INTEGER NOT NULL,
    PRIMARY KEY (user_id, file_id)
) STRICT;

CREATE TABLE sessions (
    token_hash  BLOB PRIMARY KEY,
    user_id     BLOB NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    created_at  INTEGER NOT NULL,
    expires_at  INTEGER NOT NULL,
    revoked_at  INTEGER
) STRICT;


-- ROOMS --
CREATE TABLE rooms (
    id                  BLOB PRIMARY KEY,
    name                TEXT NOT NULL,
    visibility          TEXT NOT NULL,
    default_permissions TEXT NOT NULL,
    created_at          INTEGER NOT NULL
) STRICT;

-- Read access to the room
CREATE TABLE room_access (
    room_id     BLOB NOT NULL REFERENCES rooms(id) ON DELETE CASCADE,
    user_id     BLOB NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    permissions TEXT NOT NULL,
    notify      TEXT NOT NULL DEFAULT 'all',
    granted_at  INTEGER NOT NULL,
    PRIMARY KEY (room_id, user_id)
) STRICT;

-- Pending invitations to rooms
CREATE TABLE room_invites (
    room_id     BLOB NOT NULL REFERENCES rooms(id) ON DELETE CASCADE,
    user_id     BLOB NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    invited_by  BLOB NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    created_at  INTEGER NOT NULL,
    expires_at  INTEGER,
    PRIMARY KEY (room_id, user_id)
) STRICT;

CREATE TABLE room_bans (
    room_id     BLOB NOT NULL REFERENCES rooms(id) ON DELETE CASCADE,
    user_id     BLOB NOT NULL REFERENCES users(id) ON DELETE CASCADE,
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


-- MESSAGES --
CREATE TABLE messages (
    seq           INTEGER PRIMARY KEY AUTOINCREMENT,
    id            BLOB NOT NULL UNIQUE,
    room_id       BLOB NOT NULL REFERENCES rooms(id) ON DELETE CASCADE,
    author_id     BLOB REFERENCES users(id) ON DELETE SET NULL,
    body          TEXT,
    client_nonce  BLOB NOT NULL,
    created_at    INTEGER NOT NULL,
    edited_at     INTEGER
) STRICT;

CREATE TABLE message_attachments (
    message_id  BLOB NOT NULL REFERENCES messages(id) ON DELETE CASCADE,
    file_id     BLOB NOT NULL REFERENCES files(id),
    ordinal     INTEGER NOT NULL, -- the display order
    spoiler     INTEGER NOT NULL DEFAULT 0,
    PRIMARY KEY (message_id, ordinal),
    UNIQUE (message_id, file_id)
) STRICT;

-- SERVICES --
CREATE TABLE push_keys (
    id          INTEGER PRIMARY KEY CHECK (id = 1),
    public_key  BLOB NOT NULL,
    private_key BLOB NOT NULL,
    created_at  INTEGER NOT NULL
) STRICT;

CREATE TABLE push_subscriptions (
    endpoint    TEXT PRIMARY KEY,
    user_id     BLOB NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    p256dh      BLOB NOT NULL,
    auth        BLOB NOT NULL
) STRICT;

CREATE TABLE xbox_accounts (
    user_id       BLOB PRIMARY KEY REFERENCES users(id) ON DELETE CASCADE,
    xuid          TEXT NOT NULL,
    gamertag      TEXT NOT NULL,
    refresh_token TEXT NOT NULL,
    honor_cloaked INTEGER NOT NULL DEFAULT 1
) STRICT;


-- INDEXES --
CREATE INDEX room_access_user ON room_access(user_id);
CREATE INDEX messages_room ON messages(room_id, seq);
CREATE UNIQUE INDEX message_nonce ON messages(author_id, client_nonce);

-- Finds an author's messages newest first (seq is the rowid, so already in order)
CREATE INDEX messages_author ON messages(author_id);

-- Finds the messages a file is attached to
CREATE INDEX message_attachments_file ON message_attachments(file_id);

-- Finds the users who have a file in their library
CREATE INDEX user_files_file ON user_files(file_id);


-- TRIGGERS --
CREATE TRIGGER messages_author_is_member BEFORE INSERT ON messages
WHEN NOT EXISTS (
    SELECT 1 FROM room_access a
    WHERE a.room_id = new.room_id AND a.user_id = new.author_id
)
BEGIN SELECT RAISE(ABORT, 'author is not a member of this room');
END;


-- SEARCH --

-- Message bodies
CREATE VIRTUAL TABLE messages_fts USING fts5(
    body, content = 'messages', content_rowid = 'seq', tokenize = 'trigram'
);

-- Removes deleted text from the index immediately
INSERT INTO messages_fts(messages_fts, rank) VALUES ('secure-delete', 1);

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

-- File names
CREATE VIRTUAL TABLE files_fts USING fts5(
    filename, content = 'files', content_rowid = 'seq', tokenize = 'trigram'
);

-- Removes deleted names from the index immediately
INSERT INTO files_fts(files_fts, rank) VALUES ('secure-delete', 1);

CREATE TRIGGER files_fts_insert AFTER INSERT ON files BEGIN
    INSERT INTO files_fts(rowid, filename) VALUES (new.seq, new.filename);
END;

CREATE TRIGGER files_fts_delete AFTER DELETE ON files BEGIN
    INSERT INTO files_fts(files_fts, rowid, filename) VALUES ('delete', old.seq, old.filename);
END;

CREATE TRIGGER files_fts_update AFTER UPDATE OF filename ON files BEGIN
    INSERT INTO files_fts(files_fts, rowid, filename) VALUES ('delete', old.seq, old.filename);
    INSERT INTO files_fts(rowid, filename) VALUES (new.seq, new.filename);
END;
