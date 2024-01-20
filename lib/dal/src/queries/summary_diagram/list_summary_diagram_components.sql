SELECT DISTINCT ON (id) row_to_json(sdc1.*) AS object
FROM summary_diagram_components AS sdc1
WHERE in_tenancy_v1($1, sdc1)
  AND ((sdc1.visibility_change_set_pk = $2
    AND (sdc1.visibility_deleted_at IS NULL OR
         EXISTS (SELECT 1
                 FROM summary_diagram_components AS sdc2
                 WHERE sdc2.component_id = sdc1.component_id
                   AND sdc2.visibility_change_set_pk = ident_nil_v1()
                   AND sdc2.visibility_deleted_at IS NULL))
           )
    OR sdc1.visibility_change_set_pk = ident_nil_v1() AND sdc1.visibility_deleted_at IS NULL)
ORDER BY sdc1.id, sdc1.visibility_change_set_pk DESC, sdc1.visibility_deleted_at DESC;