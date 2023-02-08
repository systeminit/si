CREATE OR REPLACE FUNCTION component_delete_and_propagate_v1(
    this_tenancy jsonb,
    this_visibility jsonb,
    this_component_id ident
)
    RETURNS TABLE
            (
                object json
            )
AS
$$
DECLARE
    table_name           text;
    target_id            ident;
    peer_component_id    ident;
    internal_provider_id ident;
    external_provider_id ident;
BEGIN

    -- Outgoing Edges
    FOR target_id, peer_component_id, internal_provider_id, external_provider_id IN
        SELECT e.id, e.head_object_id, sbtip.belongs_to_id, sbtep.belongs_to_id
        FROM edges_v1(this_tenancy, this_visibility) e
                 LEFT JOIN socket_belongs_to_internal_provider_v1(this_tenancy, this_visibility) sbtip
                           ON sbtip.object_id = e.head_socket_id
                 LEFT JOIN socket_belongs_to_external_provider_v1(this_tenancy, this_visibility) sbtep
                           ON sbtep.object_id = e.head_socket_id
        WHERE e.tail_object_id = this_component_id
    LOOP
        PERFORM delete_by_id_v1('edges', this_tenancy, this_visibility, target_id);

        -- We have to get the edge head values so we can update them after edge deletion
        RETURN QUERY SELECT row_to_json(av.*) AS object
                     FROM attribute_values_v1(this_tenancy, this_visibility) av
                     WHERE attribute_context_component_id = peer_component_id
                       AND (attribute_context_internal_provider_id = internal_provider_id OR
                            attribute_context_external_provider_id = external_provider_id);
    END LOOP;


    FOR target_id, table_name IN
        SELECT id, 'edges' as table_name -- Incoming Edges
        FROM edges_v1(this_tenancy, this_visibility)
        WHERE head_object_id = this_component_id
        UNION
        SELECT id, 'attribute_prototypes' as table_name
        FROM attribute_prototypes_v1(this_tenancy, this_visibility)
        WHERE attribute_context_component_id = this_component_id
        UNION
        SELECT id, 'attribute_values' as table_name
        FROM attribute_values_v1(this_tenancy, this_visibility)
        WHERE attribute_context_component_id = this_component_id
        UNION
        SELECT id, 'attribute_prototype_arguments' as table_name
        FROM attribute_prototype_arguments_v1(this_tenancy, this_visibility)
        WHERE (head_component_id = this_component_id OR tail_component_id = this_component_id)
        UNION
        SELECT nbtc.object_id, 'nodes' as table_name
        FROM node_belongs_to_component_v1(this_tenancy, this_visibility) nbtc
        WHERE nbtc.belongs_to_id = this_component_id
        UNION
        SELECT nbtc.id, 'node_belongs_to_component' as table_name
        FROM node_belongs_to_component_v1(this_tenancy, this_visibility) nbtc
        WHERE nbtc.belongs_to_id = this_component_id
        UNION
        SELECT npbtn.object_id, 'node_positions' as table_name
        FROM node_belongs_to_component_v1(this_tenancy, this_visibility) nbtc
                 INNER JOIN node_position_belongs_to_node_v1(this_tenancy, this_visibility) npbtn
                            ON nbtc.object_id = npbtn.belongs_to_id
        WHERE nbtc.belongs_to_id = this_component_id
        UNION
        SELECT npbtn.id, 'node_position_belongs_to_node' as table_name
        FROM node_belongs_to_component_v1(this_tenancy, this_visibility) nbtc
                 INNER JOIN node_position_belongs_to_node_v1(this_tenancy, this_visibility) npbtn
                            ON nbtc.object_id = npbtn.belongs_to_id
        WHERE nbtc.belongs_to_id = this_component_id
    LOOP
        -- In the future, we'll possibly want to deal differently with edges that don't exist on HEAD vs the ones that do
        -- we don't make that distinction right now
        PERFORM delete_by_id_v1(table_name, this_tenancy, this_visibility, target_id);
    END LOOP;

    PERFORM delete_by_id_v1('components', this_tenancy, this_visibility, this_component_id);
