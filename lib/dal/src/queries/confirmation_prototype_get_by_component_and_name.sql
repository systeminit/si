SELECT DISTINCT ON (confirmation_prototypes.id) confirmation_prototypes.id,
                                                confirmation_prototypes.component_id,
                                                confirmation_prototypes.name,
                                                confirmation_prototypes.schema_id,
                                                confirmation_prototypes.schema_variant_id,
                                                confirmation_prototypes.system_id,
                                                confirmation_prototypes.visibility_change_set_pk,
                                                row_to_json(confirmation_prototypes.*) AS object

FROM confirmation_prototypes
WHERE in_tenancy_and_visible_v1($1, $2, confirmation_prototypes)
  AND (confirmation_prototypes.component_id = $3
    OR confirmation_prototypes.schema_id = $5
    OR confirmation_prototypes.schema_variant_id = $6)
  AND confirmation_prototypes.name = $4
  AND (confirmation_prototypes.system_id = $7 OR confirmation_prototypes.system_id = -1)

ORDER BY id,
         component_id DESC,
         name DESC,
         system_id DESC,
         schema_variant_id DESC,
         schema_id DESC,
         visibility_change_set_pk DESC,
         visibility_deleted_at DESC NULLS FIRST;
