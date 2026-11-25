-- Bans and invites hold only who and where: no expiry, time, sender, or reason.


-- Expired rows would become permanent once their expiry is dropped
DELETE FROM room_invites WHERE expires_at <= CAST(unixepoch('subsec') * 1000 AS INTEGER);
DELETE FROM room_bans WHERE expires_at <= CAST(unixepoch('subsec') * 1000 AS INTEGER);

ALTER TABLE room_invites DROP COLUMN expires_at;
ALTER TABLE room_invites DROP COLUMN created_at;
ALTER TABLE room_invites DROP COLUMN invited_by;
ALTER TABLE room_bans DROP COLUMN expires_at;
ALTER TABLE room_bans DROP COLUMN created_at;
ALTER TABLE room_bans DROP COLUMN reason;
