CREATE OR REPLACE FUNCTION attribute_value_list_payload_for_read_context_v1(this_tenancy jsonb,
                                                                            this_visibility jsonb,
                                                                            this_context jsonb,
                                                                            this_prop_id bigint)
    RETURNS TABLE
            (
                parent_attribute_value_id        bigint,
                attribute_value_object           json,
                prop_object                      json,
                func_binding_return_value_object json
            )
AS
$$
DECLARE
    parent_attribute_value_id bigint;
BEGIN
    -- Grab the initial AttributeValueId based on the PropId we were given.
    SELECT DISTINCT ON (
        attribute_values.attribute_context_prop_id,
        COALESCE(avbtav.belongs_to_id, -1),
        COALESCE(attribute_values.key, '')
    )
        attribute_values.id AS attribute_value_id
    INTO STRICT parent_attribute_value_id
    FROM attribute_values
         LEFT JOIN attribute_value_belongs_to_attribute_value AS avbtav ON
             attribute_values.id = avbtav.object_id
             AND in_tenancy_and_visible_v1(this_tenancy, this_visibility, avbtav)
         INNER JOIN prop_many_to_many_schema_variants AS pmtmsv ON
             attribute_values.attribute_context_prop_id = pmtmsv.left_object_id
             AND in_tenancy_and_visible_v1(this_tenancy, this_visibility, pmtmsv)
    WHERE
        in_tenancy_and_visible_v1(this_tenancy, this_visibility, attribute_values)
        AND in_attribute_context_v1(this_context, attribute_values)
        AND pmtmsv.right_object_id = this_prop_id
    ORDER BY
        attribute_values.attribute_context_prop_id,
        COALESCE(avbtav.belongs_to_id, -1),
        COALESCE(attribute_values.key, ''),
        attribute_values.visibility_change_set_pk DESC,
        attribute_values.visibility_deleted_at DESC NULLS FIRST,
        attribute_values.attribute_context_internal_provider_id DESC,
        attribute_values.attribute_context_external_provider_id DESC,
        attribute_values.attribute_context_schema_id DESC,
        attribute_values.attribute_context_schema_variant_id DESC,
        attribute_values.attribute_context_component_id DESC,
        attribute_values.attribute_context_system_id DESC;

    RETURN QUERY SELECT *
                 FROM attribute_value_list_payload_for_read_context_and_root_v1(this_tenancy, this_visibility, this_context, parent_attribute_value_id);
END;
$$ LANGUAGE PLPGSQL;
