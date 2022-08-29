CREATE OR REPLACE FUNCTION attribute_value_list_payload_for_read_context_and_root_v1(this_tenancy jsonb,
                                                                                     this_visibility jsonb,
                                                                                     this_context jsonb,
                                                                                     this_attribute_value_id bigint)
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
    new_child_attribute_value_ids bigint[];
    parent_attribute_value_ids    bigint[];
BEGIN
    -- Make sure we return the result for the base AttributeValue before looping through
    -- to return all of its children.
    RETURN QUERY
        SELECT
            desired_attribute_values.parent_attribute_value_id,
            desired_attribute_values.attribute_value_object,
            desired_attribute_values.prop_object,
            desired_attribute_values.func_binding_return_value_object
        FROM (
            SELECT DISTINCT ON (attribute_values.id)
                attribute_values.id,
                attribute_values.visibility_change_set_pk,
                attribute_values.visibility_deleted_at,
                avbtav.belongs_to_id AS parent_attribute_value_id,
                row_to_json(attribute_values.*) AS attribute_value_object,
                row_to_json(props.*) AS prop_object,
                row_to_json(func_binding_return_values.*) AS func_binding_return_value_object
            FROM
                attribute_values
                LEFT JOIN attribute_value_belongs_to_attribute_value AS avbtav ON
                    attribute_values.id = avbtav.object_id
                    AND in_tenancy_and_visible_v1(this_tenancy, this_visibility, avbtav)
                INNER JOIN props ON
                    attribute_values.attribute_context_prop_id = props.id
                    AND in_tenancy_and_visible_v1(this_tenancy, this_visibility, props)
                INNER JOIN func_binding_return_values ON
                    func_binding_return_values.id = attribute_values.func_binding_return_value_id
                    AND in_tenancy_and_visible_v1(this_tenancy, this_visibility, func_binding_return_values)
            WHERE
                in_tenancy_and_visible_v1(this_tenancy, this_visibility, attribute_values)
                AND attribute_values.id = this_attribute_value_id
            ORDER BY
                attribute_values.id,
                visibility_change_set_pk DESC,
                visibility_deleted_at DESC NULLS FIRST
        ) AS desired_attribute_values;

    parent_attribute_value_ids := ARRAY [this_attribute_value_id];
    LOOP
        SELECT
            array_agg(attribute_value_id) AS attribute_value_ids
        INTO STRICT new_child_attribute_value_ids
        FROM (
            SELECT DISTINCT ON (
                avbtav.belongs_to_id,
                attribute_values.attribute_context_prop_id,
                COALESCE(avbtav.belongs_to_id, -1),
                COALESCE(attribute_values.key, '')
            )
                attribute_values.id AS attribute_value_id
            FROM
                attribute_values
                LEFT JOIN attribute_value_belongs_to_attribute_value AS avbtav ON
                    attribute_values.id = avbtav.object_id
                    AND in_tenancy_and_visible_v1(this_tenancy, this_visibility, avbtav)
            WHERE
                in_attribute_context_v1(this_context, attribute_values)
                AND avbtav.belongs_to_id = ANY(parent_attribute_value_ids)
            ORDER BY
                avbtav.belongs_to_id,
                attribute_values.attribute_context_prop_id,
                COALESCE(avbtav.belongs_to_id, -1),
                COALESCE(attribute_values.key, ''),
                attribute_values.visibility_change_set_pk DESC,
                attribute_values.visibility_deleted_at DESC NULLS FIRST,
                attribute_values.attribute_context_schema_id DESC,
                attribute_values.attribute_context_schema_variant_id DESC,
                attribute_values.attribute_context_component_id DESC,
                attribute_values.attribute_context_system_id DESC
        ) AS av_ids;

        -- Exit the loop, since we haven't found any new child AttributeValues to return.
        EXIT WHEN new_child_attribute_value_ids IS NULL;

        -- This returns a partial result for the AttributeValues that we've found so far.
        RETURN QUERY
            SELECT
                desired_attribute_values.parent_attribute_value_id,
                desired_attribute_values.attribute_value_object,
                desired_attribute_values.prop_object,
                desired_attribute_values.func_binding_return_value_object
            FROM (
                SELECT DISTINCT ON (attribute_values.id)
                    attribute_values.id,
                    attribute_values.visibility_change_set_pk,
                    attribute_values.visibility_deleted_at,
                    avbtav.belongs_to_id AS parent_attribute_value_id,
                    row_to_json(attribute_values.*) AS attribute_value_object,
                    row_to_json(props.*) AS prop_object,
                    row_to_json(func_binding_return_values.*) AS func_binding_return_value_object
                FROM
                    attribute_values
                    LEFT JOIN attribute_value_belongs_to_attribute_value AS avbtav ON
                        attribute_values.id = avbtav.object_id
                        AND in_tenancy_and_visible_v1(this_tenancy, this_visibility, avbtav)
                    INNER JOIN props ON
                        attribute_values.attribute_context_prop_id = props.id
                        AND in_tenancy_and_visible_v1(this_tenancy, this_visibility, props)
                    INNER JOIN func_binding_return_values ON
                        func_binding_return_values.id = attribute_values.func_binding_return_value_id
                        AND in_tenancy_and_visible_v1(this_tenancy, this_visibility, func_binding_return_values)
                WHERE
                    attribute_values.id = ANY(new_child_attribute_value_ids)
                    AND in_tenancy_and_visible_v1(this_tenancy, this_visibility, attribute_values)
                ORDER BY
                    attribute_values.id,
                    visibility_change_set_pk DESC,
                    visibility_deleted_at DESC NULLS FIRST
            ) AS desired_attribute_values;

        -- Prime parent_attribute_value_ids with the child IDs we just found, so
        -- we can look for their children.
        parent_attribute_value_ids := new_child_attribute_value_ids;
    END LOOP;
END;
$$ LANGUAGE PLPGSQL;
