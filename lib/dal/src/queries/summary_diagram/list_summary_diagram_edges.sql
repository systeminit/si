SELECT DISTINCT ON (id) row_to_json(sde1.*) AS object
FROM summary_diagram_edges AS sde1
WHERE in_tenancy_v1($1, sde1)
  AND ((sde1.visibility_change_set_pk = $2
    AND (sde1.visibility_deleted_at IS NULL OR
         EXISTS (SELECT 1
                 FROM summary_diagram_edges AS sde2
                 WHERE sde2.edge_id = sde1.edge_id
                   AND sde2.visibility_change_set_pk = ident_nil_v1()
                   AND sde2.visibility_deleted_at IS NULL))
           )
    OR sde1.visibility_change_set_pk = ident_nil_v1() AND sde1.visibility_deleted_at IS NULL)
ORDER BY sde1.id, sde1.visibility_change_set_pk DESC, sde1.visibility_deleted_at DESC;