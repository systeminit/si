CREATE TABLE IF NOT EXISTS attribute_value_dependencies (
    pk                             ident PRIMARY KEY DEFAULT ident_create_v1(),
    id                             ident NOT NULL DEFAULT ident_create_v1(),
    tenancy_workspace_pk           ident,
    visibility_change_set_pk       ident NOT NULL DEFAULT ident_nil_v1(),
    visibility_deleted_at          TIMESTAMP WITH TIME ZONE,
    created_at                     TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT clock_timestamp(),
    updated_at                     TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT clock_timestamp(),
    source_attribute_value_id      ident NOT NULL,
    destination_attribute_value_id ident NOT NULL
);

SELECT standard_model_table_constraints_v1('attribute_value_dependencies');
INSERT INTO standard_models (table_name, table_type, history_event_label_base, history_event_message_name)
    VALUES ('attribute_value_dependencies', 'model', 'av_deps', 'AV Deps Summary');

CREATE INDEX ON attribute_value_dependencies(source_attribute_value_id);
CREATE INDEX ON attribute_value_dependencies(destination_attribute_value_id);

CREATE INDEX ON components(visibility_deleted_at) WHERE (visibility_deleted_at IS NOT NULL);
CREATE INDEX ON components(tenancy_workspace_pk);
CREATE INDEX ON attribute_values(visibility_deleted_at) WHERE (visibility_deleted_at IS NOT NULL);
CREATE INDEX ON attribute_values(tenancy_workspace_pk);
CREATE INDEX ON attribute_prototypes(visibility_deleted_at) WHERE (visibility_deleted_at IS NOT NULL);
CREATE INDEX ON attribute_prototypes(tenancy_workspace_pk);
CREATE INDEX ON attribute_prototype_arguments(visibility_deleted_at) WHERE (visibility_deleted_at IS NOT NULL);
CREATE INDEX ON attribute_prototype_arguments(tenancy_workspace_pk);
CREATE INDEX ON attribute_prototype_arguments(tail_component_id);
CREATE INDEX ON props(visibility_deleted_at) WHERE (visibility_deleted_at IS NOT NULL);
CREATE INDEX ON props(tenancy_workspace_pk);
CREATE INDEX ON prop_belongs_to_prop(visibility_deleted_at) WHERE (visibility_deleted_at IS NOT NULL);
CREATE INDEX ON prop_belongs_to_prop(tenancy_workspace_pk);
CREATE INDEX ON internal_providers(visibility_deleted_at) WHERE (visibility_deleted_at IS NOT NULL);
CREATE INDEX ON internal_providers(tenancy_workspace_pk);
CREATE INDEX ON external_providers(visibility_deleted_at) WHERE (visibility_deleted_at IS NOT NULL);
CREATE INDEX ON external_providers(tenancy_workspace_pk);

DROP INDEX IF EXISTS attribute_value_belongs_to_attribute_prototype_single_associati;
CREATE UNIQUE INDEX attribute_value_belongs_to_attribute_prototype_single_associati
    ON attribute_value_belongs_to_attribute_prototype (object_id, tenancy_workspace_pk, visibility_change_set_pk)
    WHERE visibility_deleted_at IS NULL;
DROP INDEX IF EXISTS attribute_value_belongs_to_attribute_value_single_association;
CREATE UNIQUE INDEX attribute_value_belongs_to_attribute_value_single_association
    ON attribute_value_belongs_to_attribute_value (object_id, tenancy_workspace_pk, visibility_change_set_pk)
    WHERE visibility_deleted_at IS NULL;
DROP INDEX IF EXISTS component_belongs_to_schema_single_association;
CREATE UNIQUE INDEX component_belongs_to_schema_single_association
    ON component_belongs_to_schema (object_id, tenancy_workspace_pk, visibility_change_set_pk)
    WHERE visibility_deleted_at IS NULL;
DROP INDEX IF EXISTS component_belongs_to_schema_variant_single_association;
CREATE UNIQUE INDEX component_belongs_to_schema_variant_single_association
    ON component_belongs_to_schema_variant (object_id, tenancy_workspace_pk, visibility_change_set_pk)
    WHERE visibility_deleted_at IS NULL;
DROP INDEX IF EXISTS fix_belongs_to_fix_batch_single_association;
CREATE UNIQUE INDEX fix_belongs_to_fix_batch_single_association
    ON fix_belongs_to_fix_batch (object_id, tenancy_workspace_pk, visibility_change_set_pk)
    WHERE visibility_deleted_at IS NULL;
DROP INDEX IF EXISTS func_binding_belongs_to_func_single_association;
CREATE UNIQUE INDEX func_binding_belongs_to_func_single_association
    ON func_binding_belongs_to_func (object_id, tenancy_workspace_pk, visibility_change_set_pk)
    WHERE visibility_deleted_at IS NULL;
DROP INDEX IF EXISTS node_belongs_to_component_single_association;
CREATE UNIQUE INDEX node_belongs_to_component_single_association
    ON node_belongs_to_component (object_id, tenancy_workspace_pk, visibility_change_set_pk)
    WHERE visibility_deleted_at IS NULL;
DROP INDEX IF EXISTS prop_belongs_to_prop_single_association;
CREATE UNIQUE INDEX prop_belongs_to_prop_single_association
    ON prop_belongs_to_prop (object_id, tenancy_workspace_pk, visibility_change_set_pk)
    WHERE visibility_deleted_at IS NULL;
DROP INDEX IF EXISTS schema_ui_menu_belongs_to_schema_single_association;
CREATE UNIQUE INDEX schema_ui_menu_belongs_to_schema_single_association
    ON schema_ui_menu_belongs_to_schema (object_id, tenancy_workspace_pk, visibility_change_set_pk)
    WHERE visibility_deleted_at IS NULL;
DROP INDEX IF EXISTS schema_variant_belongs_to_schema_single_association;
CREATE UNIQUE INDEX schema_variant_belongs_to_schema_single_association
    ON schema_variant_belongs_to_schema (object_id, tenancy_workspace_pk, visibility_change_set_pk)
    WHERE visibility_deleted_at IS NULL;
DROP INDEX IF EXISTS socket_belongs_to_external_provider_single_association;
CREATE UNIQUE INDEX socket_belongs_to_external_provider_single_association
    ON socket_belongs_to_external_provider (object_id, tenancy_workspace_pk, visibility_change_set_pk)
    WHERE visibility_deleted_at IS NULL;
DROP INDEX IF EXISTS socket_belongs_to_internal_provider_single_association;
CREATE UNIQUE INDEX socket_belongs_to_internal_provider_single_association
    ON socket_belongs_to_internal_provider (object_id, tenancy_workspace_pk, visibility_change_set_pk)
    WHERE visibility_deleted_at IS NULL;

