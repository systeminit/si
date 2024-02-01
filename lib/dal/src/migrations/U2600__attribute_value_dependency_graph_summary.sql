CREATE TABLE IF NOT EXISTS attribute_value_dependencies (
    pk ident PRIMARY KEY DEFAULT ident_create_v1(),
    id ident NOT NULL DEFAULT ident_create_v1(),
    tenancy_workspace_pk ident,
    visibility_change_set_pk ident NOT NULL DEFAULT ident_nil_v1(),
    visibility_deleted_at TIMESTAMP WITH TIME ZONE,
    created_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT clock_timestamp(),
    updated_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT clock_timestamp(),
    source_attribute_value_id ident NOT NULL,
    destination_attribute_value_id ident NOT NULL
);

SELECT standard_model_table_constraints_v1('attribute_value_dependencies');
INSERT INTO standard_models (table_name, table_type, history_event_label_base, history_event_message_name)
    VALUES ('attribute_value_dependencies', 'model', 'av_deps', 'AV Deps Summary');

CREATE INDEX ON attribute_value_dependencies(source_attribute_value_id);
CREATE INDEX ON attribute_value_dependencies(destination_attribute_value_id);

CREATE OR REPLACE FUNCTION attribute_value_dependencies_update_v1(
    this_tenancy_workspace_pk     ident,
    this_visibility_change_set_pk ident,
    this_visibility_deleted_at    TIMESTAMP WITH TIME ZONE,
    this_destination_av_id        ident
) RETURNS VOID
AS
$$
DECLARE
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

    -- If the AV is deleted, then we want to remove _ALL_ of its references in the graph, both
    -- incoming, and outgoing.
    IF this_visibility_deleted_at IS NOT NULL THEN
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

    SELECT *
    INTO STRICT this_destination_av
    FROM attribute_values_v1(this_tenancy, this_visibility)
    WHERE id = this_destination_av_id;

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
                FROM attribute_value_belongs_to_attribute_prototype_v1(this_tenancy, this_visibility) AS avbtap
                    INNER JOIN attribute_prototype_arguments_v1(this_tenancy, this_visibility) AS apa
                        ON apa.attribute_prototype_id = avbtap.belongs_to_id
                            AND apa.head_component_id = this_destination_av.attribute_context_component_id
                    INNER JOIN attribute_values_v1(this_tenancy, this_visibility) AS source_av
                        ON source_av.attribute_context_external_provider_id = apa.external_provider_id
                            AND source_av.attribute_context_component_id = apa.tail_component_id
                    -- Make sure we're not considering deleted Components as available sources.
                    INNER JOIN components_v1(this_tenancy, this_visibility) AS components
                        ON components.id = source_av.attribute_context_component_id
                WHERE avbtap.object_id = this_destination_av_id
            ) AS source_avs;
        ELSE
            SELECT array_agg(source_avs.id)
            INTO this_source_av_ids
            FROM attribute_value_dependencies_ip_av_sources_v1(
                this_tenancy,
                this_visibility,
                this_internal_provider.prop_id,
                this_destination_av.attribute_context_component_id
            ) AS source_avs(id);
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
                        AND source_av.attribute_context_component_id = this_destination_av.attribute_context_component_id
                -- Make sure we're not considering deleted Components as available sources.
                INNER JOIN components_v1(this_tenancy, this_visibility) AS components
                    ON components.id = source_av.attribute_context_component_id
            WHERE avbtap.object_id = this_destination_av_id
        ) AS source_avs;
    END IF;

    -- Remove any links that are no longer relevant
    FOR this_summary_row_id IN
        SELECT id
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
        SELECT DISTINCT current_sources.id
        FROM unnest(this_source_av_ids) AS current_sources(id)
            LEFT JOIN attribute_value_dependencies_v1(this_tenancy, this_visibility) AS avd
                ON avd.source_attribute_value_id = current_sources.id
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
$$ LANGUAGE PLPGSQL;

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
$$ LANGUAGE SQL;

