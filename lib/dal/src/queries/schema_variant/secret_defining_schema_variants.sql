SELECT row_to_json(sv.*) AS object
FROM props_v1($1, $2) prop_definition
    JOIN schema_variants_v1($1, $2) sv on sv.id = prop_definition.schema_variant_id
WHERE prop_definition.path = 'rootsecret_definition'