CREATE OR REPLACE FUNCTION attribute_value_dependencies_update_v1(
    this_tenancy_workspace_pk     ident,
    this_visibility_change_set_pk ident,
    this_visibility_deleted_at    TIMESTAMP WITH TIME ZONE,
    this_destination_av_id        ident
) RETURNS VOID
AS
$$
DECLARE
    this_non_builtin_func  bool;
    this_current_av        attribute_values%ROWTYPE;
    this_destination_av    attribute_values%ROWTYPE;
    this_internal_provider internal_providers%ROWTYPE;
    this_source_av_id      ident;
    this_source_av_ids     ident[];
    this_summary_row_id    ident;
    this_tenancy           jsonb;
    this_visibility        jsonb;
BEGIN
    this_tenancy := jsonb_build_object('tenancy_workspace_pk', this_tenancy_workspace_pk);
    this_visibility := jsonb_build_object(
        'visibility_change_set_pk', this_visibility_change_set_pk,
        'visibility_deleted_at', this_visibility_deleted_at
    );

    SELECT *
    INTO this_destination_av
    FROM attribute_values_v1(this_tenancy, this_visibility) AS av
        INNER JOIN components_v1(this_tenancy, this_visibility) AS components
            ON components.id = av.attribute_context_component_id
    WHERE av.id = this_destination_av_id;

    -- If the AV (or its Component) is deleted, then we want to remove _ALL_ of its
    -- references in the graph, both incoming, and outgoing.
    IF this_destination_av IS NULL THEN
        FOR this_summary_row_id IN
            SELECT id
            FROM attribute_value_dependencies_v1(this_tenancy, this_visibility) AS avd
            WHERE avd.destination_attribute_value_id = this_destination_av_id
                OR avd.source_attribute_value_id = this_destination_av_id
        LOOP
            PERFORM delete_by_id_v1(
                'attribute_value_dependencies',
                this_tenancy,
                this_visibility,
                this_summary_row_id
            );
        END LOOP;
        RETURN;
    END IF;

    IF this_destination_av.attribute_context_prop_id != ident_nil_v1() THEN
        -- "Normal" AV depends on an InternalProvider
        SELECT array_agg(source_avs.id)
        INTO this_source_av_ids
        FROM (
            SELECT source_av.*
            FROM attribute_value_belongs_to_attribute_prototype_v1(this_tenancy, this_visibility) AS avbtap
                INNER JOIN attribute_prototype_arguments_v1(this_tenancy, this_visibility) AS apa
                    ON apa.attribute_prototype_id = avbtap.belongs_to_id
                INNER JOIN attribute_values_v1(this_tenancy, this_visibility) AS source_av
                    ON source_av.attribute_context_internal_provider_id = apa.internal_provider_id
                        AND source_av.attribute_context_component_id = this_destination_av.attribute_context_component_id
                -- Make sure we're not considering deleted Components as available sources.
                INNER JOIN components_v1(this_tenancy, this_visibility) AS components
                    ON components.id = source_av.attribute_context_component_id
            WHERE avbtap.object_id = this_destination_av_id
        ) AS source_avs;

        -- If the prototype's func isn't one of the 'si:set*', or 'si:unset', then it's a
        -- "dynamic" function. We also need to update the InternalProvider for any child props
        -- to have this AttributeValue as a dependency.
        SELECT funcs.name NOT LIKE 'si:set%' AND funcs.name != 'si:unset'
        INTO this_non_builtin_func
        FROM funcs_v1(this_tenancy, this_visibility) AS funcs
            INNER JOIN attribute_prototypes_v1(this_tenancy, this_visibility) AS ap
                ON ap.func_id = funcs.id
            INNER JOIN attribute_value_belongs_to_attribute_prototype_v1(this_tenancy, this_visibility) AS avbtap
                ON avbtap.belongs_to_id = ap.id
        WHERE avbtap.object_id = this_destination_av_id;
        IF this_non_builtin_func THEN
            -- What are the InternalProviders "below" this destination.
            FOR this_source_av_id IN
                WITH RECURSIVE child_props(id) AS (
                    SELECT this_destination_av.attribute_context_prop_id AS id
                    UNION
                    SELECT pbtp.object_id AS id
                    FROM prop_belongs_to_prop_v1(this_tenancy, this_visibility) AS pbtp
                        INNER JOIN child_props ON child_props.id = pbtp.belongs_to_id
                )
                SELECT av.id
                FROM attribute_values_v1(this_tenancy, this_visibility) AS av
                    INNER JOIN internal_providers_v1(this_tenancy, this_visibility) AS ip
                        ON ip.id = av.attribute_context_internal_provider_id
                    INNER JOIN child_props ON ip.prop_id = child_props.id
                WHERE av.attribute_context_component_id = this_destination_av.attribute_context_component_id
            LOOP
                -- Have them re-update their inputs to include us.
                PERFORM attribute_value_dependencies_update_v1(
                    this_tenancy_workspace_pk,
                    this_visibility_change_set_pk,
                    this_visibility_deleted_at,
                    this_source_av_id
                );
            END LOOP;
        END IF;
    ELSIF this_destination_av.attribute_context_internal_provider_id != ident_nil_v1() THEN
        SELECT *
        INTO this_internal_provider
        FROM internal_providers_v1(this_tenancy, this_visibility)
        WHERE id = this_destination_av.attribute_context_internal_provider_id;

        IF this_internal_provider.prop_id = ident_nil_v1() THEN
            -- Explicit InternalProvider depends on another component's ExternalProvider
            SELECT array_agg(source_avs.id)
            INTO this_source_av_ids
            FROM (
                SELECT source_av.*
                FROM attribute_values_v1(this_tenancy, this_visibility) AS source_av
                    INNER JOIN attribute_prototype_arguments_v1(this_tenancy, this_visibility) AS apa
                        ON apa.external_provider_id = source_av.attribute_context_external_provider_id
                            AND source_av.attribute_context_component_id = apa.tail_component_id
                    INNER JOIN attribute_value_belongs_to_attribute_prototype_v1(this_tenancy, this_visibility) AS avbtap
                        ON avbtap.belongs_to_id = apa.attribute_prototype_id
                    INNER JOIN attribute_values_v1(this_tenancy, this_visibility) AS destination_av
                        ON destination_av.id = avbtap.object_id
                    INNER JOIN components_v1(this_tenancy, this_visibility) AS components
                        ON components.id = source_av.attribute_context_component_id
                WHERE destination_av.id = this_destination_av_id
                    AND apa.head_component_id = destination_av.attribute_context_component_id
            ) AS source_avs;
        ELSE
            -- Implicit InternalProvider
            SELECT array_agg(source_avs.id)
            INTO this_source_av_ids
            FROM attribute_value_dependencies_ip_av_sources_v1(
                this_tenancy,
                this_visibility,
                this_internal_provider.prop_id,
                this_destination_av.attribute_context_component_id
            ) AS source_avs(id);

            -- Implicit InternalProvider's _can_ also depend on the AttributeValue for a parent Prop,
            -- if that parent Prop is populated by a dynamic function. The function on the parent Prop's
            -- AttributeValue could populate a complex object that would end up populating our Prop's
            -- AttributeValue.
            SELECT av.*
            INTO this_current_av
            FROM attribute_values_v1(this_tenancy, this_visibility) AS av
            WHERE av.attribute_context_prop_id = this_internal_provider.prop_id
                AND av.attribute_context_component_id = this_destination_av.attribute_context_component_id;

            LOOP
                -- Break out if we ran out of parent AVs to look at.
                IF this_current_av.id IS NULL THEN
                    EXIT;
                END IF;

                -- Check if this AV is from a "dynamic" function (one that has arguments). If it is, then
                -- it should be considered as a source for this InternalProvider, as it will be populating
                -- **ALL** Prop AttributeValues below it (and we are "below" it).
                SELECT funcs.name NOT LIKE 'si:set%' AND funcs.name != 'si:unset'
                INTO this_non_builtin_func
                FROM funcs_v1(this_tenancy, this_visibility) AS funcs
                    INNER JOIN attribute_prototypes_v1(this_tenancy, this_visibility) AS ap
                        ON ap.func_id = funcs.id
                    INNER JOIN attribute_value_belongs_to_attribute_prototype_v1(this_tenancy, this_visibility) AS avbtap
                        ON avbtap.belongs_to_id = ap.id
                WHERE avbtap.object_id = this_current_av.id;
                IF this_non_builtin_func THEN
                    this_source_av_ids := array_append(this_source_av_ids, this_current_av.id);
                    -- Break out of the loop, since "dynamic" functions can't live below other dynamic
                    -- functions.
                    EXIT;
                END IF;

                -- Walk up to the parent AV.
                SELECT av.*
                INTO this_current_av
                FROM attribute_values_v1(this_tenancy, this_visibility) AS av
                    INNER JOIN attribute_value_belongs_to_attribute_value_v1(this_tenancy, this_visibility) AS avbtav
                        ON avbtav.belongs_to_id = av.id
                WHERE avbtav.object_id = this_current_av.id;
            END LOOP;
        END IF;
    ELSIF this_destination_av.attribute_context_external_provider_id != ident_nil_v1() THEN
        -- ExternalProvider AV depends on an InternalProvider
        SELECT array_agg(source_avs.id)
        INTO this_source_av_ids
        FROM (
            SELECT source_av.*
            FROM attribute_value_belongs_to_attribute_prototype_v1(this_tenancy, this_visibility) AS avbtap
                INNER JOIN attribute_prototype_arguments_v1(this_tenancy, this_visibility) AS apa
                    ON apa.attribute_prototype_id = avbtap.belongs_to_id
                INNER JOIN attribute_values_v1(this_tenancy, this_visibility) AS source_av
                    ON source_av.attribute_context_internal_provider_id = apa.internal_provider_id
                -- Make sure we're not considering deleted Components as available sources.
                INNER JOIN components_v1(this_tenancy, this_visibility) AS components
                    ON components.id = source_av.attribute_context_component_id
            WHERE avbtap.object_id = this_destination_av_id
                AND source_av.attribute_context_component_id = this_destination_av.attribute_context_component_id
        ) AS source_avs;
    END IF;

    -- Remove any links that are no longer relevant
    FOR this_summary_row_id IN
        SELECT avd.id
        FROM attribute_value_dependencies_v1(this_tenancy, this_visibility) AS avd
        WHERE avd.destination_attribute_value_id = this_destination_av_id
            AND avd.source_attribute_value_id != ALL(this_source_av_ids)
    LOOP
        PERFORM delete_by_id_v1(
            'attribute_value_dependencies',
            this_tenancy,
            this_visibility,
            this_summary_row_id
        );
    END LOOP;

    -- Add the new links
    FOR this_source_av_id IN
        SELECT DISTINCT new_sources.id
        FROM unnest(this_source_av_ids) AS new_sources(id)
            LEFT JOIN attribute_value_dependencies_v1(this_tenancy, this_visibility) AS avd
                ON avd.source_attribute_value_id = new_sources.id
                    AND avd.destination_attribute_value_id = this_destination_av_id
        WHERE avd.id IS NULL
    LOOP
        INSERT INTO attribute_value_dependencies (
            tenancy_workspace_pk,
            visibility_change_set_pk,
            source_attribute_value_id,
            destination_attribute_value_id
        )
        VALUES (
            this_tenancy_workspace_pk,
            this_visibility_change_set_pk,
            this_source_av_id,
            this_destination_av_id
        );
    END LOOP;
