CREATE TYPE func_with_attribute_prototype_context AS
(
    id                                  bigint,
    name                                text,
    display_name                        text,
    backend_kind                        text,
    backend_response_type               text,
    is_builtin                          bool,
    attribute_prototype_id              bigint,
    attribute_context_schema_id         bigint,
    attribute_context_schema_variant_id bigint,
    attribute_context_component_id      bigint,
    attribute_context_system_id         bigint
);

CREATE OR REPLACE FUNCTION attribute_value_list_payload_for_read_context_and_root_v1(this_tenancy jsonb,
                                                                                     this_visibility jsonb,
                                                                                     this_context jsonb,
                                                                                     this_attribute_value_id bigint)
    RETURNS TABLE
            (
                parent_attribute_value_id        bigint,
                attribute_value_object           json,
                prop_object                      json,
                func_binding_return_value_object json,
                func_with_prototype_context      json
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
            avbtav.parent_attribute_value_id             AS parent_attribute_value_id,
            row_to_json(av.*)                            AS attribute_value_object,
            row_to_json(prop.*)                          AS prop_object,
            row_to_json(fbrv.*)                          AS func_binding_return_value_object,
            row_to_json(cast(row (
                func.id,
                func.name,
                func.display_name,
                func.backend_kind,
                func.backend_response_type,
                CASE
                    WHEN func.tenancy_universal is true AND func.visibility_change_set_pk = -1
                        THEN true
                    ELSE false END ,
                ap.id,
                ap.attribute_context_schema_id,
                ap.attribute_context_schema_variant_id,
                ap.attribute_context_component_id,
                ap.attribute_context_system_id
            ) AS func_with_attribute_prototype_context)) AS func_with_prototype_context
        FROM attribute_values_v1(this_tenancy, this_visibility) AS av
        LEFT JOIN (
            SELECT DISTINCT ON (object_id)
                object_id AS child_attribute_value_id,
                belongs_to_id AS parent_attribute_value_id
            FROM attribute_value_belongs_to_attribute_value
            WHERE in_tenancy_and_visible_v1(this_tenancy, this_visibility, attribute_value_belongs_to_attribute_value)
            ORDER BY
                object_id,
                visibility_change_set_pk DESC,
                visibility_deleted_at DESC NULLS FIRST
        ) AS avbtav ON av.id = avbtav.child_attribute_value_id
        INNER JOIN (
            SELECT DISTINCT ON (object_id)
                object_id AS attribute_value_id,
                belongs_to_id AS attribute_prototype_id
            FROM attribute_value_belongs_to_attribute_prototype AS avbtap
            WHERE in_tenancy_and_visible_v1(this_tenancy, this_visibility, avbtap)
            ORDER BY
                object_id,
                visibility_change_set_pk DESC,
                visibility_deleted_at DESC NULLS FIRST
        ) AS avbtap ON avbtap.attribute_value_id = av.id
        INNER JOIN (
            SELECT DISTINCT ON (id)
                *
            FROM attribute_prototypes
            WHERE in_tenancy_and_visible_v1(this_tenancy, this_visibility, attribute_prototypes)
            ORDER BY
                id,
                visibility_change_set_pk DESC,
                visibility_deleted_at DESC
        ) AS ap ON avbtap.attribute_prototype_id = ap.id
        INNER JOIN (
            SELECT DISTINCT ON (id)
                *
            FROM funcs
            WHERE in_tenancy_and_visible_v1(this_tenancy, this_visibility, funcs)
            ORDER BY
                id,
                visibility_change_set_pk DESC,
                visibility_deleted_at DESC NULLS FIRST
        ) AS func ON ap.func_id = func.id
        INNER JOIN (
            SELECT DISTINCT ON (id)
                *
            FROM props
            WHERE in_tenancy_and_visible_v1(this_tenancy, this_visibility, props)
            ORDER BY
                id,
                visibility_change_set_pk DESC,
                visibility_deleted_at DESC NULLS FIRST
        ) AS prop ON av.attribute_context_prop_id = prop.id
        INNER JOIN (
            SELECT DISTINCT ON (id)
                *
            FROM func_binding_return_values
            WHERE in_tenancy_and_visible_v1(this_tenancy, this_visibility, func_binding_return_values)
            ORDER BY
                id,
                visibility_change_set_pk DESC,
                visibility_deleted_at DESC NULLS FIRST
        ) AS fbrv ON fbrv.id = av.func_binding_return_value_id
        WHERE av.id = this_attribute_value_id
        ORDER BY av.id;

    parent_attribute_value_ids := ARRAY [this_attribute_value_id];
    LOOP
        SELECT array_agg(attribute_value_id) AS attribute_value_ids
        INTO STRICT new_child_attribute_value_ids
        FROM (
            SELECT DISTINCT ON (
                COALESCE(avbtav.parent_attribute_value_id, -1),
                av.attribute_context_prop_id,
                COALESCE(av.key, '')
            )
            av.id AS attribute_value_id
            FROM attribute_values_v1(this_tenancy, this_visibility) AS av
            LEFT JOIN (
                SELECT DISTINCT ON (object_id)
                    object_id AS child_attribute_value_id,
                    belongs_to_id AS parent_attribute_value_id
                FROM attribute_value_belongs_to_attribute_value
                WHERE in_tenancy_and_visible_v1(this_tenancy, this_visibility, attribute_value_belongs_to_attribute_value)
                ORDER BY
                    object_id,
                    visibility_change_set_pk DESC,
                    visibility_deleted_at DESC NULLS FIRST
            ) AS avbtav ON av.id = avbtav.child_attribute_value_id
            WHERE
                in_attribute_context_v1(this_context, av)
                AND avbtav.parent_attribute_value_id = ANY (parent_attribute_value_ids)
            ORDER BY
                COALESCE(avbtav.parent_attribute_value_id, -1) DESC,
                av.attribute_context_prop_id DESC,
                COALESCE(av.key, ''),
                av.attribute_context_schema_id DESC,
                av.attribute_context_schema_variant_id DESC,
                av.attribute_context_component_id DESC,
                av.attribute_context_system_id DESC
        ) AS av_ids;

        -- Exit the loop, since we haven't found any new child AttributeValues to return.
        EXIT WHEN new_child_attribute_value_ids IS NULL;

        -- This returns a partial result for the AttributeValues that we've found so far.
        RETURN QUERY
            SELECT
                avbtav.parent_attribute_value_id             AS parent_attribute_value_id,
                row_to_json(av.*)                            AS attribute_value_object,
                row_to_json(prop.*)                          AS prop_object,
                row_to_json(fbrv.*)                          AS func_binding_return_value_object,
                row_to_json(cast(row (
                    func.id,
                    func.name,
                    func.display_name,
                    func.backend_kind,
                    func.backend_response_type,
                    CASE
                        WHEN func.tenancy_universal is true AND func.visibility_change_set_pk = -1
                            THEN true
                        ELSE false END ,
                    ap.id,
                    ap.attribute_context_schema_id,
                    ap.attribute_context_schema_variant_id,
                    ap.attribute_context_component_id,
                    ap.attribute_context_system_id
                ) AS func_with_attribute_prototype_context)) AS func_with_prototype_context
            FROM attribute_values_v1(this_tenancy, this_visibility) AS av
            LEFT JOIN (
                SELECT DISTINCT ON (object_id)
                    object_id AS child_attribute_value_id,
                    belongs_to_id AS parent_attribute_value_id
                FROM attribute_value_belongs_to_attribute_value
                WHERE in_tenancy_and_visible_v1(this_tenancy, this_visibility, attribute_value_belongs_to_attribute_value)
                ORDER BY
                    object_id,
                    visibility_change_set_pk DESC,
                    visibility_deleted_at DESC NULLS FIRST
            ) AS avbtav ON av.id = avbtav.child_attribute_value_id
            INNER JOIN (
                SELECT DISTINCT ON (object_id)
                    object_id AS attribute_value_id,
                    belongs_to_id AS attribute_prototype_id
                FROM attribute_value_belongs_to_attribute_prototype AS avbtap
                WHERE in_tenancy_and_visible_v1(this_tenancy, this_visibility, avbtap)
                ORDER BY
                    object_id,
                    visibility_change_set_pk DESC,
                    visibility_deleted_at DESC NULLS FIRST
            ) AS avbtap ON avbtap.attribute_value_id = av.id
            INNER JOIN (
                SELECT DISTINCT ON (id)
                    *
                FROM attribute_prototypes
                WHERE in_tenancy_and_visible_v1(this_tenancy, this_visibility, attribute_prototypes)
                ORDER BY
                    id,
                    visibility_change_set_pk DESC,
                    visibility_deleted_at DESC
            ) AS ap ON avbtap.attribute_prototype_id = ap.id
            INNER JOIN (
                SELECT DISTINCT ON (id)
                    *
                FROM funcs
                WHERE in_tenancy_and_visible_v1(this_tenancy, this_visibility, funcs)
                ORDER BY
                    id,
                    visibility_change_set_pk DESC,
                    visibility_deleted_at DESC NULLS FIRST
            ) AS func ON ap.func_id = func.id
            INNER JOIN (
                SELECT DISTINCT ON (id)
                    *
                FROM props
                WHERE in_tenancy_and_visible_v1(this_tenancy, this_visibility, props)
                ORDER BY
                    id,
                    visibility_change_set_pk DESC,
                    visibility_deleted_at DESC NULLS FIRST
            ) AS prop ON av.attribute_context_prop_id = prop.id
            INNER JOIN (
                SELECT DISTINCT ON (id)
                    *
                FROM func_binding_return_values
                WHERE in_tenancy_and_visible_v1(this_tenancy, this_visibility, func_binding_return_values)
                ORDER BY
                    id,
                    visibility_change_set_pk DESC,
                    visibility_deleted_at DESC NULLS FIRST
            ) AS fbrv ON fbrv.id = av.func_binding_return_value_id
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
        this_prop_id bigint
    ) RETURNS bigint LANGUAGE sql PARALLEL SAFE AS $$
