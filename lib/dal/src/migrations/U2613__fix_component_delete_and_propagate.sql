CREATE OR REPLACE FUNCTION component_delete_and_propagate_v3(
    this_tenancy jsonb,
    this_visibility jsonb,
    this_component_id ident,
    this_user_pk ident,
    this_has_resource boolean
)
    RETURNS TABLE
            (
                object json
            )
AS
$$
DECLARE
    deleted_timestamp       timestamp with time zone;
    external_provider_id    ident;
    internal_provider_id    ident;
    peer_component_id       ident;
    table_name              text;
    target_id               ident;
    this_peer_component_ids ident[];
    this_component_av_id    ident;
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
            SELECT delete_by_id_v1('edges', this_tenancy, this_visibility, target_id) INTO deleted_timestamp;

            -- We have to get the edge head values so we can update them after edge deletion
            RETURN QUERY SELECT row_to_json(av.*) AS object
                         FROM attribute_values_v1(this_tenancy, this_visibility) av
                         WHERE attribute_context_component_id = peer_component_id
                           AND (attribute_context_internal_provider_id = internal_provider_id OR
                                attribute_context_external_provider_id = external_provider_id);
            SELECT array_agg(av.attribute_context_component_id)
            INTO this_peer_component_ids
            FROM attribute_values_v1(this_tenancy, this_visibility) av
            WHERE attribute_context_component_id = peer_component_id
              AND (attribute_context_internal_provider_id = internal_provider_id OR
                   attribute_context_external_provider_id = external_provider_id);

            PERFORM update_by_id_v1('edges',
                                    'deleted_implicitly',
                                    this_tenancy,
                                    this_visibility || jsonb_build_object('visibility_deleted_at', deleted_timestamp),
                                    target_id,
                                    true);
        END LOOP;

    FOR target_id, table_name IN
        SELECT id, 'edges' as table_name -- Incoming Edges
        FROM edges_v1(this_tenancy, this_visibility)
        WHERE head_object_id = this_component_id
        LOOP
        -- In the future, we'll possibly want to deal differently with edges that don't exist on HEAD vs the ones that do
        -- we don't make that distinction right now
            SELECT delete_by_id_v1(table_name, this_tenancy, this_visibility, target_id) INTO deleted_timestamp;

            PERFORM update_by_id_v1('edges',
                                    'deleted_implicitly',
                                    this_tenancy,
                                    this_visibility || jsonb_build_object('visibility_deleted_at', deleted_timestamp),
                                    target_id,
                                    true);
        END LOOP;

    FOR target_id, table_name IN
        SELECT nbtc.object_id, 'nodes' as table_name
        FROM node_belongs_to_component_v1(this_tenancy, this_visibility) nbtc
        WHERE nbtc.belongs_to_id = this_component_id
        UNION
        SELECT nbtc.id, 'node_belongs_to_component' as table_name
        FROM node_belongs_to_component_v1(this_tenancy, this_visibility) nbtc
        WHERE nbtc.belongs_to_id = this_component_id
        LOOP
            PERFORM delete_by_id_v1(table_name, this_tenancy, this_visibility, target_id);
        END LOOP;

    SELECT delete_by_id_v1('components', this_tenancy, this_visibility, this_component_id) INTO deleted_timestamp;

    -- Remove the deleted Component's AttributeValues from the dependency graph.
    FOR this_component_av_id IN
        SELECT av.id
        FROM attribute_values_v1(this_tenancy, this_visibility) AS av
        WHERE av.attribute_context_component_id = this_component_id
        LOOP
            PERFORM attribute_value_dependencies_update_v1(
                    (this_tenancy ->> 'tenancy_workspace_pk')::ident,
                    (this_visibility ->> 'visibility_change_set_pk')::ident,
                    deleted_timestamp::timestamptz,
                    this_component_av_id
                    );
        END LOOP;

    IF this_peer_component_ids IS NULL THEN
        this_peer_component_ids := '{}';
    end if;

    -- Update the dependencies of all Components that used this one as an input
    FOREACH peer_component_id IN ARRAY this_peer_component_ids
        LOOP
            PERFORM attribute_value_dependencies_update_component_v1(
                    this_tenancy,
                    this_visibility,
                    peer_component_id
                    );
        END LOOP;

    -- Mark the component as needing destruction
    PERFORM update_by_id_v1('components',
                            'needs_destroy',
                            this_tenancy,
                            this_visibility || jsonb_build_object('visibility_deleted_at', deleted_timestamp),
                            this_component_id,
                            this_has_resource);

    -- Ensure we now set the actor of who has deleted the component
    PERFORM update_by_id_v1('components',
                            'deletion_user_pk',
                            this_tenancy,
                            this_visibility || jsonb_build_object('visibility_deleted_at', deleted_timestamp),
                            this_component_id,
                            this_user_pk);
END;
$$ LANGUAGE PLPGSQL STABLE;