-- For use as a trigger on attribute_values
CREATE OR REPLACE FUNCTION attribute_value_dependencies_av_trigger_v1()
RETURNS trigger AS
$$
BEGIN
    PERFORM attribute_value_dependencies_update_v1(
        NEW.tenancy_workspace_pk,
        NEW.visibility_change_set_pk,
        NEW.visibility_deleted_at,
        NEW.id
    );

    -- AFTER INSERT OR UPDATE triggers don't actually care about the content of the row returned,
    -- only that there is one returned.
    RETURN NEW;
END
$$ LANGUAGE PLPGSQL;

-- For use as a trigger on attribute_value_belongs_to_attribute_value.
--
-- Whenever an AttributeValue gets a new child, that child needs to be added as one of the
-- "sources" of the InternalProvider of the closest ancestor that has an InternalProvider.
-- If the closest ancestor InternalProvider is our own, then there's nothing to do, as that's
-- handled by other triggers.
--
-- The only AttributeValues that have an attribute_value_belongs_to_attribute_value relation
-- are ones for Props.
CREATE OR REPLACE FUNCTION attribute_value_dependencies_avbtav_trigger_v1()
RETURNS trigger AS
$$
DECLARE
    this_child_av                attribute_values%ROWTYPE;
    this_current_ancestor_av     attribute_values%ROWTYPE;
    this_destination_ip_av_id    ident;
    this_internal_provider       internal_providers%ROWTYPE;
    this_internal_provider_av_id ident;
    this_tenancy                 jsonb;
    this_visibility              jsonb;
BEGIN
    this_tenancy := jsonb_build_object('tenancy_workspace_pk', NEW.tenancy_workspace_pk);
    this_visibility := jsonb_build_object(
        'visibility_change_set_pk', NEW.visibility_change_set_pk,
        'visibility_deleted_at', NEW.visibility_deleted_at
    );

    SELECT av.*
    INTO STRICT this_child_av
    FROM attribute_values_v1(this_tenancy, this_visibility) AS av
    WHERE av.id = NEW.object_id;

    SELECT ip.*
    INTO this_internal_provider
    FROM internal_providers_v1(this_tenancy, this_visibility) AS ip
    WHERE ip.prop_id = this_child_av.attribute_context_prop_id;
    IF this_internal_provider.id IS NOT NULL THEN
        -- If the child has an InternalProvider itself, then there's nothing to do here,
        -- as the InternalProvider's AttributeValue will include this in its source
        -- calculations.
        RETURN NEW;
    END IF;

    SELECT av.*
    INTO STRICT this_current_ancestor_av
    FROM attribute_values_v1(this_tenancy, this_visibility) AS av
    WHERE av.id = NEW.belongs_to_id;

    LOOP
        SELECT ip.*
        INTO this_internal_provider
        FROM internal_providers_v1(this_tenancy, this_visibility) AS ip
        WHERE ip.prop_id = this_current_ancestor_av.attribute_context_prop_id;
        IF this_internal_provider.id IS NOT NULL THEN
            SELECT av.id
            INTO STRICT this_internal_provider_av_id
            FROM attribute_values_v1(this_Tenancy, this_visibility) AS AV
            WHERE av.attribute_context_internal_provider_id = this_internal_provider.id
                AND av.attribute_context_component_id = this_current_ancestor_av.attribute_context_component_id;

            PERFORM attribute_value_dependencies_update_v1(
                NEW.tenancy_workspace_pk,
                NEW.visibility_change_set_pk,
                NEW.visibility_deleted_at,
                this_internal_provider_av_id
            );
            -- RETURN NEW;
        END IF;

        this_child_av := this_current_ancestor_av;
        SELECT av.*
        INTO this_current_ancestor_av
        FROM attribute_values_v1(this_tenancy, this_visibility) AS av
            INNER JOIN attribute_value_belongs_to_attribute_value_v1(this_tenancy, this_visibility) AS avbtav
                ON avbtav.belongs_to_id = av.id
        WHERE avbtav.object_id = this_child_av.id;
        IF this_current_ancestor_av.id IS NULL THEN
            -- We've run out of parent attribute values to look at when trying to find
            -- one that has an associated InternalProvider.
            RETURN NEW;
        END IF;
    END LOOP;
