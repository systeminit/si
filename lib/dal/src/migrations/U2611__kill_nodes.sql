-- DROP NODES
DROP TABLE nodes CASCADE;
DROP TABLE node_belongs_to_component CASCADE;
DELETE
FROM standard_models
where table_name = 'nodes';
DELETE
FROM standard_models
where table_name = 'node_belongs_to_component';
DROP FUNCTION node_create_v1;

-- MODIFY EDGES
ALTER TABLE edges
    DROP head_node_id,
    DROP head_object_kind,
    DROP tail_node_id,
    DROP tail_object_kind;
ALTER TABLE edges
    RENAME head_object_id TO head_component_id;
ALTER TABLE edges
    RENAME tail_object_id TO tail_component_id;

CREATE OR REPLACE FUNCTION edge_create_v2(
    this_tenancy jsonb,
    this_visibility jsonb,
    this_kind text,
    this_head_component_id ident,
    this_head_socket_id ident,
    this_tail_component_id ident,
    this_tail_socket_id ident,
    this_user_pk ident,
    OUT object json) AS
$$
DECLARE
    this_tenancy_record    tenancy_record_v1;
    this_visibility_record visibility_record_v1;
    this_new_row           edges%ROWTYPE;
BEGIN
    this_tenancy_record := tenancy_json_to_columns_v1(this_tenancy);
    this_visibility_record := visibility_json_to_columns_v1(this_visibility);

    INSERT INTO edges (tenancy_workspace_pk,
                       visibility_change_set_pk, kind,
                       head_component_id, head_socket_id,
                       tail_component_id, tail_socket_id,
                       creation_user_pk)
    VALUES (this_tenancy_record.tenancy_workspace_pk,
            this_visibility_record.visibility_change_set_pk,
            this_kind,
            this_head_component_id, this_head_socket_id,
            this_tail_component_id, this_tail_socket_id, this_user_pk)
    RETURNING * INTO this_new_row;

    object := row_to_json(this_new_row);
END;
$$ LANGUAGE PLPGSQL VOLATILE;

ALTER TABLE components
    ADD COLUMN x      text NOT NULL DEFAULT '0',
    ADD COLUMN y      text NOT NULL DEFAULT '0',
    ADD COLUMN width  text,
    ADD COLUMN height text;

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
        SELECT e.id, e.head_component_id, sbtip.belongs_to_id, sbtep.belongs_to_id
        FROM edges_v1(this_tenancy, this_visibility) e
                 LEFT JOIN socket_belongs_to_internal_provider_v1(this_tenancy, this_visibility) sbtip
                           ON sbtip.object_id = e.head_socket_id
                 LEFT JOIN socket_belongs_to_external_provider_v1(this_tenancy, this_visibility) sbtep
                           ON sbtep.object_id = e.head_socket_id
        WHERE e.tail_component_id = this_component_id
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
        WHERE head_component_id = this_component_id
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
                    (this_visibility ->> 'visibility_deleted_at')::timestamptz,
                    this_component_av_id
                    );
        END LOOP;

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


CREATE OR REPLACE FUNCTION component_restore_and_propagate_v4(
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
    external_provider_id         ident;
    internal_provider_id         ident;
    peer_component_id            ident;
    peer_component_ids           ident[];
    target_pk                    ident;
    this_visibility_with_deleted jsonb;
BEGIN
    -- Don't run this for components on HEAD
    IF (this_visibility ->> 'visibility_change_set_pk')::ident = ident_nil_v1() THEN
        RAISE WARNING 'Trying to restore component (%) on HEAD', this_component_id;
        RETURN;
    END IF;

    this_visibility_with_deleted := this_visibility || jsonb_build_object('visibility_deleted_at', now());

    -- Outgoing Edges
    FOR target_pk, peer_component_id, internal_provider_id, external_provider_id IN
        SELECT e.pk, e.head_component_id, sbtip.belongs_to_id, sbtep.belongs_to_id
        FROM edges_v1(this_tenancy, this_visibility_with_deleted) e
                 LEFT JOIN socket_belongs_to_internal_provider sbtip ON sbtip.object_id = e.head_socket_id
                 LEFT JOIN socket_belongs_to_external_provider sbtep ON sbtep.object_id = e.head_socket_id
        WHERE tail_component_id = this_component_id
          AND e.visibility_deleted_at IS NOT NULL
          AND e.visibility_change_set_pk = (this_visibility ->> 'visibility_change_set_pk')::ident
        LOOP
            peer_component_ids := array_append(peer_component_ids, peer_component_id);

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
        WHERE head_component_id = this_component_id
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

    SELECT pk
    INTO target_pk
    FROM components_v1(this_tenancy, this_visibility_with_deleted)
    WHERE id = this_component_id
      AND visibility_deleted_at IS NOT NULL
      AND visibility_change_set_pk = (this_visibility ->> 'visibility_change_set_pk')::ident;

    PERFORM hard_delete_by_pk_v1('components', target_pk);

    -- Update the dependency graph for the "restored" Component.
    PERFORM attribute_value_dependencies_update_component_v1(
            this_tenancy,
            this_visibility,
            this_component_id
            );

    -- Update the dependency graphs of all Components that used this Component.
    FOREACH peer_component_id IN ARRAY peer_component_ids
        LOOP
            PERFORM attribute_value_dependencies_update_component_v1(
                    this_tenancy,
                    this_visibility,
                    peer_component_id
                    );
        END LOOP;
END;
$$ LANGUAGE PLPGSQL STABLE;