END;
$$ LANGUAGE PLPGSQL STABLE;

CREATE OR REPLACE FUNCTION component_restore_and_propagate_v1(
    this_tenancy jsonb,
    this_visibility jsonb,
    this_component_id ident
)
    RETURNS TABLE
            (
                object json
            )
AS
$$
DECLARE
    table_name                   text;
    target_pk                    ident;
    peer_component_id            ident;
    internal_provider_id         ident;
    external_provider_id         ident;
    this_visibility_with_deleted jsonb;

BEGIN
    this_visibility_with_deleted := this_visibility || jsonb_build_object('visibility_deleted_at', now());

    -- Outgoing Edges
    FOR target_pk, peer_component_id, internal_provider_id, external_provider_id IN
        SELECT e.pk, e.head_object_id, sbtip.belongs_to_id, sbtep.belongs_to_id
        FROM edges_v1(this_tenancy, this_visibility_with_deleted) e
                 LEFT JOIN socket_belongs_to_internal_provider sbtip ON sbtip.object_id = e.head_socket_id
                 LEFT JOIN socket_belongs_to_external_provider sbtep ON sbtep.object_id = e.head_socket_id
        WHERE tail_object_id = this_component_id
          AND e.visibility_deleted_at IS NOT NULL
          AND e.visibility_change_set_pk = (this_visibility ->> 'visibility_change_set_pk')::ident
    LOOP
        -- In the future, we'll possibly want to deal differently with edges that don't exist on HEAD vs the ones that do
        -- we don't make that distinction right now
        PERFORM hard_delete_by_pk_v1('edges', target_pk);

        -- We have to get the edge head values so we can make update them after edge deletion
        RETURN QUERY SELECT row_to_json(av.*) AS object
                     FROM attribute_values_v1(this_tenancy, this_visibility) av
                     WHERE attribute_context_component_id = peer_component_id
                       AND (attribute_context_internal_provider_id = internal_provider_id OR
                            attribute_context_external_provider_id = external_provider_id);
    END LOOP;

    -- Incoming Edges
    FOR target_pk, internal_provider_id, external_provider_id IN
        SELECT e.pk, sbtip.belongs_to_id, sbtep.belongs_to_id
        FROM edges_v1(this_tenancy, this_visibility_with_deleted) e
                 LEFT JOIN socket_belongs_to_internal_provider sbtip ON sbtip.object_id = e.head_socket_id
                 LEFT JOIN socket_belongs_to_external_provider sbtep ON sbtep.object_id = e.head_socket_id
        WHERE head_object_id = this_component_id
          AND e.visibility_deleted_at IS NOT NULL
          AND e.visibility_change_set_pk = (this_visibility ->> 'visibility_change_set_pk')::ident
    LOOP
        PERFORM hard_delete_by_pk_v1('edges', target_pk);

        -- We have to get the edge head values so we can make update them after edge deletion
        RETURN QUERY SELECT row_to_json(av.*) AS object
                     FROM attribute_values_v1(this_tenancy, this_visibility) av
                     WHERE attribute_context_component_id = this_component_id
                       AND (attribute_context_internal_provider_id = internal_provider_id OR
                            attribute_context_external_provider_id = external_provider_id);
    END LOOP;

    -- Standard component dependencies that don't need extra work
    FOR target_pk, table_name IN
        SELECT pk, agg.table_name
        FROM (SELECT pk, 'attribute_prototypes' as table_name, visibility_deleted_at, visibility_change_set_pk
              FROM attribute_prototypes_v1(this_tenancy, this_visibility_with_deleted)
              WHERE attribute_context_component_id = this_component_id
              UNION
              SELECT pk, 'attribute_values' as table_name, visibility_deleted_at, visibility_change_set_pk
              FROM attribute_values_v1(this_tenancy, this_visibility_with_deleted)
              WHERE attribute_context_component_id = this_component_id
              UNION
              SELECT pk, 'attribute_prototype_arguments' as table_name, visibility_deleted_at, visibility_change_set_pk
              FROM attribute_prototype_arguments_v1(this_tenancy, this_visibility_with_deleted)
              WHERE (head_component_id = this_component_id OR tail_component_id = this_component_id)) as agg
        WHERE visibility_deleted_at IS NOT NULL
          AND visibility_change_set_pk = (this_visibility ->> 'visibility_change_set_pk')::ident
    LOOP
        PERFORM hard_delete_by_pk_v1(table_name, target_pk);
    END LOOP;

    -- Belongs to queries are a bit more complicated (and should be gone pretty soon)
    FOR target_pk, table_name IN
        SELECT nbtc.pk, 'node_belongs_to_component' as table_name
        FROM node_belongs_to_component_v1(this_tenancy, this_visibility_with_deleted) nbtc
        WHERE nbtc.belongs_to_id = this_component_id
          AND nbtc.visibility_deleted_at IS NOT NULL
          AND nbtc.visibility_change_set_pk = (this_visibility ->> 'visibility_change_set_pk')::ident
        UNION
        SELECT n.pk, 'nodes' as table_name
        FROM node_belongs_to_component_v1(this_tenancy, this_visibility_with_deleted) nbtc
                 INNER JOIN nodes_v1(this_tenancy, this_visibility_with_deleted) n ON n.id = nbtc.object_id
            AND n.visibility_deleted_at IS NOT NULL
            AND n.visibility_change_set_pk = (this_visibility ->> 'visibility_change_set_pk')::ident
        WHERE nbtc.belongs_to_id = this_component_id
          AND nbtc.visibility_deleted_at IS NOT NULL
          AND nbtc.visibility_change_set_pk = (this_visibility ->> 'visibility_change_set_pk')::ident
        UNION
        SELECT npbtn.pk, 'node_position_belongs_to_node' as table_name
        FROM node_belongs_to_component_v1(this_tenancy, this_visibility_with_deleted) nbtc
                 INNER JOIN node_position_belongs_to_node_v1(this_tenancy, this_visibility_with_deleted) npbtn
                            ON nbtc.object_id = npbtn.belongs_to_id
                                AND npbtn.visibility_deleted_at IS NOT NULL
                                AND npbtn.visibility_change_set_pk = (this_visibility ->> 'visibility_change_set_pk')::ident
        WHERE nbtc.belongs_to_id = this_component_id
          AND nbtc.visibility_deleted_at IS NOT NULL
          AND nbtc.visibility_change_set_pk = (this_visibility ->> 'visibility_change_set_pk')::ident
        UNION
        SELECT np.pk, 'node_positions' as table_name
        FROM node_belongs_to_component_v1(this_tenancy, this_visibility_with_deleted) nbtc
                 INNER JOIN node_position_belongs_to_node_v1(this_tenancy, this_visibility_with_deleted) npbtn
                            ON nbtc.object_id = npbtn.belongs_to_id
                                AND npbtn.visibility_deleted_at IS NOT NULL
                                AND npbtn.visibility_change_set_pk = (this_visibility ->> 'visibility_change_set_pk')::ident
                 INNER JOIN node_positions_v1(this_tenancy, this_visibility_with_deleted) np
                            ON npbtn.object_id = np.id
                                AND np.visibility_deleted_at IS NOT NULL
                                AND np.visibility_change_set_pk = (this_visibility ->> 'visibility_change_set_pk')::ident

        WHERE nbtc.belongs_to_id = this_component_id
          AND nbtc.visibility_deleted_at IS NOT NULL
          AND nbtc.visibility_change_set_pk = (this_visibility ->> 'visibility_change_set_pk')::ident
    LOOP
        PERFORM hard_delete_by_pk_v1(table_name, target_pk);
    END LOOP;

    SELECT pk
    INTO target_pk
    FROM components_v1(this_tenancy, this_visibility_with_deleted)
    WHERE id = this_component_id
      AND visibility_deleted_at IS NOT NULL
      AND visibility_change_set_pk = (this_visibility ->> 'visibility_change_set_pk')::ident;

    PERFORM hard_delete_by_pk_v1('components', target_pk);
END;
$$ LANGUAGE PLPGSQL STABLE;