END
$$ LANGUAGE PLPGSQL;

-- For use as a trigger on attribute_prototypes
CREATE OR REPLACE FUNCTION attribute_value_dependencies_ap_trigger_v1()
RETURNS trigger AS
$$
DECLARE
    this_destination_av_id ident;
    this_tenancy           jsonb;
    this_visibility        jsonb;
BEGIN
    this_tenancy := jsonb_build_object('tenancy_workspace_pk', NEW.tenancy_workspace_pk);
    this_visibility := jsonb_build_object(
        'visibility_change_set_pk', NEW.visibility_change_set_pk,
        'visibility_deleted_at', NEW.visibility_deleted_at
    );

    FOR this_destination_av_id IN
        SELECT destination_av.id
        FROM attribute_value_belongs_to_attribute_prototype_v1(this_tenancy, this_visibility) AS avbtap
            INNER JOIN attribute_values_v1(this_tenancy, this_visibility) AS destination_av
                ON destination_av.id = avbtap.object_id
            -- Make sure we're not doing things for a deleted component.
            INNER JOIN components_v1(this_tenancy, this_visibility) AS components
                ON destination_av.attribute_context_component_id = components.id
        WHERE avbtap.belongs_to_id = NEW.id
    LOOP
        PERFORM attribute_value_dependencies_update_v1(
            NEW.tenancy_workspace_pk,
            NEW.visibility_change_set_pk,
            NEW.visibility_deleted_at,
            this_destination_av_id
        );
    END LOOP;

    -- AFTER INSERT OR UPDATE triggers don't actually care about the content of the row returned,
    -- only that there is one returned.
    RETURN NEW;
END
$$ LANGUAGE PLPGSQL;

-- For use as a trigger on attribute_prototype_arguments
CREATE OR REPLACE FUNCTION attribute_value_dependencies_apa_trigger_v1()
RETURNS trigger AS
$$
DECLARE
    this_destination_av_id ident;
    this_tenancy           jsonb;
    this_visibility        jsonb;
BEGIN
    this_tenancy := jsonb_build_object('tenancy_workspace_pk', NEW.tenancy_workspace_pk);
    this_visibility := jsonb_build_object(
        'visibility_change_set_pk', NEW.visibility_change_set_pk,
        'visibility_deleted_at', NEW.visibility_deleted_at
    );

    FOR this_destination_av_id IN
        -- An AttributePrototypeArgument linking two Components
        SELECT destination_av.id
        FROM attribute_prototype_arguments_v1(this_tenancy, this_visibility) AS apa
            INNER JOIN attribute_value_belongs_to_attribute_prototype_v1(this_tenancy, this_visibility) AS avbtap
                ON avbtap.belongs_to_id = apa.attribute_prototype_id
            INNER JOIN attribute_values_v1(this_tenancy, this_visibility) AS destination_av
                ON destination_av.id = avbtap.object_id
                    AND destination_av.attribute_context_component_id = apa.head_component_id
            -- Make sure we're not doing things for a deleted component
            INNER JOIN components_v1(this_tenancy, this_visibility) AS components
                ON components.id = destination_av.attribute_context_component_id
        WHERE apa.id = NEW.id
            AND apa.external_provider_id != ident_nil_v1()
        UNION
        -- An AttributePrototypeArgument internal to a Component
        SELECT destination_av.id
        FROM attribute_prototype_arguments_v1(this_tenancy, this_visibility) AS apa
            INNER JOIN attribute_value_belongs_to_attribute_prototype_v1(this_tenancy, this_visibility) AS avbtap
                ON avbtap.belongs_to_id = apa.attribute_prototype_id
            INNER JOIN attribute_values_v1(this_tenancy, this_visibility) AS destination_av
                ON destination_av.id = avbtap.object_id
            -- Make sure we're not doing things for a deleted component
            INNER JOIN components_v1(this_tenancy, this_visibility) AS components
                ON components.id = destination_av.attribute_context_component_id
        WHERE apa.id = NEW.id
            AND apa.external_provider_id = ident_nil_v1()
    LOOP
        PERFORM attribute_value_dependencies_update_v1(
            NEW.tenancy_workspace_pk,
            NEW.visibility_change_set_pk,
            NEW.visibility_deleted_at,
            this_destination_av_id
        );
    END LOOP;

    -- AFTER INSERT OR UPDATE triggers don't actually care about the content of the row returned,
    -- only that there is one returned.
    RETURN NEW;