END
$$ LANGUAGE PLPGSQL VOLATILE;

CREATE OR REPLACE FUNCTION attribute_value_dependencies_ip_av_sources_v1(
    this_tenancy                 jsonb,
    this_visibility              jsonb,
    this_ip_prop_id              ident,
    this_ip_component_id         ident
) RETURNS TABLE (source_av_id ident)
AS
$$
  SELECT source_avs.id
  FROM (
      -- Implicit InternalProvider depends on associated Prop's AttributeValue and on
      -- all of the descendant AttributeValues of the associated Prop's AttributeValue.
      WITH RECURSIVE descendant_avs AS (
          SELECT source_av.*
          FROM attribute_values_v1(this_tenancy, this_visibility) AS source_av
              WHERE source_av.attribute_context_prop_id = this_ip_prop_id
                  AND source_av.attribute_context_component_id = this_ip_component_id
          UNION ALL
          SELECT source_av.*
          FROM attribute_values_v1(this_tenancy, this_visibility) AS source_av
              INNER JOIN attribute_value_belongs_to_attribute_value_v1(this_tenancy, this_visibility) AS avbtav
                  ON avbtav.object_id = source_av.id
              INNER JOIN descendant_avs
                  ON descendant_avs.id = avbtav.belongs_to_id
      )
      SELECT descendant_avs.* FROM descendant_avs
      UNION ALL
      -- Implicit InternalProvider depends on InternalProviders of all of associated Props's child
      -- Props
      SELECT source_av.*
      FROM prop_belongs_to_prop_v1(this_tenancy, this_visibility) AS pbtp
          INNER JOIN internal_providers_v1(this_tenancy, this_visibility) AS ip
              ON pbtp.object_id = ip.prop_id
          INNER JOIN attribute_values_v1(this_tenancy, this_visibility) AS source_av
              ON ip.id = source_av.attribute_context_internal_provider_id
                  AND source_av.attribute_context_component_id = this_ip_component_id
      WHERE pbtp.belongs_to_id = this_ip_prop_id
  ) AS source_avs
      -- Make sure we're not considering deleted Components as available sources.
      INNER JOIN components_v1(this_tenancy, this_visibility) AS components
          ON components.id = source_avs.attribute_context_component_id
$$ LANGUAGE SQL VOLATILE;

