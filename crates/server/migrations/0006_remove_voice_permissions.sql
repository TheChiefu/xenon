-- Remove permissions that no longer exist
UPDATE rooms SET default_permissions = (
    SELECT json_group_array(value)
    FROM json_each(rooms.default_permissions)
    WHERE value NOT IN ('connect', 'speak', 'video')
);

UPDATE room_access SET permissions = (
    SELECT json_group_array(value)
    FROM json_each(room_access.permissions)
    WHERE value NOT IN ('connect', 'speak', 'video')
);
