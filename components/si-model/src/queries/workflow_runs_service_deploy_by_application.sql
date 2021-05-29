SELECT obj AS object
FROM workflow_runs
WHERE obj -> 'ctx' -> 'entity' ->> 'id' IN (
    SELECT head_vertex_object_si_id
    FROM edges
    WHERE tail_vertex_object_si_id = $1
      AND kind = 'includes'
      and head_vertex_object_type = 'service'
) AND obj -> 'data' ->> 'name' = 'service:deploy';