CREATE OR REPLACE FUNCTION attribute_value_dependency_graph_v1(
    this_tenancy                  jsonb,
    this_visibility               jsonb,
    this_attribute_value_id_list  ident[]
)
    RETURNS TABLE
        (
            attribute_value_id           ident,
            dependent_attribute_value_id ident
        )
AS
$$
    WITH RECURSIVE dependency_graph AS (
      SELECT
          destination_attribute_value_id AS attribute_value_id,
          source_attribute_value_id AS dependent_attribute_value_id
      FROM attribute_value_dependencies_v1(this_tenancy, this_visibility) AS avd
      WHERE source_attribute_value_id = ANY(this_attribute_value_id_list)
      UNION
      SELECT
          destination_attribute_value_id AS attribute_value_id,
          source_attribute_value_id AS dependent_attribute_value_id
      FROM dependency_graph
          INNER JOIN attribute_value_dependencies_v1(this_tenancy, this_visibility) AS avd
              ON dependency_graph.attribute_value_id = avd.source_attribute_value_id
    )
    SELECT * FROM dependency_graph
$$ LANGUAGE SQL;

CREATE OR REPLACE FUNCTION attribute_value_dependencies_update_component_v1(
    this_tenancy jsonb,
    this_visibility jsonb,
    this_component_id ident
) RETURNS VOID
AS
$$
DECLARE
    this_attribute_value   attribute_values%ROWTYPE;
BEGIN
    FOR this_attribute_value IN
        SELECT av.*
        FROM attribute_values_v1(this_tenancy, this_visibility) AS av
        WHERE attribute_context_component_id = this_component_id
    LOOP
        PERFORM attribute_value_dependencies_update_v1(
            this_attribute_value.tenancy_workspace_pk,
            this_attribute_value.visibility_change_set_pk,
            this_attribute_value.visibility_deleted_at,
            this_attribute_value.id
        );
    END LOOP;
END
$$ LANGUAGE PLPGSQL VOLATILE;

CREATE OR REPLACE FUNCTION component_create_v2(
    this_tenancy jsonb,
    this_visibility jsonb,
    this_user_pk ident,
    this_kind text,
    this_schema_variant_id ident,
    OUT object json) AS
$$
DECLARE
    this_tenancy_record                     tenancy_record_v1;
    this_visibility_record                  visibility_record_v1;
    this_attribute_context                  jsonb;
    this_attribute_prototype                RECORD;
    this_attribute_value_id                 ident;
    this_external_provider                  RECORD;
    this_internal_provider                  RECORD;
    this_new_attribute_value                jsonb;
    this_new_attribute_value_id             ident;
    this_parent_attribute_value_id          ident;
    this_prop_attribute_value               RECORD;
    this_schema_id                          ident;
    this_unset_func_binding_id              ident;
    this_unset_func_binding_return_value_id ident;
    this_unset_func_id                      ident;
    this_new_row                            components%ROWTYPE;
