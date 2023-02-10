CREATE OR REPLACE FUNCTION attribute_value_list_payload_for_read_context_and_root_v1(
    this_tenancy jsonb,
    this_visibility jsonb,
    this_context jsonb,
    this_attribute_value_id ident
)
    RETURNS TABLE
            (
                parent_attribute_value_id        ident,
                attribute_value_object           json,
                prop_object                      json,
                func_binding_return_value_object json
            )
AS
$$
DECLARE
    new_child_attribute_value_ids ident[];
    parent_attribute_value_ids    ident[];
BEGIN
    -- Make sure we return the result for the base AttributeValue before looping through
    -- to return all of its children.
    RETURN QUERY
        SELECT avbtav.belongs_to_id AS parent_attribute_value_id,
               row_to_json(av.*)    AS attribute_value_object,
               row_to_json(prop.*)  AS prop_object,
               row_to_json(fbrv.*)  AS func_binding_return_value_object
        FROM attribute_values_v1(this_tenancy, this_visibility) AS av
                 LEFT JOIN attribute_value_belongs_to_attribute_value_v1(this_tenancy, this_visibility) AS avbtav
                           ON av.id = avbtav.object_id
                 INNER JOIN props_v1(this_tenancy, this_visibility) AS prop
                            ON av.attribute_context_prop_id = prop.id
                 INNER JOIN func_binding_return_values_v1(this_tenancy, this_visibility) AS fbrv
                            ON fbrv.id = av.func_binding_return_value_id
        WHERE av.id = this_attribute_value_id
        ORDER BY av.id;

    parent_attribute_value_ids := ARRAY [ this_attribute_value_id ];

    LOOP
        SELECT array_agg(attribute_value_id) AS attribute_value_ids
        INTO STRICT new_child_attribute_value_ids
        FROM (
                 SELECT DISTINCT ON (
                     COALESCE(avbtav.belongs_to_id, ident_nil_v1()),
                     av.attribute_context_prop_id,
                     COALESCE(av.key, '')
                     ) av.id AS attribute_value_id
                 FROM attribute_values_v1(this_tenancy, this_visibility) AS av
                          LEFT JOIN attribute_value_belongs_to_attribute_value_v1(this_tenancy,
                                                                                  this_visibility) AS avbtav
                                    ON av.id = avbtav.object_id
                 WHERE in_attribute_context_v1(this_context, av)
                   AND avbtav.belongs_to_id = ANY (parent_attribute_value_ids)
                 ORDER BY COALESCE(avbtav.belongs_to_id, ident_nil_v1()) DESC,
                          av.attribute_context_prop_id DESC,
                          COALESCE(av.key, ''),
                          av.attribute_context_component_id DESC
             ) AS av_ids;
        -- Exit the loop, since we haven't found any new child AttributeValues to return.
        EXIT WHEN new_child_attribute_value_ids IS NULL;

        -- This returns a partial result for the AttributeValues that we've found so far.
        RETURN QUERY
            SELECT avbtav.belongs_to_id AS parent_attribute_value_id,
                   row_to_json(av.*)    AS attribute_value_object,
                   row_to_json(prop.*)  AS prop_object,
                   row_to_json(fbrv.*)  AS func_binding_return_value_object
            FROM attribute_values_v1(this_tenancy, this_visibility) AS av
                     LEFT JOIN attribute_value_belongs_to_attribute_value_v1(this_tenancy, this_visibility) AS avbtav
                               ON av.id = avbtav.object_id
                     INNER JOIN props_v1(this_tenancy, this_visibility) AS prop
                                ON av.attribute_context_prop_id = prop.id
                     INNER JOIN func_binding_return_values_v1(this_tenancy, this_visibility) AS fbrv
                                ON fbrv.id = av.func_binding_return_value_id
            WHERE av.id = ANY (new_child_attribute_value_ids)
            ORDER BY av.id;


        -- Prime parent_attribute_value_ids with the child IDs we just found, so
        -- we can look for their children.
        --        RAISE LOG 'parent_attribute_value_ids: %',parent_attribute_value_ids;
        --        RAISE LOG 'new_child_attribute_value_ids: %',new_child_attribute_value_ids;
        parent_attribute_value_ids := new_child_attribute_value_ids;
    END LOOP;
END;
$$ LANGUAGE PLPGSQL;

CREATE OR REPLACE FUNCTION attribute_value_id_for_prop_and_context_v1(
    this_tenancy jsonb,
    this_visibility jsonb,
    this_context jsonb,
    this_prop_id ident
)
    RETURNS ident
    LANGUAGE SQL
    STABLE
    PARALLEL SAFE
AS
$$
SELECT DISTINCT ON (
    av.attribute_context_prop_id,
    COALESCE(avbtav.belongs_to_id, ident_nil_v1()),
    COALESCE(av.key, '')
    ) av.id
FROM attribute_values_v1(this_tenancy, this_visibility) AS av
         LEFT JOIN attribute_value_belongs_to_attribute_value_v1(this_tenancy, this_visibility) AS avbtav
                   ON av.id = avbtav.object_id
         INNER JOIN prop_many_to_many_schema_variants_v1(this_tenancy, this_visibility) AS pmtmsv
                    ON av.attribute_context_prop_id = pmtmsv.left_object_id
WHERE in_attribute_context_v1(this_context, av)
  AND pmtmsv.right_object_id = this_prop_id
ORDER BY av.attribute_context_prop_id,
         COALESCE(avbtav.belongs_to_id, ident_nil_v1()),
         COALESCE(av.key, ''),
         av.visibility_change_set_pk DESC,
         av.visibility_deleted_at DESC NULLS FIRST,
         av.attribute_context_internal_provider_id DESC,
         av.attribute_context_external_provider_id DESC,
         av.attribute_context_component_id DESC
$$;

CREATE OR REPLACE FUNCTION attribute_value_list_payload_for_read_context_v1(
    this_tenancy jsonb,
    this_visibility jsonb,
    this_context jsonb,
    this_prop_id ident
)
    RETURNS TABLE
            (
                parent_attribute_value_id        ident,
                attribute_value_object           json,
                prop_object                      json,
                func_binding_return_value_object json
            )
    LANGUAGE SQL
    STABLE
    PARALLEL SAFE
AS
$$
SELECT *
FROM attribute_value_list_payload_for_read_context_and_root_v1(
        this_tenancy,
        this_visibility,
        this_context,
        attribute_value_id_for_prop_and_context_v1(
                this_tenancy,
                this_visibility,
                this_context,
                this_prop_id
            )
    )
$$;
