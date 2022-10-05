SELECT DISTINCT ON (code_generation_prototypes.id) code_generation_prototypes.id,
                                                   code_generation_prototypes.component_id,
                                                   code_generation_prototypes.schema_id,
                                                   code_generation_prototypes.schema_variant_id,
                                                   code_generation_prototypes.system_id,
                                                   code_generation_prototypes.visibility_change_set_pk,

                                                   row_to_json(code_generation_prototypes.*) AS object
FROM code_generation_prototypes
WHERE in_tenancy_and_visible_v1($1, $2, code_generation_prototypes)
  AND func_id = $3
ORDER BY code_generation_prototypes.id,
         visibility_change_set_pk DESC,
         component_id DESC,
         func_id DESC,
         system_id DESC,
         schema_variant_id DESC,
         schema_id DESC;