BEGIN
    this_tenancy_record := tenancy_json_to_columns_v1(this_tenancy);
    this_visibility_record := visibility_json_to_columns_v1(this_visibility);

    INSERT INTO components (tenancy_workspace_pk,
                            visibility_change_set_pk, kind, creation_user_pk)
    VALUES (this_tenancy_record.tenancy_workspace_pk,
            this_visibility_record.visibility_change_set_pk, this_kind,
            this_user_pk)
    RETURNING * INTO this_new_row;

    -- Create unset AttributeValues for the ExternalProviders, InternalProviders,
    -- and for the Props starting at the root prop, up until (and including) the
    -- first Array/Map that is encountered. These will be place holders for
    -- when we set values (such as the root.si.name), and do function evaluation
    -- later on.
    SELECT belongs_to_id
    INTO STRICT this_schema_id
    FROM schema_variant_belongs_to_schema
    WHERE in_tenancy_and_visible_v1(this_tenancy, this_visibility, schema_variant_belongs_to_schema)
        AND object_id = this_schema_variant_id;

    PERFORM set_belongs_to_v1(
      'component_belongs_to_schema',
      this_tenancy,
      this_visibility,
      this_new_row.id,
      this_schema_id
    );
    PERFORM set_belongs_to_v1(
      'component_belongs_to_schema_variant',
      this_tenancy,
      this_visibility,
      this_new_row.id,
      this_schema_variant_id
    );

    -- Find the "si:unset" Func Binding, and Func Binding Return Value to use
    -- when creating the Attribute Values for the External & Internal Providers.
    SELECT id
    INTO this_unset_func_id
    FROM find_by_attr_v1('funcs',
                         this_tenancy,
                         this_visibility,
                         'name',
                         'si:unset');
    IF this_unset_func_id IS NULL THEN
        RAISE 'attribute_value_insert_for_context_raw_v1: Unable to find Func(%) in Tenancy(%), Visibility(%)',
              'si:unset',
              this_tenancy,
              this_visibility;
    END IF;
    SELECT new_func_binding_id, new_func_binding_return_value_id
    INTO this_unset_func_binding_id, this_unset_func_binding_return_value_id
    FROM func_binding_create_and_execute_v1(
      this_tenancy,
      this_visibility,
      'null'::jsonb,
      this_unset_func_id
    );

    -- External Providers
    FOR this_external_provider IN
        SELECT *
        FROM external_providers_v1(this_tenancy, this_visibility)
        WHERE schema_variant_id = this_schema_variant_id
    LOOP
        this_attribute_context := attribute_context_build_from_parts_v1(
            ident_nil_v1(), -- Prop ID
            ident_nil_v1(), -- Internal Provider ID
            this_external_provider.id, -- External Provider ID
            -- We won't find a component-specific prototype, since the component
            -- didn't exist before calling this function, but we'll want the
            -- component ID set when we go to create the Attribute Value.
            this_new_row.id -- Component ID
        );

        SELECT *
        INTO STRICT this_attribute_prototype
        FROM attribute_prototypes_v1(this_tenancy, this_visibility) AS ap
        WHERE in_attribute_context_v1(this_attribute_context, ap);

        SELECT av.object
        INTO this_new_attribute_value
        FROM attribute_value_create_v1(
            this_tenancy,
            this_visibility,
            this_attribute_context,
            this_unset_func_binding_id,
            this_unset_func_binding_return_value_id,
            NULL
        ) AS av;

        PERFORM set_belongs_to_v1(
            'attribute_value_belongs_to_attribute_prototype',
            this_tenancy,
            this_visibility,
            this_new_attribute_value ->> 'id',
            this_attribute_prototype.id
        );
    END LOOP;

    -- Explicit Internal Providers
    FOR this_internal_provider IN
        SELECT *
        FROM internal_providers_v1(this_tenancy, this_visibility)
        WHERE schema_variant_id = this_schema_variant_id
    LOOP
        this_attribute_context := attribute_context_build_from_parts_v1(
            ident_nil_v1(), -- Prop ID
            this_internal_provider.id, -- Internal Provider ID
            ident_nil_v1(), -- External Provider ID
            this_new_row.id -- Component ID
        );

        SELECT *
        INTO STRICT this_attribute_prototype
        FROM attribute_prototypes_v1(this_tenancy, this_visibility) AS ap
        WHERE in_attribute_context_v1(this_attribute_context, ap);

        SELECT av.object
        INTO this_new_attribute_value
        FROM attribute_value_create_v1(
            this_tenancy,
            this_visibility,
            this_attribute_context,
            this_unset_func_binding_id,
            this_unset_func_binding_return_value_id,
            NULL
        ) AS av;

        PERFORM set_belongs_to_v1(
            'attribute_value_belongs_to_attribute_prototype',
            this_tenancy,
            this_visibility,
            this_new_attribute_value ->> 'id',
            this_attribute_prototype.id
        );
    END LOOP;

    -- Implicit Internal Providers
    FOR this_internal_provider IN
        SELECT ip.*
        FROM internal_providers_v1(this_tenancy, this_visibility) AS ip
            INNER JOIN props_v1(this_tenancy, this_visibility) AS props
              ON ip.prop_id = props.id
        WHERE props.schema_variant_id = this_schema_variant_id
    LOOP
        -- Create an Attribute Value for the Internal Provider
        this_attribute_context := attribute_context_build_from_parts_v1(
            ident_nil_v1(), -- Prop ID
            this_internal_provider.id, -- Internal Provider ID
            ident_nil_v1(), -- External Provider ID
            this_new_row.id -- Component ID
        );

        SELECT *
        INTO STRICT this_attribute_prototype
        FROM attribute_prototypes_v1(this_tenancy, this_visibility) AS ap
        WHERE in_attribute_context_v1(this_attribute_context, ap);

        SELECT av.object
        INTO this_new_attribute_value
        FROM attribute_value_create_v1(
            this_tenancy,
            this_visibility,
            this_attribute_context,
            this_unset_func_binding_id,
            this_unset_func_binding_return_value_id,
            NULL
        ) AS av;

        PERFORM set_belongs_to_v1(
            'attribute_value_belongs_to_attribute_prototype',
            this_tenancy,
            this_visibility,
            this_new_attribute_value ->> 'id',
            this_attribute_prototype.id
        );

        -- Create an Attribute Value for the Prop.
        this_attribute_context := attribute_context_build_from_parts_v1(
            this_internal_provider.prop_id, -- Prop ID
            ident_nil_v1(), -- Internal Provider ID
            ident_nil_v1(), -- External Provider ID
            this_new_row.id -- Component ID
        );

        SELECT *
        INTO this_attribute_prototype
        FROM attribute_prototypes_v1(this_tenancy, this_visibility) AS ap
        WHERE in_attribute_context_v1(this_attribute_context, ap)
        ORDER BY id DESC
        LIMIT 1;

        -- See what the func_binding & func_binding_return_value are on the
        -- prop-specific Attribute Value, and copy those over.
        SELECT *
        INTO STRICT this_prop_attribute_value
        FROM attribute_values_v1(this_tenancy, this_visibility) AS av
        WHERE in_attribute_context_v1(
            attribute_context_build_from_parts_v1(
                this_internal_provider.prop_id,
                ident_nil_v1(),
                ident_nil_v1(),
                ident_nil_v1()
            ),
            av
        );

        SELECT av.object
        INTO this_new_attribute_value
        FROM attribute_value_create_v1(
            this_tenancy,
            this_visibility,
            this_attribute_context,
            this_prop_attribute_value.func_binding_id,
            this_prop_attribute_value.func_binding_return_value_id,
            NULL
        ) AS av;

        PERFORM set_belongs_to_v1(
            'attribute_value_belongs_to_attribute_prototype',
            this_tenancy,
            this_visibility,
            this_new_attribute_value ->> 'id',
            this_attribute_prototype.id
        );
    END LOOP;

    -- Some map Props have entries for specific keys as part of the Schema
    -- Variant's definition. This should only be happening for things like
    -- qualifications, and code-gen, which means that it should only ever be
    -- happening for the first-level map encountered from the root, when it
    -- happens at all.
    FOR this_prop_attribute_value IN
        SELECT av.*
        FROM attribute_values_v1(this_tenancy, this_visibility) AS av
            INNER JOIN props_v1(this_tenancy, this_visibility) AS props
                ON av.attribute_context_prop_id = props.id
        WHERE props.schema_variant_id = this_schema_variant_id
            AND av.key IS NOT NULL
            AND av.attribute_context_component_id = ident_nil_v1()
    LOOP
        this_attribute_context := attribute_context_build_from_parts_v1(
            this_prop_attribute_value.attribute_context_prop_id,
            ident_nil_v1(),
            ident_nil_v1(),
            this_new_row.id
        );

        SELECT ap.*
        INTO STRICT this_attribute_prototype
        FROM attribute_prototypes_v1(this_tenancy, this_visibility) AS ap
            INNER JOIN attribute_value_belongs_to_attribute_prototype_v1(this_tenancy, this_visibility) AS avbtap
                ON ap.id = avbtap.belongs_to_id
        WHERE avbtap.object_id = this_prop_attribute_value.id;

        SELECT av.object
        INTO this_new_attribute_value
        FROM attribute_value_create_v1(
            this_tenancy,
            this_visibility,
            this_attribute_context,
            this_prop_attribute_value.func_binding_id,
            this_prop_attribute_value.func_binding_return_value_id,
            this_prop_attribute_value.key
        ) AS av;

        PERFORM set_belongs_to_v1(
            'attribute_value_belongs_to_attribute_prototype',
            this_tenancy,
            this_visibility,
            this_new_attribute_value ->> 'id',
            this_attribute_prototype.id
        );
    END LOOP;

    -- We need to create the attribute_value_belongs_to_attribute_value
    -- relationship for the Prop Attribute Values of the Component. We are doing
    -- this after all of the Attribute Values have been created because we're
    -- guaranteeing that they're created in topographical order, which prevents
    -- us from setting the belongs_to relationship as we go along.
    this_attribute_context := attribute_context_build_from_parts_v1(
        NULL, -- Prop ID
        ident_nil_v1(), -- Internal Provider ID
        ident_nil_v1(), -- External Provider ID
        this_new_row.id -- Component ID
    );
    FOR this_parent_attribute_value_id, this_attribute_value_id IN
        WITH RECURSIVE avbtav(parent_av_id, av_id) AS (
            SELECT parent_av.id, av.id
            FROM prop_belongs_to_prop AS pbtp
            INNER JOIN attribute_values_v1(this_tenancy, this_visibility) AS av
                ON av.attribute_context_prop_id = pbtp.object_id
            INNER JOIN attribute_values_v1(this_tenancy, this_visibility) AS parent_av
                ON parent_av.attribute_context_prop_id = pbtp.belongs_to_id
            WHERE in_attribute_context_v1(this_attribute_context, av)
                AND av.attribute_context_component_id = this_new_row.id
                AND in_attribute_context_v1(this_attribute_context, parent_av)
                AND parent_av.attribute_context_component_id = this_new_row.id
        )
        SELECT * FROM avbtav
    LOOP
        PERFORM set_belongs_to_v1(
            'attribute_value_belongs_to_attribute_value',
            this_tenancy,
            this_visibility,
            this_attribute_value_id,
            this_parent_attribute_value_id
        );
    END LOOP;

    -- Make sure we've populated the dependency graph for this (new) component.
    FOR this_new_attribute_value_id IN
        SELECT av.id
        FROM attribute_values_v1(this_tenancy, this_visibility) AS av
        WHERE av.attribute_context_component_id = this_new_row.id
    LOOP
        PERFORM attribute_value_dependencies_update_v1(
            this_tenancy_record.tenancy_workspace_pk,
            this_visibility_record.visibility_change_set_pk,
            this_visibility_record.visibility_deleted_at,
            this_new_attribute_value_id
        );
    END LOOP;

    -- Create a parallel record to store creation and update status, meaning that this table's id refers to components.id
    INSERT INTO component_statuses (id,
                                    tenancy_workspace_pk,
                                    visibility_change_set_pk,
                                    creation_user_pk, update_user_pk)
    VALUES (this_new_row.id,
            this_new_row.tenancy_workspace_pk,
            this_new_row.visibility_change_set_pk,
            this_user_pk, this_user_pk);

    object := row_to_json(this_new_row);
