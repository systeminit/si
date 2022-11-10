SELECT DISTINCT ON (id) id,
                        row_to_json(code_generation_prototypes.*) AS object

FROM code_generation_prototypes_v1($1, $2) AS code_generation_prototypes
WHERE prop_id = $3

ORDER BY id,
         prop_id DESC,
         visibility_change_set_pk DESC,
         visibility_deleted_at DESC NULLS FIRST;
