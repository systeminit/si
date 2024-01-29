WITH recursive parent_funcs AS (
  SELECT
    funcs.name AS func_name,
    funcs.id AS func_id,
    av.id AS attribute_value_id,
    array[]::ident[] AS parent_av_ids
  FROM component_belongs_to_schema_variant_v1($1, $2) AS cbtsv
    INNER JOIN schema_variants_v1($1, $2) AS sv
      ON cbtsv.belongs_to_id = sv.id
    INNER JOIN attribute_values_v1($1, $2) AS av
      ON sv.root_prop_id = av.attribute_context_prop_id
    INNER JOIN attribute_value_belongs_to_attribute_prototype_v1($1, $2) AS avbtap
      ON av.id = avbtap.object_id
    INNER JOIN attribute_prototypes_v1($1, $2) AS ap
      ON avbtap.belongs_to_id = ap.id
    INNER JOIN funcs_v1($1, $2) AS funcs
      ON ap.func_id = funcs.id
  WHERE cbtsv.object_id = $3
    AND av.attribute_context_component_id = $3

  UNION ALL

  SELECT
    funcs.name AS func_name,
    funcs.id AS func_id,
    avbtav.object_id AS attribute_value_id,
    array_append(parent_av_ids, avbtav.belongs_to_id) AS parent_av_ids
  FROM parent_funcs
    INNER JOIN attribute_value_belongs_to_attribute_value_v1($1, $2) AS avbtav
      ON parent_funcs.attribute_value_id = avbtav.belongs_to_id
    INNER JOIN attribute_value_belongs_to_attribute_prototype_v1($1, $2) AS avbtap
      ON avbtav.object_id = avbtap.object_id
    INNER JOIN attribute_prototypes_v1($1, $2) AS ap
      ON avbtap.belongs_to_id = ap.id
    INNER JOIN funcs_v1($1, $2) AS funcs
      ON ap.func_id = funcs.id
)
SELECT
  row_to_json(parent_funcs.*) AS object
FROM parent_funcs;
