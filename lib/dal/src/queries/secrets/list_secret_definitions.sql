SELECT json_build_object(
               'secret_definition', prop_secret.name,
               'form_data', json_agg(
                       json_build_object(
                               'name', fields.name,
                               'kind', fields.kind,
                               'widget_kind', json_build_object(
                                       'kind', fields.widget_kind,
                                       'options', fields.widget_options))
                   )) AS object
FROM props_v1($1, $2) prop_definition
         JOIN prop_belongs_to_prop_v1($1, $2) pbtp ON prop_definition.id = pbtp.belongs_to_id
         JOIN props_v1($1, $2) fields ON fields.id = pbtp.object_id

         JOIN props_v1($1, $2) prop_secret ON
            prop_definition.schema_variant_id = prop_secret.schema_variant_id
        AND prop_secret.path LIKE 'rootsecrets%'
WHERE prop_definition.path = 'rootsecret_definition'
GROUP BY prop_definition.id, prop_secret.name;
