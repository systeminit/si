SELECT row_to_json(nodes.*) AS object
FROM nodes_v1($1, $2) AS nodes
         INNER JOIN node_belongs_to_component_v1($1, $2) AS bt ON bt.object_id = nodes.id
         INNER JOIN components_v1($1, $2) AS components ON components.id = bt.belongs_to_id
WHERE ((components.needs_destroy AND ($2 ->> 'visibility_change_set_pk')::ident = ident_nil_v1())
    OR nodes.visibility_deleted_at IS NULL)
  AND nodes.kind = $3

UNION ALL

SELECT row_to_json(nodes.*) AS object
FROM nodes_v1($1, $2) as nodes
         INNER JOIN node_belongs_to_component_v1($1, $2) AS bt ON bt.object_id = nodes.id
         INNER JOIN components_v1($1, $2) AS components ON components.id = bt.belongs_to_id
WHERE nodes.id IN (SELECT id
                   FROM nodes
                   WHERE visibility_change_set_pk = ident_nil_v1()
                     AND ($2 ->> 'visibility_change_set_pk')::ident != ident_nil_v1()
                     AND visibility_deleted_at IS NULL
                     AND in_tenancy_v1($1, tenancy_workspace_pk))
  AND NOT (components.needs_destroy AND ($2 ->> 'visibility_change_set_pk')::ident = ident_nil_v1())
  AND nodes.kind = $3
  AND nodes.visibility_change_set_pk = ($2 ->> 'visibility_change_set_pk')::ident
  AND nodes.visibility_deleted_at IS NOT NULL
  AND in_tenancy_v1($1, nodes.tenancy_workspace_pk)