END;
$$ LANGUAGE PLPGSQL VOLATILE;

CREATE OR REPLACE FUNCTION attribute_value_vivify_value_and_parent_values_raw_v1(
    this_tenancy               jsonb,
    this_visibility            jsonb,
    this_attribute_context     jsonb,
    this_attribute_value_id    ident,
    this_create_child_proxies  bool,
    OUT new_attribute_value_id ident
)
AS
$$
DECLARE
    attribute_value                    attribute_values%ROWTYPE;
    prop                               props%ROWTYPE;
    empty_value                        jsonb;
    unset_func_id                      ident;
    func_id                            ident;
    maybe_parent_attribute_value_id    ident;
    schema_variant_attribute_value     attribute_values%ROWTYPE;
    schema_variant_attribute_prototype attribute_prototypes%ROWTYPE;
    schema_variant_func                funcs%ROWTYPE;
BEGIN
    RAISE DEBUG 'attribute_value_vivify_value_and_parent_values_raw_v1(%, %, %, %, %)',
        this_tenancy,
        this_visibility,
        this_attribute_context,
        this_attribute_value_id,
        this_create_child_proxies;
    SELECT *
    INTO attribute_value
    FROM attribute_values_v1(this_tenancy, this_visibility) AS av
    WHERE id = this_attribute_value_id
    ORDER BY id;
    IF NOT FOUND THEN
        RAISE 'Unable to find AttributeValue(%) with Tenancy(%) and Visibility(%)', this_attribute_value_id,
                                                                                    this_tenancy,
                                                                                    this_visibility;
    END IF;

    -- If this value is for a Component, then we should check to see what func the version for the
    -- SchemaVariant is using, and preserve that (and the value), if it's something other
    -- than si:unset.
    IF attribute_value.attribute_context_component_id != ident_nil_v1() THEN
        SELECT *
        INTO schema_variant_attribute_value
        FROM attribute_values_v1(this_tenancy, this_visibility) AS av
        WHERE av.attribute_context_prop_id = attribute_value.attribute_context_prop_id
            AND av.attribute_context_internal_provider_id = attribute_value.attribute_context_internal_provider_id
            AND av.attribute_context_external_provider_id = attribute_value.attribute_context_external_provider_id
            AND av.attribute_context_component_id = ident_nil_v1()
            AND (av.key = attribute_value.key OR (av.key IS NULL AND attribute_value.key IS NULL));

        IF schema_variant_attribute_value.id IS NOT NULL THEN
            SELECT ap.*
            INTO schema_variant_attribute_prototype
            FROM attribute_prototypes_v1(this_tenancy, this_visibility) AS ap
                INNER JOIN attribute_value_belongs_to_attribute_prototype_v1(this_tenancy, this_visibility) AS avbtap
                    ON avbtap.belongs_to_id = ap.id
            WHERE avbtap.object_id = schema_variant_attribute_value.id;

            SELECT *
            INTO schema_variant_func
            FROM funcs_v1(this_tenancy, this_visibility) AS funcs
            WHERE funcs.id = schema_variant_attribute_prototype.func_id;
        END IF;
    END IF;

    SELECT *
    INTO prop
    FROM props_v1(this_tenancy, this_visibility) AS p
    WHERE id = attribute_value.attribute_context_prop_id;
    -- If the AttributeValue isn't for a Prop, check if it's for an InternalProvider, and grab the
    -- associated Prop.
    IF NOT FOUND THEN
        SELECT p.*
        INTO prop
        FROM props_v1(this_tenancy, this_visibility) AS p
        INNER JOIN internal_providers_v1(this_tenancy, this_visibility) AS ip
            ON ip.prop_id = p.id
        WHERE ip.id = attribute_value.attribute_context_internal_provider_id;
    END IF;
    -- If the AttributeValue isn't for a Prop, or an Internal Provider, then the only thing left
    -- is an ExternalProvider, which doesn't have a Prop at all. Pretend that the `prop.kind` is
    -- `map` for the purposes of creating the placeholder value.
    IF NOT FOUND THEN
        prop.kind := 'map';
    END IF;

    CASE
        WHEN prop.kind = 'array' THEN
            empty_value := '[]'::jsonb;
        WHEN prop.kind = 'object' OR prop.kind = 'map' THEN
            empty_value := '{}'::jsonb;
        ELSE
            -- Everything else isn't a container, so the "empty" version is the "unset" value.
            empty_value := NULL;
    END CASE;

    SELECT belongs_to_id
    INTO STRICT func_id
    FROM func_binding_belongs_to_func_v1(this_tenancy, this_visibility)
    WHERE object_id = attribute_value.func_binding_id;

    SELECT id
    INTO unset_func_id
    FROM find_by_attr_v1(
        'funcs',
        this_tenancy,
        this_visibility,
        'name',
        'si:unset'
    );

    -- If the AttributeValue is already set, there might not be anything for us to do.
    IF
        -- The AttributeValue must already be set to something other than "unset"
        func_id != unset_func_id AND exact_attribute_context_v1(this_attribute_context, attribute_value)
    THEN
        RAISE DEBUG 'attribute_value_vivify_value_and_parent_values_raw_v1: Re-using AttributeValue(%) for PropKind(%)',
            attribute_value.id,
            prop.kind;
        new_attribute_value_id := attribute_value.id;
        RETURN;
    END IF;

    SELECT belongs_to_id
    INTO maybe_parent_attribute_value_id
    FROM attribute_value_belongs_to_attribute_value_v1(this_tenancy, this_visibility) AS avbtav
    WHERE object_id = attribute_value.id;

    RAISE DEBUG 'attribute_value_vivify_value_and_parent_values_raw_v1: Update for context on AttributeValue(%) in AttributeContext(%)',
        attribute_value.id,
        this_attribute_context;
    new_attribute_value_id := attribute_value_update_for_context_raw_v1(
        this_tenancy,
        this_visibility,
        attribute_value.id,
        maybe_parent_attribute_value_id,
        this_attribute_context,
        empty_value,
        null,
        this_create_child_proxies
    );

    IF
        new_attribute_value_id != attribute_value.id
        -- Providers don't have Proxy values, only AttributeValues directly for Props.
        AND (this_attribute_context ->> 'attribute_context_prop_id')::ident != ident_nil_v1()
    THEN
        PERFORM update_by_id_v1(
            'attribute_values',
            'proxy_for_attribute_value_id',
            this_tenancy,
            this_visibility,
            new_attribute_value_id,
            attribute_value.id
        );
    END IF;

    -- If this value is for a Component, then we should check to see what func the version for the
    -- SchemaVariant is using, and preserve that (and the value), if it's something other
    -- than si:unset.
    IF schema_variant_func.id IS NOT NULL
        AND schema_variant_func.name != 'si:unset'
    THEN
        PERFORM unset_belongs_to_v1(
            'attribute_value_belongs_to_attribute_prototype',
            this_tenancy,
            this_visibility,
            new_attribute_value_id
        );
        PERFORM set_belongs_to_v1(
            'attribute_value_belongs_to_attribute_prototype',
            this_tenancy,
            this_visibility,
            new_attribute_value_id,
            schema_variant_attribute_prototype.id
        );
        PERFORM update_by_id_v1(
            'attribute_values',
            'func_binding_id',
            this_tenancy,
            this_visibility,
            new_attribute_value_id,
            schema_variant_attribute_value.func_binding_id
        );
        PERFORM update_by_id_v1(
            'attribute_values',
            'func_binding_return_value_id',
            this_tenancy,
            this_visibility,
            new_attribute_value_id,
            schema_variant_attribute_value.func_binding_return_value_id
        );
    END IF;
