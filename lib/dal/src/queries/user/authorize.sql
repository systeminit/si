SELECT
    true AS authorized,
    users.id,
    users.visibility_change_set_pk
FROM users_v1($1, $2) AS users
INNER JOIN group_many_to_many_users_v1($1, $2) AS group_many_to_many_users
    ON users.id = group_many_to_many_users.right_object_id
        AND group_many_to_many_users.visibility_deleted_at IS NULL
INNER JOIN capability_belongs_to_group_v1($1, $2) AS capability_belongs_to_group
    ON capability_belongs_to_group.belongs_to_id = group_many_to_many_users.left_object_id
        AND capability_belongs_to_group.visibility_deleted_at IS NULL
INNER JOIN capabilities_v1($1, $2) AS capabilities
    ON capabilities.id = capability_belongs_to_group.object_id
        AND capabilities.visibility_deleted_at IS NULL
        AND capabilities.subject = 'any'
        AND capabilities.action = 'any'
WHERE users.id = $3
LIMIT 1;
