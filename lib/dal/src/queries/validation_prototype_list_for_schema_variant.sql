SELECT DISTINCT ON (validation_prototypes.id) validation_prototypes.id,
                                              validation_prototypes.visibility_change_set_pk,
                                              validation_prototypes.visibility_deleted_at,
                                              row_to_json(validation_prototypes.*) AS object

FROM validation_prototypes
         INNER JOIN props
                    ON props.id = validation_prototypes.prop_id
                        AND in_tenancy_and_visible_v1($1, $2, props)
                        AND props.id IN (
                            WITH RECURSIVE recursive_props AS (
                                SELECT left_object_id AS prop_id
                                FROM prop_many_to_many_schema_variants
                                WHERE right_object_id = $3
                                UNION ALL
                                SELECT pbp.object_id AS prop_id
                                FROM prop_belongs_to_prop AS pbp
                                         JOIN recursive_props ON pbp.belongs_to_id = recursive_props.prop_id
                            )
                            SELECT prop_id
                            FROM recursive_props)

WHERE in_tenancy_and_visible_v1($1, $2, validation_prototypes)
  AND validation_prototypes.system_id = $4

ORDER BY validation_prototypes.id,
         validation_prototypes.visibility_change_set_pk DESC,
         validation_prototypes.visibility_deleted_at DESC NULLS FIRST;


