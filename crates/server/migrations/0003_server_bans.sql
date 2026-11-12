-- A server ban refuses login, separate from the lock failed logins set.


ALTER TABLE users ADD COLUMN banned INTEGER NOT NULL DEFAULT 0; -- Refuses login on TRUE (bool)
