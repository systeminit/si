CREATE OR REPLACE FUNCTION component_restore_and_propagate_v2(
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
    -- Don't run this for components on HEAD
    IF (this_visibility ->> 'visibility_change_set_pk')::ident = ident_nil_v1() THEN
        RAISE WARNING 'Trying to restore component (%) on HEAD', this_component_id;
        RETURN;
    END IF;

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

DROP FUNCTION restore_edge_by_id_v1;
-- deleted this since it's badly named and not used anymore
CREATE OR REPLACE FUNCTION restore_summary_edge_by_id_v1(
    this_tenancy jsonb,
    this_visibility jsonb,
    this_edge_id ident,
    OUT object json
)
AS
$$
DECLARE
    this_tenancy_record    tenancy_record_v1;
    this_visibility_record visibility_record_v1;
BEGIN
    -- Don't restore stuff on head
    IF (this_visibility ->> 'visibility_change_set_pk')::ident = ident_nil_v1() THEN
        RAISE WARNING 'Trying to restore edge (%) on HEAD', this_edge_id;
        RETURN;
    END IF;

    this_tenancy_record := tenancy_json_to_columns_v1(this_tenancy);
    this_visibility_record := visibility_json_to_columns_v1(this_visibility);

    DELETE
    FROM summary_diagram_edges e
    WHERE e.id = this_edge_id
      AND e.tenancy_workspace_pk = this_tenancy_record.tenancy_workspace_pk
      AND e.visibility_change_set_pk = this_visibility_record.visibility_change_set_pk;
END;
$$ LANGUAGE PLPGSQL VOLATILE;
