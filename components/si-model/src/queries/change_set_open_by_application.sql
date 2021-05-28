SELECT obj AS object
FROM change_sets
WHERE id IN (
    SELECT DISTINCT change_set_id
    FROM entities_change_set_projection
    WHERE obj ->> 'id' IN (
        SELECT head_vertex_object_si_id
        FROM edges
        WHERE tail_vertex_object_si_id = $1
          AND kind = 'includes'
    )
)
  AND obj ->> 'status' = 'open';