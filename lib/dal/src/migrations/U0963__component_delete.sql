CREATE OR REPLACE FUNCTION component_delete_and_propagate_v1(
    this_tenancy jsonb,
    this_visibility jsonb,
    this_component_id ident,
    this_user_id ident,
    this_has_resource boolean
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
    deleted_timestamp    timestamp with time zone;
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
        SELECT nbtc.object_id, 'nodes' as table_name
        FROM node_belongs_to_component_v1(this_tenancy, this_visibility) nbtc
        WHERE nbtc.belongs_to_id = this_component_id
        UNION
        SELECT nbtc.id, 'node_belongs_to_component' as table_name
        FROM node_belongs_to_component_v1(this_tenancy, this_visibility) nbtc
        WHERE nbtc.belongs_to_id = this_component_id
    LOOP
        -- In the future, we'll possibly want to deal differently with edges that don't exist on HEAD vs the ones that do
        -- we don't make that distinction right now
        PERFORM delete_by_id_v1(table_name, this_tenancy, this_visibility, target_id);
    END LOOP;

    SELECT delete_by_id_v1('components', this_tenancy, this_visibility, this_component_id) INTO deleted_timestamp;

    -- Mark the component as needing destruction
    PERFORM update_by_id_v1('components',
                            'needs_destroy',
                            this_tenancy,
                            this_visibility || jsonb_build_object('visibility_deleted_at', deleted_timestamp),
                            this_component_id,
                            this_has_resource);

    -- Ensure we now set the actor of who has deleted the component
    PERFORM update_by_id_v1('components',
                            'deletion_user_id',
                            this_tenancy,
                            this_visibility || jsonb_build_object('visibility_deleted_at', deleted_timestamp),
                            this_component_id,
                            this_user_id);
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