END;
$$ LANGUAGE PLPGSQL;

CREATE OR REPLACE FUNCTION component_delete_and_propagate_v2(
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
    this_attribute_value    ident;
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
            this_tenancy ->> 'tenancy_workspace_pk',
            this_visibility ->> 'visibility_change_set_pk',
            this_visibility ->> 'visibility_deleted_at',
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

CREATE OR REPLACE FUNCTION component_restore_and_propagate_v3(
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
    table_name                   text;
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
        SELECT e.pk, e.head_object_id, sbtip.belongs_to_id, sbtep.belongs_to_id
        FROM edges_v1(this_tenancy, this_visibility_with_deleted) e
                 LEFT JOIN socket_belongs_to_internal_provider sbtip ON sbtip.object_id = e.head_socket_id
                 LEFT JOIN socket_belongs_to_external_provider sbtep ON sbtep.object_id = e.head_socket_id
        WHERE tail_object_id = this_component_id
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

    -- Update the dependency graph for the "restored" Component.
    PERFORM attribute_value_dependencies_update_component_v1(
        this_tenancy,
        this_visibility,
        this_component_id
    );

    -- Update the dependency graphs of all Components that used this Component.
    FOREACH peer_component_id IN ARRAY peer_component_ids LOOP
        PERFORM attribute_value_dependencies_update_component_v1(
            this_tenancy,
            this_visibility,
            peer_component_id
        );
    END LOOP;
END;
$$ LANGUAGE PLPGSQL STABLE;

CREATE OR REPLACE FUNCTION attribute_prototype_argument_create_v2(
    this_tenancy jsonb,
    this_visibility jsonb,
    this_attribute_prototype_argument_id ident,
    this_func_argument_id ident,
    this_internal_provider_id ident,
    this_external_provider_id ident,
    this_tail_component_id ident,
    this_head_component_id ident,
    OUT object json) AS
$$
DECLARE
    this_attribute_value   attribute_values%ROWTYPE;
    this_new_row           attribute_prototype_arguments%ROWTYPE;
    this_tenancy_record    tenancy_record_v1;
    this_visibility_record visibility_record_v1;
BEGIN
    this_tenancy_record := tenancy_json_to_columns_v1(this_tenancy);
    this_visibility_record := visibility_json_to_columns_v1(this_visibility);

    INSERT INTO attribute_prototype_arguments (tenancy_workspace_pk,
                                               visibility_change_set_pk,
                                               attribute_prototype_id,
                                               func_argument_id,
                                               internal_provider_id,
                                               external_provider_id,
                                               tail_component_id,
                                               head_component_id)
    VALUES (this_tenancy_record.tenancy_workspace_pk,
            this_visibility_record.visibility_change_set_pk,
            this_attribute_prototype_argument_id,
            this_func_argument_id,
            this_internal_provider_id,
            this_external_provider_id,
            this_tail_component_id,
            this_head_component_id)
    RETURNING * INTO this_new_row;

    IF this_head_component_id IS NOT NULL THEN
        PERFORM attribute_value_dependencies_update_component_v1(
            this_tenancy,
            this_visibility,
            this_head_component_id
        );
    ELSE
        FOR this_attribute_value IN
            SELECT av.*
            FROM attribute_values_v1(this_tenancy, this_visibility) AS av
                INNER JOIN attribute_value_belongs_to_attribute_prototype_v1(this_tenancy, this_visibility) AS avbtap
                    ON avbtap.object_id = av.id
            WHERE avbtap.belongs_to_id = this_new_row.id
        LOOP
            PERFORM attribute_value_dependencies_update_v1(
                this_tenancy ->> 'tenancy_workspace_pk',
                this_visibility ->> 'visibility_change_set_pk',
                this_visibility ->> 'visibility_deleted_at',
                this_attribute_value.id
            );
        END LOOP;
    END IF;

    RAISE DEBUG 'attribute_prototype_argument_create_v1: Created AttributePrototypeArgument(%)', this_new_row;

    object := row_to_json(this_new_row);
END;
$$ LANGUAGE PLPGSQL VOLATILE;

CREATE OR REPLACE FUNCTION belongs_to_table_create_v1(this_table_name text,
                                                      this_object_table text,
                                                      this_belongs_to_table text)
    RETURNS VOID
AS
$$
DECLARE
    create_table text;
BEGIN
    create_table := format('CREATE TABLE %1$I ( '
                           ' pk                          ident primary key default ident_create_v1(), '
                           ' id                          ident not null default ident_create_v1(), '
                           ' object_id                   ident                   NOT NULL, '
                           ' belongs_to_id               ident                   NOT NULL, '
                           ' tenancy_workspace_pk        ident, '
                           ' visibility_change_set_pk    ident                   NOT NULL DEFAULT ident_nil_v1(), '
                           ' visibility_deleted_at       timestamp with time zone, '
                           ' created_at                  timestamp with time zone NOT NULL DEFAULT CLOCK_TIMESTAMP(), '
                           ' updated_at                  timestamp with time zone NOT NULL DEFAULT CLOCK_TIMESTAMP() '
                           '); '
                           'CREATE UNIQUE INDEX %1$s_visibility_tenancy ON %1$I (id, '
                           '                                    tenancy_workspace_pk, '
                           '                                    visibility_change_set_pk); '
                           'ALTER TABLE %1$I '
                           '    ADD CONSTRAINT %1$s_object_id_is_valid '
                           '        CHECK (check_id_in_table_v1(%2$L, object_id)); '
                           'ALTER TABLE %1$I '
                           '    ADD CONSTRAINT %1$s_belongs_to_id_is_valid '
                           '        CHECK (check_id_in_table_v1(%3$L, belongs_to_id)); '
                           'CREATE UNIQUE INDEX %1$s_single_association ON %1$I (object_id, '
                           '                                        tenancy_workspace_pk, '
                           '                                        visibility_change_set_pk) '
                           '    WHERE visibility_deleted_at IS NULL; '
                           'CREATE INDEX ON %1$I (tenancy_workspace_pk); '
                           'CREATE INDEX ON %1$I (visibility_change_set_pk); '
                           'CREATE INDEX ON %1$I (object_id); '
                           'CREATE INDEX ON %1$I (belongs_to_id); '
                           'CREATE FUNCTION is_visible_v1( '
                           '    check_visibility jsonb, '
                           '    reference %1$I '
                           ') '
                           'RETURNS bool '
                           'LANGUAGE sql '
                           'IMMUTABLE PARALLEL SAFE CALLED ON NULL INPUT '
                           'AS $is_visible_fn$ '
                           '    SELECT is_visible_v1( '
                           '        check_visibility, '
                           '        reference.visibility_change_set_pk, '
                           '        reference.visibility_deleted_at '
                           '    ) '
                           '$is_visible_fn$; '
                           'CREATE FUNCTION in_tenancy_v1( '
                           '    tenancy jsonb, '
                           '    record_to_check %1$I '
                           ') '
                           'RETURNS bool '
                           'LANGUAGE sql '
                           'IMMUTABLE PARALLEL SAFE CALLED ON NULL INPUT '
                           'AS $in_tenancy_fn$ '
                           '    SELECT in_tenancy_v1( '
                           '        tenancy, '
                           '        record_to_check.tenancy_workspace_pk '
                           '    ) '
                           '$in_tenancy_fn$; '
                           'CREATE FUNCTION in_tenancy_and_visible_v1( '
                           '    tenancy jsonb, '
                           '    check_visibility jsonb, '
                           '    record_to_check %1$I '
                           ') '
                           'RETURNS bool '
                           'LANGUAGE sql '
                           'IMMUTABLE PARALLEL SAFE CALLED ON NULL INPUT '
                           'AS $in_tenancy_and_visible_fn$ '
                           '    SELECT '
                           '        in_tenancy_v1( '
                           '            tenancy, '
                           '            record_to_check.tenancy_workspace_pk '
                           '        ) '
                           '        AND is_visible_v1( '
                           '            check_visibility, '
                           '            record_to_check.visibility_change_set_pk, '
                           '            record_to_check.visibility_deleted_at '
                           '        ) '
                           '$in_tenancy_and_visible_fn$; '
                           'CREATE FUNCTION %1$I_v1 ( '
                           '    this_tenancy jsonb, '
                           '    this_visibility jsonb '
                           ') '
                           'RETURNS SETOF %1$I '
                           'LANGUAGE sql '
                           'STABLE PARALLEL SAFE CALLED ON NULL INPUT '
                           'AS $table_view_fn$ '
                           '    SELECT DISTINCT ON (object_id) %1$I.* '
                           '    FROM %1$I '
                           '    WHERE in_tenancy_and_visible_v1(this_tenancy, this_visibility, %1$I) '
                           '    ORDER BY '
                           '        object_id, '
                           '        visibility_change_set_pk DESC, '
                           '        visibility_deleted_at DESC NULLS FIRST '
                           '$table_view_fn$; ',
                           this_table_name,
                           this_object_table,
                           this_belongs_to_table);
    RAISE DEBUG 'create_table query: %', create_table;

    EXECUTE create_table;
END;
$$ LANGUAGE PLPGSQL VOLATILE;

CREATE OR REPLACE FUNCTION attribute_value_dependencies_migration_populate_v1()
RETURNS VOID
AS
$$
DECLARE
    this_attribute_value attribute_values%ROWTYPE;
    this_change_set      change_sets%ROWTYPE;
    this_workspace       workspaces%ROWTYPE;
BEGIN
    FOR this_attribute_value IN
        SELECT av.*
        FROM attribute_values AS av
        WHERE visibility_deleted_at IS NULL
            AND tenancy_workspace_pk = ident_nil_v1()
    LOOP
        PERFORM attribute_value_dependencies_update_v1(
            this_attribute_value.tenancy_workspace_pk,
            this_attribute_value.visibility_change_set_pk,
            this_attribute_value.visibility_deleted_at,
            this_attribute_value.id
        );
    END LOOP;

    -- "HEAD" in all Workspaces
    FOR this_workspace IN
        SELECT *
        FROM workspaces
    LOOP
        FOR this_attribute_value IN
            SELECT av.*
            FROM attribute_values AS av
            WHERE visibility_deleted_at IS NULL
                AND tenancy_workspace_pk = this_workspace.pk
                AND visibility_change_set_pk = ident_nil_v1()
        LOOP
            PERFORM attribute_value_dependencies_update_v1(
                this_attribute_value.tenancy_workspace_pk,
                this_attribute_value.visibility_change_set_pk,
                this_attribute_value.visibility_deleted_at,
                this_attribute_value.id
            );
        END LOOP;
    END LOOP;

    -- All open ChangeSets across all Workspaces
    FOR this_change_set IN
        SELECT *
        FROM change_sets
        WHERE status = 'open'
    LOOP
        FOR this_attribute_value IN
            SELECT av.*
            FROM attribute_values AS av
            WHERE visibility_deleted_at IS NULL
                AND tenancy_workspace_pk = this_change_set.tenancy_workspace_pk
                AND visibility_change_set_pk = this_change_set.pk
        LOOP
            PERFORM attribute_value_dependencies_update_v1(
                this_attribute_value.tenancy_workspace_pk,
                this_attribute_value.visibility_change_set_pk,
                this_attribute_value.visibility_deleted_at,
                this_attribute_value.id
            );
        END LOOP;
    END LOOP;
END;
$$ LANGUAGE PLPGSQL VOLATILE;

SELECT attribute_value_dependencies_migration_populate_v1();
