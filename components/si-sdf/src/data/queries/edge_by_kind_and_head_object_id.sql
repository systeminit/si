-- "SELECT a.*
--            FROM `{bucket}` AS a
--           WHERE a.siStorable.typeName = \"edge\"
--             AND a.kind = \"{edge_kind}\"
--             AND a.headVertex.objectId = \"{object_id}\"
SELECT obj AS object
FROM edges
WHERE kind = $1
AND head_vertex_object_si_id = $2