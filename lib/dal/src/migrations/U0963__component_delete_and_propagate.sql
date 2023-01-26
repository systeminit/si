CREATE OR REPLACE FUNCTION component_delete_and_propagate_v1(
    this_read_tenancy jsonb,
    this_write_tenancy jsonb,
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
        FROM edges_v1(this_read_tenancy, this_visibility) e
                 LEFT JOIN socket_belongs_to_internal_provider sbtip ON sbtip.object_id = e.head_socket_id
                 LEFT JOIN socket_belongs_to_external_provider sbtep ON sbtep.object_id = e.head_socket_id
        WHERE tail_object_id = this_component_id
    LOOP
        PERFORM FROM delete_by_id_v1('edges', this_read_tenancy, this_write_tenancy, this_visibility, target_id);

        -- We have to get the edge head values so we can make update them after edge deletion
        RETURN QUERY SELECT row_to_json(av.*) AS object
                     FROM attribute_values_v1(this_read_tenancy, this_visibility) av
                     WHERE attribute_context_component_id = peer_component_id
                       AND (attribute_context_internal_provider_id = internal_provider_id OR
                            attribute_context_external_provider_id = external_provider_id);
    END LOOP;


    FOR target_id, table_name IN
        (SELECT id, 'edges' as table_name -- Incoming Edges
         FROM edges_v1(this_read_tenancy, this_visibility)
         WHERE head_object_id = this_component_id)
        UNION
        (SELECT id, 'attribute_prototypes' as table_name
         FROM attribute_prototypes_v1(this_read_tenancy, this_visibility)
         WHERE attribute_context_component_id = this_component_id)
        UNION
        (SELECT id, 'attribute_values' as table_name
         FROM attribute_values_v1(this_read_tenancy, this_visibility)
         WHERE attribute_context_component_id = this_component_id)
        UNION
        (SELECT id, 'attribute_prototype_arguments' as table_name
         FROM attribute_prototype_arguments_v1(this_read_tenancy, this_visibility)
         WHERE (head_component_id = this_component_id OR tail_component_id = this_component_id))
        UNION
        (SELECT nbtc.object_id, 'nodes' as table_name
         FROM node_belongs_to_component_v1(this_read_tenancy, this_visibility) nbtc
         WHERE nbtc.belongs_to_id = this_component_id)
        UNION
        (SELECT nbtc.id, 'node_belongs_to_component' as table_name
         FROM node_belongs_to_component_v1(this_read_tenancy, this_visibility) nbtc
         WHERE nbtc.belongs_to_id = this_component_id)
        UNION
        (SELECT npbtn.object_id, 'node_positions' as table_name
         FROM node_belongs_to_component_v1(this_read_tenancy, this_visibility) nbtc
                  INNER JOIN node_position_belongs_to_node_v1(this_read_tenancy, this_visibility) npbtn
                             ON nbtc.object_id = npbtn.belongs_to_id
         WHERE nbtc.belongs_to_id = this_component_id)
        UNION
        (SELECT npbtn.id, 'node_position_belongs_to_node' as table_name
         FROM node_belongs_to_component_v1(this_read_tenancy, this_visibility) nbtc
                  INNER JOIN node_position_belongs_to_node_v1(this_read_tenancy, this_visibility) npbtn
                             ON nbtc.object_id = npbtn.belongs_to_id
         WHERE nbtc.belongs_to_id = this_component_id)
    LOOP
        -- In the future, we'll possibly want to deal differently with edges that don't exist on HEAD vs the ones that do
        -- we don't make that distinction right now
        PERFORM
        FROM delete_by_id_v1(
                table_name, this_read_tenancy, this_write_tenancy,
                this_visibility, target_id);
    END LOOP;

    PERFORM
    FROM delete_by_id_v1(
            'components',
            this_read_tenancy, this_write_tenancy, this_visibility,
            this_component_id);
END;
$$ LANGUAGE PLPGSQL STABLE;