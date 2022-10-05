SELECT DISTINCT ON (qualification_prototypes.id) qualification_prototypes.id,
                                                 qualification_prototypes.component_id,
                                                 qualification_prototypes.schema_id,
                                                 qualification_prototypes.schema_variant_id,
                                                 qualification_prototypes.system_id,
                                                 qualification_prototypes.visibility_change_set_pk,
                                                 row_to_json(qualification_prototypes.*) AS object
FROM qualification_prototypes
WHERE in_tenancy_and_visible_v1($1, $2, qualification_prototypes)
  AND func_id = $3
ORDER BY qualification_prototypes.id,
         visibility_change_set_pk DESC,
         component_id DESC,
         func_id DESC,
         system_id DESC,
         schema_variant_id DESC,
         schema_id DESC;
