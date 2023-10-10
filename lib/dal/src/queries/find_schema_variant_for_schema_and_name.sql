SELECT row_to_json(sv.*) AS object FROM schema_variants_v1($1, $2) sv
  JOIN schema_variant_belongs_to_schema_v1($1, $2) svbts
    ON svbts.belongs_to_id = $3
WHERE sv.id = svbts.object_id AND sv.name = $4 LIMIT 1
