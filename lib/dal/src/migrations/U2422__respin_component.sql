-- Recreate a component's attribute value tree from scratch for a given
-- schema_variant_id. this does not handle edges, which you'll need to delete as
-- well.
CREATE OR REPLACE FUNCTION component_respin_v1(
    this_tenancy jsonb,
    this_visibility jsonb,
    this_component_id ident,
    this_schema_variant_id ident,
    OUT object JSON) 
AS
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
    this_parent_attribute_value_id          ident;
    this_prop_attribute_value               RECORD;
    this_schema_id                          ident;
    this_unset_func_binding_id              ident;
    this_unset_func_binding_return_value_id ident;
    this_unset_func_id                      ident;
    this_component_row                      components%ROWTYPE;
BEGIN
    this_tenancy_record := tenancy_json_to_columns_v1(this_tenancy);
    this_visibility_record := visibility_json_to_columns_v1(this_visibility);

    -- Delete all attribute values 
    FOR this_attribute_value_id IN
        SELECT id FROM attribute_values_v1(this_tenancy, this_visibility) AS avs
        WHERE attribute_context_component_id = this_component_id
    LOOP
        PERFORM delete_by_id_v1(
            'attribute_values',
            this_tenancy,
            this_visibility,
            this_attribute_value_id
        );
    END LOOP;
    
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

    PERFORM unset_all_belongs_to_v1(
        'component_belongs_to_schema',
        this_tenancy,
        this_visibility,
        this_component_id
    );
    PERFORM set_belongs_to_v1(
      'component_belongs_to_schema',
      this_tenancy,
      this_visibility,
      this_component_id,
      this_schema_id
    );
    PERFORM unset_all_belongs_to_v1(
        'component_belongs_to_schema_variant',
        this_tenancy,
        this_visibility,
        this_component_id
    );
    PERFORM set_belongs_to_v1(
      'component_belongs_to_schema_variant',
      this_tenancy,
      this_visibility,
      this_component_id,
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
            this_component_id -- Component ID
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
            this_component_id -- Component ID
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
            this_component_id -- Component ID
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
            this_component_id -- Component ID
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
            this_component_id
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
        this_component_id -- Component ID
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
                AND av.attribute_context_component_id = this_component_id
                AND in_attribute_context_v1(this_attribute_context, parent_av)
                AND parent_av.attribute_context_component_id = this_component_id
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

    SELECT row_to_json(component.*) 
    INTO object 
    FROM components_v1(this_tenancy, this_visibility) AS component 
    WHERE id=this_component_id;
END;
$$ LANGUAGE PLPGSQL VOLATILE;