END
$$ LANGUAGE PLPGSQL;

CREATE OR REPLACE FUNCTION attribute_value_dependencies_avbtap_trigger_v1()
RETURNS trigger AS
$$
BEGIN
    PERFORM attribute_value_dependencies_update_v1(
        NEW.tenancy_workspace_pk,
        NEW.visibility_change_set_pk,
        NEW.visibility_deleted_at,
        NEW.object_id
    );

    RETURN NEW;
END
$$ LANGUAGE PLPGSQL;

-- CREATE OR REPLACE TRIGGER av_update_attribute_value_dependencies
--     AFTER INSERT OR UPDATE OF func_binding_id
--     ON attribute_values
--     FOR EACH ROW
--     EXECUTE FUNCTION attribute_value_dependencies_av_trigger_v1();

CREATE OR REPLACE TRIGGER avbtav_update_attribute_value_dependencies
    AFTER INSERT OR UPDATE OF
        object_id,
        belongs_to_id,
        visibility_deleted_at
    ON attribute_value_belongs_to_attribute_value
    FOR EACH ROW
    EXECUTE FUNCTION attribute_value_dependencies_avbtav_trigger_v1();

CREATE OR REPLACE TRIGGER ap_update_attribute_value_dependencies
    AFTER INSERT OR UPDATE OF
        func_id,
        visibility_deleted_at
    ON attribute_prototypes
    FOR EACH ROW
    EXECUTE FUNCTION attribute_value_dependencies_ap_trigger_v1();

CREATE OR REPLACE TRIGGER apa_update_attribute_value_dependencies
    AFTER INSERT OR UPDATE OF
        internal_provider_id,
        external_provider_id,
        tail_component_id,
        head_component_id,
        visibility_deleted_at
    ON attribute_prototype_arguments
    FOR EACH ROW
    EXECUTE FUNCTION attribute_value_dependencies_apa_trigger_v1();

CREATE OR REPLACE TRIGGER avbtap_update_attribute_value_dependencies
    AFTER INSERT OR UPDATE OF
        object_id,
        belongs_to_id,
        visibility_deleted_at
    ON attribute_value_belongs_to_attribute_prototype
    FOR EACH ROW
    EXECUTE FUNCTION attribute_value_dependencies_avbtap_trigger_v1();

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
    this_new_attribute_value_ids            ident[];
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

        this_new_attribute_value_ids := array_append(this_new_attribute_value_ids, this_new_attribute_value ->> 'id');
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

        this_new_attribute_value_ids := array_append(this_new_attribute_value_ids, this_new_attribute_value ->> 'id');
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

        this_new_attribute_value_ids := array_append(this_new_attribute_value_ids, this_new_attribute_value ->> 'id');

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

        this_new_attribute_value_ids := array_append(this_new_attribute_value_ids, this_new_attribute_value ->> 'id');
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

        this_new_attribute_value_ids := array_append(this_new_attribute_value_ids, this_new_attribute_value ->> 'id');
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

    -- Because all of the AttributeValues don't come into existance at the same time, not all of the
    -- dependency links will be set up correctly by the table trigger for AttributeValue insertion.
    -- By making sure that everything has its inputs calculated at the time that they all exist,
    -- then we can be sure that the graph will be complete.
    FOREACH this_new_attribute_value_id IN ARRAY this_new_attribute_value_ids LOOP
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