SELECT DISTINCT ON (
        av.attribute_context_prop_id,
        COALESCE(avbtav.belongs_to_id, -1),
        COALESCE(av.key, '')
    ) av.id
FROM attribute_values_v1(this_tenancy, this_visibility) AS av
    LEFT JOIN attribute_value_belongs_to_attribute_value AS avbtav ON av.id = avbtav.object_id
    AND in_tenancy_and_visible_v1(this_tenancy, this_visibility, avbtav)
    INNER JOIN prop_many_to_many_schema_variants AS pmtmsv ON av.attribute_context_prop_id = pmtmsv.left_object_id
    AND in_tenancy_and_visible_v1(this_tenancy, this_visibility, pmtmsv)
WHERE in_attribute_context_v1(this_context, av)
    AND pmtmsv.right_object_id = this_prop_id
ORDER BY av.attribute_context_prop_id,
    COALESCE(avbtav.belongs_to_id, -1),
    COALESCE(av.key, ''),
    av.visibility_change_set_pk DESC,
    av.visibility_deleted_at DESC NULLS FIRST,
    av.attribute_context_internal_provider_id DESC,
    av.attribute_context_external_provider_id DESC,
    av.attribute_context_schema_id DESC,
    av.attribute_context_schema_variant_id DESC,
    av.attribute_context_component_id DESC,
    av.attribute_context_system_id DESC $$;
CREATE OR REPLACE FUNCTION attribute_value_list_payload_for_read_context_v1(
        this_tenancy jsonb,
        this_visibility jsonb,
        this_context jsonb,
        this_prop_id bigint
    ) RETURNS TABLE (
        parent_attribute_value_id bigint,
        attribute_value_object json,
        prop_object json,
        func_binding_return_value_object json,
        func_with_prototype_context json
    ) LANGUAGE sql PARALLEL SAFE AS $$
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
    ) $$;
