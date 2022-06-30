CREATE OR REPLACE FUNCTION attribute_value_list_payload_for_read_context_v1(this_tenancy jsonb,
                                                                            this_visibility jsonb,
                                                                            this_context jsonb,
                                                                            this_prop_id bigint)
RETURNS TABLE(
    parent_attribute_value_id bigint,
    attribute_value_object json,
    prop_object json,
    func_binding_return_value_object json
)
AS
$$
DECLARE
    new_child_attribute_value_ids bigint[];
    parent_attribute_value_ids bigint[];
BEGIN
    -- Grab the initial AttributeValueId based on the PropId we were given.
    EXECUTE
        'SELECT' ||E'\n'||
        '    array_agg(attribute_value_id) AS attribute_value_ids' ||E'\n'||
        'FROM (' ||E'\n'||
        '    SELECT DISTINCT ON (' ||E'\n'||
        '        attribute_values.attribute_context_prop_id,' ||E'\n'||
        '        COALESCE(avbtav.belongs_to_id, -1),' ||E'\n'||
        '        COALESCE(attribute_values.key, '''')' ||E'\n'||
        '    )' ||E'\n'||
        '        attribute_values.id AS attribute_value_id' ||E'\n'||
        '    FROM' ||E'\n'||
        '        attribute_values' ||E'\n'||
        '        LEFT JOIN attribute_value_belongs_to_attribute_value AS avbtav ON' ||E'\n'||
        '            attribute_values.id = avbtav.object_id' ||E'\n'||
        '            AND in_tenancy_v1($1, avbtav.tenancy_universal,' ||E'\n'||
        '                                  avbtav.tenancy_billing_account_ids,' ||E'\n'||
        '                                  avbtav.tenancy_organization_ids,' ||E'\n'||
        '                                  avbtav.tenancy_workspace_ids)' ||E'\n'||
        '            AND is_visible_v1($2, avbtav.visibility_change_set_pk,' ||E'\n'||
        '                                  avbtav.visibility_edit_session_pk,' ||E'\n'||
        '                                  avbtav.visibility_deleted_at)' ||E'\n'||
        '        INNER JOIN prop_many_to_many_schema_variants AS pmtmsv ON' ||E'\n'||
        '            attribute_values.attribute_context_prop_id = pmtmsv.left_object_id' ||E'\n'||
        '            AND in_tenancy_v1($1, pmtmsv.tenancy_universal,' ||E'\n'||
        '                                  pmtmsv.tenancy_billing_account_ids,' ||E'\n'||
        '                                  pmtmsv.tenancy_organization_ids,' ||E'\n'||
        '                                  pmtmsv.tenancy_workspace_ids)' ||E'\n'||
        '            AND is_visible_v1($2, pmtmsv.visibility_change_set_pk,' ||E'\n'||
        '                                  pmtmsv.visibility_edit_session_pk,' ||E'\n'||
        '                                  pmtmsv.visibility_deleted_at)' ||E'\n'||
        '    WHERE' ||E'\n'||
        '        in_tenancy_v1($1, attribute_values.tenancy_universal,' ||E'\n'||
        '                          attribute_values.tenancy_billing_account_ids,' ||E'\n'||
        '                          attribute_values.tenancy_organization_ids,' ||E'\n'||
        '                          attribute_values.tenancy_workspace_ids)' ||E'\n'||
        '        AND is_visible_v1($2, attribute_values.visibility_change_set_pk,' ||E'\n'||
        '                              attribute_values.visibility_edit_session_pk,' ||E'\n'||
        '                              attribute_values.visibility_deleted_at)' ||E'\n'||
        '        AND in_attribute_context_v1($3, attribute_values.attribute_context_prop_id,' ||E'\n'||
        '                                        attribute_values.attribute_context_internal_provider_id,' ||E'\n'||
        '                                        attribute_values.attribute_context_external_provider_id,' ||E'\n'||
        '                                        attribute_values.attribute_context_schema_id,' ||E'\n'||
        '                                        attribute_values.attribute_context_schema_variant_id,' ||E'\n'||
        '                                        attribute_values.attribute_context_component_id,' ||E'\n'||
        '                                        attribute_values.attribute_context_system_id)' ||E'\n'||
        '        AND pmtmsv.right_object_id = $4' ||E'\n'||
        '    ORDER BY' ||E'\n'||
        '        attribute_values.attribute_context_prop_id,' ||E'\n'||
        '        COALESCE(avbtav.belongs_to_id, -1),' ||E'\n'||
        '        COALESCE(attribute_values.key, ''''),' ||E'\n'||
        '        attribute_values.visibility_change_set_pk DESC,' ||E'\n'||
        '        attribute_values.visibility_edit_session_pk DESC,' ||E'\n'||
        '        attribute_values.visibility_deleted_at DESC NULLS FIRST,' ||E'\n'||
        '        attribute_values.attribute_context_internal_provider_id DESC,' ||E'\n'||
        '        attribute_values.attribute_context_external_provider_id DESC,' ||E'\n'||
        '        attribute_values.attribute_context_schema_id DESC,' ||E'\n'||
        '        attribute_values.attribute_context_schema_variant_id DESC,' ||E'\n'||
        '        attribute_values.attribute_context_component_id DESC,' ||E'\n'||
        '        attribute_values.attribute_context_system_id DESC' ||E'\n'
        ') AS initial_values' ||E'\n'
    INTO STRICT parent_attribute_value_ids
    USING
        this_tenancy,
        this_visibility,
        this_context,
        this_prop_id;

    -- Make sure we return the result for the base AttributeValue before looping through
    -- to return all of its children.
    RETURN QUERY EXECUTE
        'SELECT ' ||
        '    parent_attribute_value_id, ' ||
        '    attribute_value_object, ' ||
        '    prop_object, ' ||
        '    func_binding_return_value_object ' ||
        'FROM ( ' ||
        '    SELECT DISTINCT ON (attribute_values.id) ' ||
        '        attribute_values.id, ' ||
        '        attribute_values.visibility_change_set_pk, ' ||
        '        attribute_values.visibility_edit_session_pk, ' ||
        '        attribute_values.visibility_deleted_at, ' ||
        '        avbtav.belongs_to_id AS parent_attribute_value_id, ' ||
        '        row_to_json(attribute_values.*) AS attribute_value_object, ' ||
        '        row_to_json(props.*) AS prop_object, ' ||
        '        row_to_json(func_binding_return_values.*) AS func_binding_return_value_object ' ||
        '    FROM ' ||
        '        attribute_values ' ||
        '        LEFT JOIN attribute_value_belongs_to_attribute_value AS avbtav ON ' ||
        '            attribute_values.id = avbtav.object_id ' ||
        '            AND is_visible_v1($2, avbtav.visibility_change_set_pk, ' ||
        '                                  avbtav.visibility_edit_session_pk, ' ||
        '                                  avbtav.visibility_deleted_at) ' ||
        '        INNER JOIN props ON ' ||
        '            attribute_values.attribute_context_prop_id = props.id ' ||
        '            AND is_visible_v1($2, props.visibility_change_set_pk, ' ||
        '                                  props.visibility_edit_session_pk, ' ||
        '                                  props.visibility_deleted_at) ' ||
        '        INNER JOIN func_binding_return_values ON ' ||
        '            func_binding_return_values.id = attribute_values.func_binding_return_value_id ' ||
        '            AND is_visible_v1($2, props.visibility_change_set_pk, ' ||
        '                                  props.visibility_edit_session_pk, ' ||
        '                                  props.visibility_deleted_at) ' ||
        '    WHERE ' ||
        '        attribute_values.id = ANY($3) ' ||
        '        AND in_tenancy_v1($1, attribute_values.tenancy_universal, ' ||
        '                              attribute_values.tenancy_billing_account_ids, ' ||
        '                              attribute_values.tenancy_organization_ids, ' ||
        '                              attribute_values.tenancy_workspace_ids) ' ||
        '        AND is_visible_v1($2, attribute_values.visibility_change_set_pk, ' ||
        '                              attribute_values.visibility_edit_session_pk, ' ||
        '                              attribute_values.visibility_deleted_at) ' ||
        '    ORDER BY ' ||
        '        attribute_values.id, ' ||
        '        visibility_change_set_pk DESC, ' ||
        '        visibility_edit_session_pk DESC, ' ||
        '        visibility_deleted_at DESC NULLS FIRST ' ||
        ') AS desired_attribute_values;'
        USING
            this_tenancy,
            this_visibility,
            parent_attribute_value_ids;

    LOOP
        EXECUTE
            'SELECT '
            '    array_agg(attribute_value_id) AS attribute_value_ids '
            'FROM ('
            '    SELECT DISTINCT ON ( '
            '        avbtav.belongs_to_id, '
            '        attribute_values.attribute_context_prop_id, '
            '        COALESCE(avbtav.belongs_to_id, -1), '
            '        COALESCE(attribute_values.key, '''') '
            '    ) '
            '        attribute_values.id AS attribute_value_id '
            '    FROM '
            '        attribute_values '
            '        LEFT JOIN attribute_value_belongs_to_attribute_value AS avbtav ON '
            '            attribute_values.id = avbtav.object_id '
            '            AND in_tenancy_v1($1, avbtav.tenancy_universal, '
            '                                  avbtav.tenancy_billing_account_ids, '
            '                                  avbtav.tenancy_organization_ids, '
            '                                  avbtav.tenancy_workspace_ids) '
            '            AND is_visible_v1($2, avbtav.visibility_change_set_pk, '
            '                                  avbtav.visibility_edit_session_pk, '
            '                                  avbtav.visibility_deleted_at) '
            '    WHERE '
            '        in_attribute_context_v1($3, attribute_values.attribute_context_prop_id, '
            '                                    attribute_values.attribute_context_internal_provider_id, '
            '                                    attribute_values.attribute_context_external_provider_id, '
            '                                    attribute_values.attribute_context_schema_id, '
            '                                    attribute_values.attribute_context_schema_variant_id, '
            '                                    attribute_values.attribute_context_component_id, '
            '                                    attribute_values.attribute_context_system_id) '
            '        AND avbtav.belongs_to_id = ANY($4) '
            '    ORDER BY '
            '        avbtav.belongs_to_id, '
            '        attribute_values.attribute_context_prop_id, '
            '        COALESCE(avbtav.belongs_to_id, -1), '
            '        COALESCE(attribute_values.key, ''''), '
            '        attribute_values.visibility_change_set_pk DESC, '
            '        attribute_values.visibility_edit_session_pk DESC, '
            '        attribute_values.visibility_deleted_at DESC NULLS FIRST, '
            '        attribute_values.attribute_context_schema_id DESC, '
            '        attribute_values.attribute_context_schema_variant_id DESC, '
            '        attribute_values.attribute_context_component_id DESC, '
            '        attribute_values.attribute_context_system_id DESC '
            ') AS av_ids'
        INTO STRICT new_child_attribute_value_ids
        USING
            this_tenancy,
            this_visibility,
            this_context,
            parent_attribute_value_ids;

        -- Exit the loop, since we haven't found any new child AttributeValues to return.
        EXIT WHEN new_child_attribute_value_ids IS NULL;

        -- This returns a partial result for the AttributeValues that we've found so far.
        RETURN QUERY EXECUTE
            'SELECT ' ||
            '    parent_attribute_value_id, ' ||
            '    attribute_value_object, ' ||
            '    prop_object, ' ||
            '    func_binding_return_value_object ' ||
            'FROM ( ' ||
            '    SELECT DISTINCT ON (attribute_values.id) ' ||
            '        attribute_values.id, ' ||
            '        attribute_values.visibility_change_set_pk, ' ||
            '        attribute_values.visibility_edit_session_pk, ' ||
            '        attribute_values.visibility_deleted_at, ' ||
            '        avbtav.belongs_to_id AS parent_attribute_value_id, ' ||
            '        row_to_json(attribute_values.*) AS attribute_value_object, ' ||
            '        row_to_json(props.*) AS prop_object, ' ||
            '        row_to_json(func_binding_return_values.*) AS func_binding_return_value_object ' ||
            '    FROM ' ||
            '        attribute_values ' ||
            '        LEFT JOIN attribute_value_belongs_to_attribute_value AS avbtav ON ' ||
            '            attribute_values.id = avbtav.object_id ' ||
            '            AND is_visible_v1($2, avbtav.visibility_change_set_pk, ' ||
            '                                  avbtav.visibility_edit_session_pk, ' ||
            '                                  avbtav.visibility_deleted_at) ' ||
            '        INNER JOIN props ON ' ||
            '            attribute_values.attribute_context_prop_id = props.id ' ||
            '            AND is_visible_v1($2, props.visibility_change_set_pk, ' ||
            '                                  props.visibility_edit_session_pk, ' ||
            '                                  props.visibility_deleted_at) ' ||
            '        INNER JOIN func_binding_return_values ON ' ||
            '            func_binding_return_values.id = attribute_values.func_binding_return_value_id ' ||
            '            AND is_visible_v1($2, props.visibility_change_set_pk, ' ||
            '                                  props.visibility_edit_session_pk, ' ||
            '                                  props.visibility_deleted_at) ' ||
            '    WHERE ' ||
            '        attribute_values.id = ANY($3) ' ||
            '        AND in_tenancy_v1($1, attribute_values.tenancy_universal, ' ||
            '                              attribute_values.tenancy_billing_account_ids, ' ||
            '                              attribute_values.tenancy_organization_ids, ' ||
            '                              attribute_values.tenancy_workspace_ids) ' ||
            '        AND is_visible_v1($2, attribute_values.visibility_change_set_pk, ' ||
            '                              attribute_values.visibility_edit_session_pk, ' ||
            '                              attribute_values.visibility_deleted_at) ' ||
            '    ORDER BY ' ||
            '        attribute_values.id, ' ||
            '        visibility_change_set_pk DESC, ' ||
            '        visibility_edit_session_pk DESC, ' ||
            '        visibility_deleted_at DESC NULLS FIRST ' ||
            ') AS desired_attribute_values;'
            USING
                this_tenancy,
                this_visibility,
                new_child_attribute_value_ids;

        -- Prime parent_attribute_value_ids with the child IDs we just found, so
        -- we can look for their children.
        parent_attribute_value_ids := new_child_attribute_value_ids;
    END LOOP;
END;
$$ LANGUAGE PLPGSQL;
