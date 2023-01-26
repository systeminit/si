SELECT row_to_json(o.*) AS object
FROM organizations AS o
WHERE pk = $1 AND visibility_deleted_at is NULL
