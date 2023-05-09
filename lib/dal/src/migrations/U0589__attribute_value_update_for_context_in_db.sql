CREATE EXTENSION IF NOT EXISTS "uuid-ossp";

CREATE OR REPLACE FUNCTION less_specific_attribute_context_v1(check_context                  jsonb,
                                                              reference_prop_id              ident,
                                                              reference_internal_provider_id ident,
                                                              reference_external_provider_id ident,
                                                              reference_schema_id            ident,
                                                              reference_schema_variant_id    ident,
                                                              reference_component_id         ident,
                                                              OUT result bool
)
AS
$$
DECLARE
    check_context_record             attribute_context_record_v1;
    check_is_component_specific      bool;
    check_is_schema_variant_specific bool;
    check_is_schema_specific         bool;
BEGIN
    check_context_record := attribute_context_json_to_columns_v1(check_context);

    check_is_component_specific := FALSE;
    check_is_schema_variant_specific := FALSE;
    check_is_schema_specific := FALSE;

    IF check_context_record.attribute_context_component_id != ident_nil_v1() THEN
        check_is_component_specific := TRUE;
    ELSE
        -- If the check_context is only specific to one of the triple that is
        -- the least level of specificity, then there is no possible way for
        -- the reference AttributeContext to be less specific than that.
        result := FALSE;
        RETURN;
    END IF;

    -- Check Component level.
    IF check_is_component_specific
       AND reference_component_id != ident_nil_v1() THEN
        -- If the ComponentId is the most specific part of the "check"
        -- AttributeContext, then "reference" needs to have ident_nil_v1() to be
        -- less specific.
        result := FALSE;
        RETURN;
    ELSIF NOT check_is_component_specific
          AND check_context_record.attribute_context_component_id != reference_component_id THEN
        -- If the Component isn't the most specific field, then the
        -- ComponentId must be the same between "check" and "reference":
        --   * If System is the most specific piece, then the ComponentId
        --     must be for the same Component on both.
        --   * If something less specific than Component is the most
        --     specific peice, then both "check" and "reference" must have
        --     ident_nil_v1().
        result := FALSE;
        RETURN;
    END IF;
    -- The only options left should all mean that the ComponentId on
    -- "reference" is acceptable:
    --   * Component is the most specific, and "reference" is set to ident_nil_v1().
    --   * Component is not the most specific, and "check" is the same
    --     "reference".

    -- Check the least specific "triple"
    IF check_context_record.attribute_context_external_provider_id != reference_external_provider_id
       OR check_context_record.attribute_context_internal_provider_id != reference_internal_provider_id
       OR check_context_record.attribute_context_prop_id != reference_prop_id THEN
        -- The External/Internal/Prop _must all_ be the same on both "check" and
        -- "reference" since we already asserted that something _more_ specific than
        -- External/Internal/Prop is the level of specificity on "check".
        result := FALSE;
        RETURN;
    END IF;

    -- We know all the checks passed, because of early returns. So, if we got here,
    -- then everything looks good.
    result := TRUE;
END;
-- This is safe to be IMMUTABLE since for any given (check_context, reference) pair, this will
-- always return the same result.
$$ LANGUAGE PLPGSQL IMMUTABLE PARALLEL SAFE;

CREATE OR REPLACE FUNCTION attribute_context_from_record_v1(source_av attribute_values)
RETURNS jsonb
LANGUAGE sql
IMMUTABLE
PARALLEL SAFE
AS $$
    SELECT jsonb_build_object(
        'attribute_context_prop_id',              source_av.attribute_context_prop_id,
        'attribute_context_internal_provider_id', source_av.attribute_context_internal_provider_id,
        'attribute_context_external_provider_id', source_av.attribute_context_external_provider_id,
        'attribute_context_component_id',         source_av.attribute_context_component_id
    )
$$;

CREATE OR REPLACE FUNCTION attribute_context_from_record_v1(source_ap attribute_prototypes)
RETURNS jsonb
LANGUAGE sql
IMMUTABLE
PARALLEL SAFE
AS $$
    SELECT jsonb_build_object(
        'attribute_context_prop_id',              source_ap.attribute_context_prop_id,
        'attribute_context_internal_provider_id', source_ap.attribute_context_internal_provider_id,
        'attribute_context_external_provider_id', source_ap.attribute_context_external_provider_id,
        'attribute_context_component_id',         source_ap.attribute_context_component_id
    )
$$;

CREATE OR REPLACE FUNCTION attribute_context_from_jsonb_v1(source_jsonb jsonb)
RETURNS jsonb
LANGUAGE sql
IMMUTABLE
PARALLEL SAFE
AS $$
    SELECT jsonb_build_object(
        'attribute_context_prop_id',              source_jsonb -> 'attribute_context_prop_id',
        'attribute_context_internal_provider_id', source_jsonb -> 'attribute_context_internal_provider_id',
        'attribute_context_external_provider_id', source_jsonb -> 'attribute_context_external_provider_id',
        'attribute_context_component_id',         source_jsonb -> 'attribute_context_component_id'
    )
$$;

CREATE OR REPLACE FUNCTION less_specific_attribute_context_v1(check_context jsonb,
                                                              reference     record,
                                                              OUT result    bool
)
AS
$$
BEGIN
    result := less_specific_attribute_context_v1(check_context,
                                                 reference.attribute_context_prop_id,
                                                 reference.attribute_context_internal_provider_id,
                                                 reference.attribute_context_external_provider_id,
                                                 reference.attribute_context_component_id);
END;
-- This is safe to be IMMUTABLE since for any given (check_context, reference) pair, this will always return
-- the same result. Technically, we only care whether the attribute_context_* fields have changed, and not the
-- entire reference record, but if the record as a whole hasn't changed, then that means that the
-- attribute_context_* fields haven't either.
$$ LANGUAGE PLPGSQL IMMUTABLE PARALLEL SAFE;

CREATE OR REPLACE FUNCTION attribute_context_is_least_specific_v1(this_context jsonb)
RETURNS bool
LANGUAGE sql
IMMUTABLE
PARALLEL SAFE
AS $$
    SELECT  (this_context ->> 'attribute_context_component_id')::ident = ident_nil_v1()
$$;

CREATE OR REPLACE FUNCTION attribute_value_set_parent_attribute_value_v1(this_tenancy                   jsonb,
                                                                         this_visibility                jsonb,
                                                                         this_attribute_value_id        ident,
                                                                         this_parent_attribute_value_id ident) RETURNS void
AS
$$
DECLARE
    attribute_value_key                  text;
    attribute_value_context              jsonb;
    potential_duplicate_child_for_parent jsonb;
BEGIN
    SELECT DISTINCT ON (id) key, attribute_context_from_record_v1(attribute_values)
    INTO attribute_value_key, attribute_value_context
    FROM attribute_values
    WHERE id = this_attribute_value_id
          AND in_tenancy_and_visible_v1(this_tenancy, this_visibility, attribute_values)
    ORDER BY id,
             visibility_change_set_pk DESC,
             visibility_deleted_at DESC NULLS FIRST;

    potential_duplicate_child_for_parent := attribute_value_find_with_parent_and_key_for_context_v1(this_tenancy,
                                                                                                    this_visibility,
                                                                                                    this_parent_attribute_value_id,
                                                                                                    attribute_value_key,
                                                                                                    attribute_value_context);

    IF attribute_contexts_match_v1(attribute_value_context, potential_duplicate_child_for_parent) THEN
        RAISE 'Found duplicate child (AttributeValue(%)) when trying to set parent (AttributeValue(%)) for '
              'AttributeValue(%), Tenancy(%), Visibility(%)',
              (potential_duplicate_child_for_parent ->> 'id')::ident,
              this_parent_attribute_value_id,
              this_attribute_value_id,
              this_tenancy,
              this_visibility;
    END IF;

    PERFORM set_belongs_to_v1(
        'attribute_value_belongs_to_attribute_value',
        this_tenancy,
        this_visibility,
        this_attribute_value_id,
        this_parent_attribute_value_id
    );
END;
$$ LANGUAGE PLPGSQL;

DROP FUNCTION IF EXISTS attribute_value_find_with_parent_and_key_for_context_v1;
CREATE OR REPLACE FUNCTION attribute_value_find_with_parent_and_key_for_context_v1(this_tenancy                   jsonb,
                                                                                   this_visibility                jsonb,
                                                                                   this_parent_attribute_value_id ident,
                                                                                   this_key                       text,
                                                                                   this_attribute_context         jsonb,
                                                                                   OUT attribute_value            jsonb
)
AS
$$
BEGIN
    SELECT DISTINCT ON (attribute_context_prop_id)
        to_jsonb(av.*)
    INTO attribute_value
    FROM attribute_values_v1(this_tenancy, this_visibility) AS av
    LEFT JOIN (
        SELECT DISTINCT ON (object_id)
            object_id AS child_attribute_value_id,
            belongs_to_id AS parent_attribute_value_id
        FROM attribute_value_belongs_to_attribute_value
        WHERE in_tenancy_and_visible_v1(this_tenancy, this_visibility, attribute_value_belongs_to_attribute_value)
        ORDER BY object_id,
                 visibility_change_set_pk DESC,
                 visibility_deleted_at DESC
    ) AS avbtav ON avbtav.child_attribute_value_id = av.id
    LEFT JOIN (
        SELECT DISTINCT ON (id)
            id as comp_id,
            needs_destroy,
            visibility_deleted_at
        FROM components
        WHERE in_tenancy_and_visible_v1(this_tenancy, this_visibility || jsonb_build_object('visibility_deleted_at', NULL), components)
        ORDER BY id,
            visibility_change_set_pk DESC,
            visibility_deleted_at DESC
    ) as components ON components.comp_id = av.attribute_context_component_id
    WHERE in_attribute_context_v1(this_attribute_context, av)
          AND CASE
                  WHEN this_parent_attribute_value_id IS NULL THEN avbtav.parent_attribute_value_id IS NULL
                  ELSE avbtav.parent_attribute_value_id = this_parent_attribute_value_id
              END
          AND CASE
                  WHEN this_key IS NULL THEN av.key IS NULL
                  ELSE av.key = this_key
              END
          AND CASE
                WHEN components.comp_id != ident_nil_v1() THEN
                    (components.visibility_deleted_at IS NOT NULL AND components.needs_destroy) OR components.visibility_deleted_at IS NULL
                ELSE true
              END
    ORDER BY attribute_context_prop_id,
             visibility_change_set_pk DESC,
             av.visibility_deleted_at DESC NULLS FIRST,
             attribute_context_internal_provider_id DESC,
             attribute_context_external_provider_id DESC,
             attribute_context_component_id DESC;
END;
$$ LANGUAGE PLPGSQL PARALLEL SAFE;

CREATE OR REPLACE FUNCTION attribute_context_less_specific_v1(this_attribute_context    jsonb,
                                                              OUT new_attribute_context jsonb
)
AS
$$
DECLARE
    new_attribute_context_record attribute_context_record_v1;
BEGIN
    new_attribute_context_record := jsonb_populate_record(null::attribute_context_record_v1, this_attribute_context);

    IF new_attribute_context_record.attribute_context_component_id != ident_nil_v1() THEN
        -- Remove the ComponentId part of the AttributeContext.
        new_attribute_context_record.attribute_context_component_id := ident_nil_v1();
    END IF;
    -- We don't try to remove the PropId/InternalProviderId/ExternalProviderId as there is nothing less
    -- specific than that part of the AttributeContext. (Those three form a triple that is the least specific
    -- portion.)

    new_attribute_context := to_jsonb(new_attribute_context_record);
END;
-- The output of this is stable based purely on the input, so it is safe to be IMMUTABLE.
$$ LANGUAGE PLPGSQL IMMUTABLE PARALLEL SAFE;

DROP FUNCTION IF EXISTS ap_find_with_parent_value_and_key_for_context_v1;
CREATE OR REPLACE FUNCTION ap_find_with_parent_value_and_key_for_context_v1(
    this_tenancy                   jsonb,
    this_visibility                jsonb,
    this_parent_attribute_value_id ident,
    this_key                       text,
    this_attribute_context         jsonb
)
RETURNS json
LANGUAGE sql
STABLE
PARALLEL SAFE
AS $$
    SELECT DISTINCT ON (
        ap.attribute_context_prop_id,
        COALESCE(ap.key, '')
    )
        to_jsonb(ap.*)
    FROM attribute_prototypes_v1(this_tenancy, this_visibility) AS ap
    INNER JOIN attribute_value_belongs_to_attribute_prototype_v1(this_tenancy, this_visibility) AS avbtap
        ON ap.id = avbtap.belongs_to_id
    INNER JOIN attribute_values_v1(this_tenancy, this_visibility) AS av
        ON avbtap.object_id = av.id
            AND in_attribute_context_v1(this_attribute_context, av)
    LEFT JOIN attribute_value_belongs_to_attribute_value_v1(this_tenancy, this_visibility) AS avbtav
        ON av.id = avbtav.object_id
    LEFT JOIN attribute_values_v1(this_tenancy, this_visibility) AS parent_av
        ON avbtav.belongs_to_id = parent_av.id
            AND in_attribute_context_v1(this_attribute_context, av)
    LEFT JOIN (
        SELECT DISTINCT ON (id)
            id as comp_id,
            needs_destroy,
            visibility_deleted_at
        FROM components
        WHERE in_tenancy_and_visible_v1(this_tenancy, this_visibility || jsonb_build_object('visibility_deleted_at', NULL), components)
        ORDER BY id,
            visibility_change_set_pk DESC,
            visibility_deleted_at DESC
    ) as components ON components.comp_id = ap.attribute_context_component_id
    WHERE
        exact_attribute_context_v1(this_attribute_context, ap)
        AND CASE
                WHEN this_parent_attribute_value_id IS NULL THEN parent_av.id IS NULL
                ELSE parent_av.id = this_parent_attribute_value_id
            END
        AND CASE
                WHEN this_key IS NULL THEN ap.key IS NULL
                ELSE ap.key = this_key
            END
        AND CASE
            WHEN components.comp_id != ident_nil_v1() THEN
                (components.visibility_deleted_at IS NOT NULL AND components.needs_destroy) OR components.visibility_deleted_at IS NULL
            ELSE true
        END
    ORDER BY
        ap.attribute_context_prop_id DESC,
        COALESCE(ap.key, ''),
        ap.attribute_context_internal_provider_id DESC,
        ap.attribute_context_external_provider_id DESC,
        ap.attribute_context_component_id DESC
$$;

CREATE OR REPLACE FUNCTION attribute_prototype_create_intermediate_proxy_values_v1(this_tenancy                   jsonb,
                                                                                   this_visibility                jsonb,
                                                                                   this_parent_attribute_value_id ident,
                                                                                   this_prototype_id              ident,
                                                                                   this_attribute_context         jsonb
) RETURNS void
AS
$$
DECLARE
    attribute_value       attribute_values%ROWTYPE;
    proxy_attribute_value attribute_values%ROWTYPE;
    proxy_target          attribute_values%ROWTYPE;
BEGIN
    IF attribute_context_is_least_specific_v1(this_attribute_context) THEN
        RETURN;
    END IF;

    attribute_value := jsonb_populate_record(null::attribute_values,
                                             attribute_value_find_with_parent_and_prototype_for_context_v1(this_tenancy,
                                                                                                           this_visibility,
                                                                                                           this_parent_attribute_value_id,
                                                                                                           this_prototype_id,
                                                                                                           this_attribute_context));
    IF attribute_value.id IS NULL THEN
        PERFORM attribute_prototype_create_intermediate_proxy_values_v1(this_tenancy,
                                                                        this_visibility,
                                                                        this_parent_attribute_value_id,
                                                                        this_prototype_id,
                                                                        attribute_context_less_specific_v1(this_attribute_context));
        proxy_target := jsonb_populate_record(null::attribute_values,
                                              attribute_value_find_with_parent_and_prototype_for_context_v1(this_tenancy,
                                                                                                            this_visibility,
                                                                                                            this_parent_attribute_value_id,
                                                                                                            this_prototype_id,
                                                                                                            attribute_context_less_specific_v1(this_attribute_context)));
        IF proxy_target.id IS NOT NULL THEN
            proxy_attribute_value = json_populate_record(null::attribute_values,
                                                         attribute_value_new_v1(this_tenancy,
                                                                                this_visibility,
                                                                                proxy_target.func_binding_id,
                                                                                proxy_target.func_binding_return_value_id,
                                                                                this_attribute_context,
                                                                                proxy_target.key));
            PERFORM update_by_id_v1('attribute_values',
                                    'proxy_for_attribute_value_id',
                                    this_tenancy,
                                    this_visibility,
                                    proxy_attribute_value.id,
                                    proxy_target.id);
            PERFORM set_belongs_to_v1(
                'attribute_value_belongs_to_attribute_prototype',
                this_tenancy,
                this_visibility,
                proxy_attribute_value.id,
                this_prototype_id
            );
        ELSE
            RAISE 'AttributeValue not found for AttributePrototype(%), parent AttributeValue(%) in Tenancy(%), Visibility(%)',
                  this_prototype_id,
                  this_parent_attribute_value_id,
                  this_tenancy,
                  this_visibility;
        END IF;
    END IF;
END;
$$ LANGUAGE PLPGSQL;

CREATE OR REPLACE FUNCTION attribute_prototype_new_v1(this_tenancy                      jsonb,
                                                      this_visibility                   jsonb,
                                                      this_func_id                      ident,
                                                      this_func_binding_id              ident,
                                                      this_func_binding_return_value_id ident,
                                                      this_attribute_context            jsonb,
                                                      this_key                          text,
                                                      this_parent_attribute_value_id    ident,
                                                      OUT new_attribute_prototype       json
)
AS
$$
DECLARE
    attribute_value_id                     ident;
    existing_attribute_value_proxy_context jsonb;
    new_attribute_prototype_id             ident;
    original_attribute_prototype_id        ident;
BEGIN
    SELECT (ap.object ->> 'id')::ident
    INTO new_attribute_prototype_id
    FROM attribute_prototype_create_v1(this_tenancy,
                                       this_visibility,
                                       this_attribute_context,
                                       this_func_id,
                                       this_key) AS ap;

    SELECT DISTINCT ON (id) row_to_json(attribute_prototypes)
    INTO new_attribute_prototype
    FROM attribute_prototypes
    WHERE in_tenancy_and_visible_v1(this_tenancy, this_visibility, attribute_prototypes)
          AND id = new_attribute_prototype_id
    ORDER BY id,
             visibility_change_set_pk DESC,
             visibility_deleted_at DESC NULLS FIRST;

    SELECT (av.object ->> 'id')::ident
    INTO attribute_value_id
    FROM attribute_value_create_v1(this_tenancy,
                                   this_visibility,
                                   this_attribute_context,
                                   this_func_binding_id,
                                   this_func_binding_return_value_id,
                                   this_key) AS av;
    PERFORM set_belongs_to_v1(
        'attribute_value_belongs_to_attribute_prototype',
        this_tenancy,
        this_visibility,
        attribute_value_id,
        new_attribute_prototype_id
    );

    IF this_parent_attribute_value_id IS NOT NULL THEN
        PERFORM set_belongs_to_v1(
            'attribute_value_belongs_to_attribute_value',
            this_tenancy,
            this_visibility,
            attribute_value_id,
            this_parent_attribute_value_id
        );
    END IF;

    IF NOT attribute_context_is_least_specific_v1(this_attribute_context) THEN
        SELECT attribute_context_from_jsonb_v1(av.attribute_value)
        INTO existing_attribute_value_proxy_context
        FROM attribute_value_find_with_parent_and_key_for_context_v1(this_tenancy,
                                                                     this_visibility,
                                                                     this_parent_attribute_value_id,
                                                                     this_key,
                                                                     this_attribute_context) AS av;

        IF NOT (existing_attribute_value_proxy_context IS NOT NULL
                AND exact_attribute_context_v1(this_attribute_context, existing_attribute_value_proxy_context))
        THEN
            SELECT (ap.found_attribute_prototype ->> 'id')::ident
            INTO original_attribute_prototype_id
            FROM ap_find_with_parent_value_and_key_for_context_v1(this_tenancy,
                                                                  this_visibility,
                                                                  this_parent_attribute_value_id,
                                                                  this_key,
                                                                  attribute_context_less_specific_v1(this_attribute_context)) AS ap;
            IF original_attribute_prototype_id IS NOT NULL THEN
                PERFORM attribute_prototype_create_intermediate_proxy_values_v1(this_tenancy,
                                                                                this_visibility,
                                                                                this_parent_attribute_value_id,
                                                                                original_attribute_prototype_id,
                                                                                attribute_context_less_specific_v1(this_attribute_context));
            END IF;
        END IF;
    END IF;
END;
$$ LANGUAGE PLPGSQL;

-- AttributePrototype::new_with_existing_value
CREATE OR REPLACE FUNCTION attribute_prototype_new_with_attribute_value_v1(this_tenancy                   jsonb,
                                                                           this_visibility                jsonb,
                                                                           this_func_id                   ident,
                                                                           this_attribute_context         jsonb,
                                                                           this_key                       text,
                                                                           this_parent_attribute_value_id ident,
                                                                           this_attribute_value_id        ident,
                                                                           OUT new_attribute_prototype_id ident
)
AS
$$
BEGIN
    SELECT (ap.object ->> 'id')::ident
    INTO new_attribute_prototype_id
    FROM attribute_prototype_create_v1(this_tenancy,
                                       this_visibility,
                                       this_attribute_context,
                                       this_func_id,
                                       this_key) AS ap;

    PERFORM set_belongs_to_v1(
        'attribute_value_belongs_to_attribute_prototype',
        this_tenancy,
        this_visibility,
        this_attribute_value_id,
        new_attribute_prototype_id
    );

    IF this_parent_attribute_value_id IS NOT NULL THEN
        PERFORM set_belongs_to_v1(
            'attribute_value_belongs_to_attribute_value',
            this_tenancy,
            this_visibility,
            this_attribute_value_id,
            this_parent_attribute_value_id
        );
    ELSE
        PERFORM unset_belongs_to_v1(
            'attribute_value_belongs_to_attribute_value',
            this_tenancy,
            this_visibility,
            this_attribute_value_id
        );
    END IF;
END;
$$ LANGUAGE PLPGSQL;

CREATE OR REPLACE FUNCTION attribute_context_least_specific_is_provider_v1(
  this_attribute_context jsonb
)
RETURNS bool
LANGUAGE sql
IMMUTABLE
PARALLEL SAFE
AS $$
    SELECT (this_attribute_context ->> 'attribute_context_internal_provider_id')::ident != ident_nil_v1()
        OR (this_attribute_context ->> 'attribtue_context_external_provider_id')::ident != ident_nil_v1()
$$;

CREATE OR REPLACE FUNCTION exact_attribute_context_v1(
    check_context     jsonb,
    reference_context jsonb
)
RETURNS bool
LANGUAGE sql
IMMUTABLE
PARALLEL SAFE
AS
$$
    SELECT exact_attribute_context_v1(
        check_context,
        (reference_context ->> 'attribute_context_prop_id')::ident,
        (reference_context ->> 'attribute_context_internal_provider_id')::ident,
        (reference_context ->> 'attribute_context_external_provider_id')::ident,
        (reference_context ->> 'attribute_context_component_id')::ident
    )
$$;

-- Changing the return type from the previous version requires dropping & recreating.
DROP FUNCTION IF EXISTS attribute_value_find_with_key_in_context_v1;
CREATE OR REPLACE FUNCTION attribute_value_find_with_key_in_context_v1(
    this_tenancy           jsonb,
    this_visibility        jsonb,
    this_key               text,
    this_attribute_context jsonb)
RETURNS SETOF attribute_values
LANGUAGE sql
STABLE
PARALLEL SAFE
AS $$
    SELECT DISTINCT ON (
        COALESCE(belongs_to_id, ident_nil_v1()),
        attribute_context_prop_id,
        attribute_context_internal_provider_id,
        attribute_context_external_provider_id,
        COALESCE(key, '')
    )
        av.*
    FROM attribute_values_v1(this_tenancy, this_visibility) AS av
    LEFT JOIN attribute_value_belongs_to_attribute_value_v1(this_tenancy, this_visibility) AS avbtav
        ON avbtav.object_id = av.id
    LEFT JOIN (
        SELECT DISTINCT ON (id)
            id as comp_id,
            needs_destroy,
            visibility_deleted_at
        FROM components
        WHERE in_tenancy_and_visible_v1(this_tenancy, this_visibility || jsonb_build_object('visibility_deleted_at', NULL), components)
        ORDER BY id,
            visibility_change_set_pk DESC,
            visibility_deleted_at DESC
    ) as components ON components.comp_id = av.attribute_context_component_id
    WHERE in_attribute_context_v1(this_attribute_context, av)
        AND CASE
                WHEN this_key IS NULL THEN key IS NULL
                ELSE key = this_key
            END
        AND CASE
                WHEN components.comp_id != ident_nil_v1() THEN
                    (components.visibility_deleted_at IS NOT NULL AND components.needs_destroy) OR components.visibility_deleted_at IS NULL
                ELSE true
        END
    ORDER BY COALESCE(belongs_to_id, ident_nil_v1()),
            attribute_context_prop_id DESC,
            attribute_context_internal_provider_id DESC,
            attribute_context_external_provider_id DESC,
            COALESCE(key, ''),
            attribute_context_component_id DESC
$$;

CREATE OR REPLACE FUNCTION attribute_value_new_v1(this_tenancy                      jsonb,
                                                  this_visibility                   jsonb,
                                                  this_func_binding_id              ident,
                                                  this_func_binding_return_value_id ident,
                                                  this_attribute_context            jsonb,
                                                  this_key                          text,
                                                  OUT new_attribute_value           jsonb
)
AS
$$
DECLARE
    found_attribute_value attribute_values;
BEGIN
    IF attribute_context_least_specific_is_provider_v1(this_attribute_context) THEN
        SELECT *
        INTO found_attribute_value
        FROM attribute_value_find_with_key_in_context_v1(this_tenancy, this_visibility, this_key, this_attribute_context) AS av
        WHERE in_tenancy_v1(this_tenancy, av);

        IF FOUND
           AND attribute_contexts_match_v1(this_attribute_context, found_attribute_value) THEN
               RAISE 'Found duplicate AttributeValue(%) for provider context(%), Tenancy(%), Visibility(%)',
                     found_attribute_value.id,
                     this_attribute_context,
                     this_tenancy,
                     this_visibility;
        END IF;
    END IF;

    new_attribute_value := attribute_value_create_v1(this_tenancy,
                                                     this_visibility,
                                                     this_attribute_context,
                                                     this_func_binding_id,
                                                     this_func_binding_return_value_id,
                                                     this_key);
END;
$$ LANGUAGE PLPGSQL;

CREATE OR REPLACE FUNCTION attribute_prototype_update_for_context_v1(this_tenancy                      jsonb,
                                                                     this_visibility                   jsonb,
                                                                     this_attribute_prototype_id       ident,
                                                                     this_attribute_context            jsonb,
                                                                     this_func_id                      ident,
                                                                     this_func_binding_id              ident,
                                                                     this_func_binding_return_value_id ident,
                                                                     this_parent_attribute_value_id    ident,
                                                                     this_existing_attribute_value_id  ident,
                                                                     OUT new_attribute_prototype_id    ident
)
AS
$$
DECLARE
    given_attribute_prototype attribute_prototypes%ROWTYPE;
    new_attribute_prototype   attribute_prototypes%ROWTYPE;
BEGIN
    SELECT DISTINCT ON (id) *
    INTO given_attribute_prototype
    FROM attribute_prototypes
    WHERE in_tenancy_and_visible_v1(this_tenancy, this_visibility, attribute_prototypes)
          AND id = this_attribute_prototype_id
    ORDER BY id,
             visibility_change_set_pk DESC,
             visibility_deleted_at DESC NULLS FIRST;

    IF attribute_contexts_match_v1(this_attribute_context, given_attribute_prototype) THEN
        new_attribute_prototype_id := given_attribute_prototype.id;
    ELSIF this_existing_attribute_value_id IS NOT NULL THEN
        new_attribute_prototype_id := attribute_prototype_new_with_attribute_value_v1(this_tenancy,
                                                                                      this_visibility,
                                                                                      this_func_id,
                                                                                      this_attribute_context,
                                                                                      given_attribute_prototype.key,
                                                                                      this_parent_attribute_value_id,
                                                                                      this_existing_attribute_value_id);

        PERFORM update_by_id_v1('attribute_values',
                                'func_binding_id',
                                this_tenancy,
                                this_visibility,
                                this_existing_attribute_value_id,
                                this_func_binding_id);
    ELSE
        new_attribute_prototype := json_populate_record(null::attribute_prototypes,
                                                        attribute_prototype_new_v1(this_tenancy,
                                                                                   this_visibility,
                                                                                   this_func_id,
                                                                                   this_func_binding_id,
                                                                                   this_func_binding_return_value_id,
                                                                                   this_attribute_context,
                                                                                   given_attribute_prototype.key,
                                                                                   this_parent_attribute_value_id));
        new_attribute_prototype_id := new_attribute_prototype.id;
    END IF;

    PERFORM update_by_id_v1('attribute_prototypes',
                            'func_id',
                            this_tenancy,
                            this_visibility,
                            new_attribute_prototype_id,
                            this_func_id);
END;
$$ LANGUAGE PLPGSQL;

CREATE OR REPLACE FUNCTION func_binding_create_and_execute_v1(
    this_tenancy                         jsonb,
    this_visibility                      jsonb,
    this_func_args                       jsonb,
    this_func_id                         ident,
    OUT new_func_binding_id              ident,
    OUT new_func_binding_return_value_id ident
)
AS
$$
DECLARE
    func_backend_kind         text;
    func_binding              func_bindings%ROWTYPE;
    func_binding_json         jsonb;
    func_binding_created      bool;
    func_binding_return_value func_binding_return_values%ROWTYPE;
    code_sha                  text;
BEGIN
    RAISE DEBUG 'func_binding_create_and_execute_v1: Args(%), FuncId(%)', this_func_args, this_func_id;
    SELECT backend_kind, code_sha256
    INTO STRICT func_backend_kind, code_sha
    FROM funcs_v1(this_tenancy, this_visibility)
    WHERE id = this_func_id;

    SELECT object
    INTO STRICT func_binding_json
    FROM func_binding_create_v1(
        this_tenancy,
        this_visibility,
        this_func_args::json,
        this_func_id,
        func_backend_kind,
        code_sha
    );
    func_binding := jsonb_populate_record(null::func_bindings, func_binding_json);
    new_func_binding_id := func_binding.id;

    SELECT func_binding_return_value_id
    INTO STRICT new_func_binding_return_value_id
    FROM func_binding_execute_v1(
        this_tenancy,
        this_visibility,
        new_func_binding_id
    );
END;
$$ LANGUAGE PLPGSQL;

DROP FUNCTION IF EXISTS attribute_value_child_attribute_values_for_context_v1;
CREATE OR REPLACE FUNCTION attribute_value_child_attribute_values_for_context_v1(
    this_tenancy                     jsonb,
    this_visibility                  jsonb,
    this_original_attribute_value_id ident,
    this_read_attribute_context      jsonb)
RETURNS SETOF attribute_values
LANGUAGE sql
STABLE
PARALLEL SAFE
AS $$
    SELECT DISTINCT ON (attribute_context_prop_id, COALESCE(key, ''))
        av.*
    FROM attribute_values_v1(this_tenancy, this_visibility) AS av
    INNER JOIN attribute_value_belongs_to_attribute_value_v1(this_tenancy, this_visibility) AS avbtav
        ON avbtav.object_id = av.id
            AND avbtav.belongs_to_id = this_original_attribute_value_id
    LEFT JOIN (
        SELECT DISTINCT ON (id)
            id as comp_id,
            needs_destroy,
            visibility_deleted_at
        FROM components
        WHERE in_tenancy_and_visible_v1(this_tenancy, this_visibility || jsonb_build_object('visibility_deleted_at', NULL), components)
        ORDER BY id,
            visibility_change_set_pk DESC,
            visibility_deleted_at DESC
    ) as components ON components.comp_id = av.attribute_context_component_id
    WHERE in_attribute_context_v1(this_read_attribute_context, av)
        AND CASE
            WHEN components.comp_id != ident_nil_v1() THEN
                (components.visibility_deleted_at IS NOT NULL AND components.needs_destroy) OR components.visibility_deleted_at IS NULL
            ELSE true
        END
    ORDER BY attribute_context_prop_id,
                COALESCE(key, ''),
                attribute_context_internal_provider_id DESC,
                attribute_context_external_provider_id DESC,
                attribute_context_component_id DESC
$$;

CREATE OR REPLACE FUNCTION attribute_value_populate_child_proxies_for_value_v1(
    this_tenancy                     jsonb,
    this_visibility                  jsonb,
    this_original_attribute_value_id ident,
    this_previous_attribute_context  jsonb,
    this_attribute_value_id          ident,
    OUT new_proxy_value_ids          ident[]
) AS
$$
DECLARE
    child_attribute_value_prototype attribute_prototypes%ROWTYPE;
    new_child_value                 attribute_values%ROWTYPE;
    original_child_value            attribute_values%ROWTYPE;
    original_child_values           jsonb[];
    read_attribute_context          jsonb;
    write_attribute_context         jsonb;
BEGIN
    read_attribute_context := this_previous_attribute_context || jsonb_build_object('attribute_context_prop_id', NULL);

    FOR original_child_value IN
        SELECT *
        FROM attribute_value_child_attribute_values_for_context_v1(this_tenancy,
                                                                   this_visibility,
                                                                   this_original_attribute_value_id,
                                                                   read_attribute_context)
    LOOP
        write_attribute_context := this_previous_attribute_context || jsonb_build_object('attribute_context_prop_id', original_child_value.attribute_context_prop_id);

        IF attribute_contexts_match_v1(write_attribute_context, original_child_value) THEN
            -- The `AttributeValue` that we found is one that was already set in the desired
            -- `AttributeContext`, but its parent was from a less-specific `AttributeContext`. Since it now has
            -- an appropriate parent `AttributeValue` within the desired `AttributeContext`, we need to have it
            -- under that parent instead of the old one.
            PERFORM set_belongs_to_v1(
                'attribute_value_belongs_to_attribute_value',
                this_tenancy,
                this_visibility,
                original_child_value.id,
                this_attribute_value_id
            );
        ELSE
            -- Since there isn't already an `AttributeValue` to represent the one from a less-specific
            -- `AttributeContext`, we need to create a proxy `AttributeValue` in the desired `AttributeContext`
            -- so that we can do things like add it to the `IndexMap` of the parent (that exists in the desired
            -- context).
            new_child_value := jsonb_populate_record(null::attribute_values,
                                                     attribute_value_new_v1(this_tenancy,
                                                                            this_visibility,
                                                                            original_child_value.func_binding_id,
                                                                            original_child_value.func_binding_return_value_id,
                                                                            write_attribute_context,
                                                                            original_child_value.key));

            new_proxy_value_ids := array_append(new_proxy_value_ids, new_child_value.id);
            SELECT ap.*
            INTO STRICT child_attribute_value_prototype
            FROM attribute_prototypes_v1(this_tenancy, this_visibility) AS ap
            INNER JOIN attribute_value_belongs_to_attribute_prototype_v1(this_tenancy, this_visibility) AS avbtap
                ON avbtap.belongs_to_id = ap.id
                    AND avbtap.object_id = original_child_value.id;

            PERFORM set_belongs_to_v1(
                'attribute_value_belongs_to_attribute_prototype',
                this_tenancy,
                this_visibility,
                new_child_value.id,
                child_attribute_value_prototype.id
            );
            PERFORM set_belongs_to_v1(
                'attribute_value_belongs_to_attribute_value',
                this_tenancy,
                this_visibility,
                new_child_value.id,
                this_attribute_value_id
            );
            PERFORM update_by_id_v1('attribute_values',
                                    'proxy_for_attribute_value_id',
                                    this_tenancy,
                                    this_visibility,
                                    new_child_value.id,
                                    original_child_value.id);

            -- Now that we've created a proxy `AttributeValue`, we need to create proxies for all of the
            -- original value's children.
            new_proxy_value_ids := array_cat(new_proxy_value_ids,
                                             attribute_value_populate_child_proxies_for_value_v1(this_tenancy,
                                                                                                 this_visibility,
                                                                                                 original_child_value.id,
                                                                                                 write_attribute_context,
                                                                                                 new_child_value.id));
        END IF;
    END LOOP;
END;
$$ LANGUAGE PLPGSQL;

CREATE OR REPLACE FUNCTION attribute_value_update_for_context_raw_v1(this_tenancy                         jsonb,
                                                                     this_visibility                      jsonb,
                                                                     this_attribute_value_id              ident,
                                                                     this_maybe_parent_attribute_value_id ident,
                                                                     this_attribute_context               jsonb,
                                                                     this_new_value                       jsonb,
                                                                     this_key                             text,
                                                                     this_create_child_proxies            bool,
                                                                     OUT new_attribute_value_id           ident
)
AS
$$
DECLARE
    attribute_prototype_id          ident;
    attribute_value_id              ident;
    func                            funcs%ROWTYPE;
    func_args                       jsonb;
    func_binding                    func_bindings%ROWTYPE;
    func_binding_created            bool;
    func_binding_id                 ident;
    func_binding_return_value       func_binding_return_values;
    func_binding_return_value_id    ident;
    func_name                       text;
    given_attribute_value           attribute_values%ROWTYPE;
    maybe_attribute_value           attribute_values%ROWTYPE;
    maybe_parent_attribute_value_id ident;
    original_attribute_prototype    attribute_prototypes%ROWTYPE;
    parent_attribute_context        jsonb;
    parent_attribute_value          attribute_values%ROWTYPE;
    prop                            props%ROWTYPE;
    typeof_value                    text;
BEGIN
    RAISE DEBUG 'attribute_value_update_for_context_raw_v1: Tenancy(%), Visibility(%) AttributeValue(%) ParentAttributeValue(%) AttributeContext(%) Value(%) Key(%) CreateChild(%)',
        this_tenancy,
        this_visibility,
        this_attribute_value_id,
        this_maybe_parent_attribute_value_id,
        this_attribute_context,
        this_new_value,
        this_key,
        this_create_child_proxies;
    maybe_parent_attribute_value_id := this_maybe_parent_attribute_value_id;

    SELECT *
    INTO given_attribute_value
    FROM attribute_values_v1(this_tenancy, this_visibility) AS av
    WHERE id = this_attribute_value_id;
    IF NOT FOUND THEN
        RAISE 'Unable to find AttributeValue(%) in Tenancy(%), Visibility(%)', this_attribute_value_id,
                                                                               this_tenancy,
                                                                               this_visibility;
    END IF;

    SELECT ap.*
    INTO original_attribute_prototype
    FROM attribute_prototypes_v1(this_tenancy, this_visibility) AS ap
    INNER JOIN attribute_value_belongs_to_attribute_prototype_v1(this_tenancy, this_visibility) AS avbtap
        ON avbtap.belongs_to_id = ap.id
            AND avbtap.object_id = given_attribute_value.id;
    IF original_attribute_prototype IS NULL THEN
        SELECT INTO func_binding_id FROM attribute_value_belongs_to_attribute_prototype as avbtap where avbtap.object_id = given_attribute_value.id;
        RAISE WARNING '%', func_binding_id;
        RAISE 'Unable to find AttributePrototype for AttributeValue(%), Tenancy(%), Visibility(%)', given_attribute_value.id,
                                                                                                    this_tenancy,
                                                                                                    this_visibility;
    END IF;

    -- We need to make sure that all of the parents "exist" (are not the "unset" value).  We can't rely on the
    -- client having created/set all of the parents already, as the parent might be an Object, or an Array/Map
    -- (instead of an element in an Array/Map).  The client will only be creating new elements in Arrays/Maps,
    -- and not Objects/Arrays/Maps themselves (unless the Object/Array/Map itself is the element of an
    -- Array/Map).
    IF maybe_parent_attribute_value_id IS NOT NULL THEN
        SELECT *
        INTO parent_attribute_value
        FROM attribute_values_v1(this_tenancy, this_visibility) AS av
        WHERE id = maybe_parent_attribute_value_id;
        IF NOT FOUND THEN
            RAISE 'Unable to find parent AttributeValue(%) in Tenancy(%), Visibility(%)',
                  maybe_parent_attribute_value_id,
                  this_tenancy,
                  this_visibility;
        END IF;

        parent_attribute_context := this_attribute_context || jsonb_build_object('attribute_context_prop_id', parent_attribute_value.attribute_context_prop_id);

        maybe_parent_attribute_value_id := attribute_value_vivify_value_and_parent_values_raw_v1(this_tenancy,
                                                                                                 this_visibility,
                                                                                                 parent_attribute_context,
                                                                                                 parent_attribute_value.id,
                                                                                                 this_create_child_proxies);
    END IF;

    -- If the AttributeValue we were given isn't for the _specific_ context that we're trying to update, make a
    -- new one. This is necessary, since the one that we were given might be the "default" one that is directly
    -- attached to a Prop, or the one from a SchemaVariant, and the AttributeContext might be requesting that
    -- we set the value in a more specific context.
    IF attribute_contexts_match_v1(this_attribute_context, given_attribute_value) THEN
        attribute_value_id := given_attribute_value.id;
    ELSE
        -- Check if we created an appropriate AttributeValue in the process of vivifying the parent
        -- `AttributeValue`s, and populating proxy `AttributeValue`s for their child `AttributeValue`s.
        maybe_attribute_value := jsonb_populate_record(NULL::attribute_values,
                                                       attribute_value_find_with_parent_and_key_for_context_v1(this_tenancy,
                                                                                                               this_visibility,
                                                                                                               maybe_parent_attribute_value_id,
                                                                                                               given_attribute_value.key,
                                                                                                               this_attribute_context));
        IF maybe_attribute_value.id IS NOT NULL
           AND attribute_contexts_match_v1(this_attribute_context, maybe_attribute_value)
        THEN
            attribute_value_id := maybe_attribute_value.id;
        ELSE
            -- We haven't found an appropriate AttributeValue to use, so we need to make one.
            SELECT (av.object ->> 'id')::ident
            INTO attribute_value_id
            FROM attribute_value_create_v1(this_tenancy,
                                           this_visibility,
                                           this_attribute_context,
                                           given_attribute_value.func_binding_id,
                                           given_attribute_value.func_binding_return_value_id,
                                           given_attribute_value.key) AS av;
            IF NOT FOUND THEN
                RAISE 'Unable to create AttributeValue: attribute_value_create_v1(%, %, %, %, %, %)',
                      this_tenancy,
                      this_visibility,
                      this_attribute_context,
                      given_attribute_value.func_binding_id,
                      given_attribute_value.func_binding_return_value_id,
                      given_attribute_value.key;
            END IF;

            IF maybe_parent_attribute_value_id IS NOT NULL THEN
                PERFORM set_belongs_to_v1(
                    'attribute_value_belongs_to_attribute_value',
                    this_tenancy,
                    this_visibility,
                    attribute_value_id,
                    maybe_parent_attribute_value_id
                );
            END IF;

            IF this_create_child_proxies THEN
                PERFORM attribute_value_populate_child_proxies_for_value_v1(this_tenancy,
                                                                            this_visibility,
                                                                            given_attribute_value.id,
                                                                            this_attribute_context,
                                                                            attribute_value_id);
            END IF;
        END IF;
    END IF;

    RAISE DEBUG 'attribute_value_update_for_context_raw_v1: this_attribute_context - %', this_attribute_context;
    IF (this_attribute_context ->> 'attribute_context_prop_id')::ident = ident_nil_v1() THEN
        typeof_value := jsonb_typeof(this_new_value);

        -- jsonb_typeof returns: 'object', 'array', 'string', 'number', 'boolean', 'null' and SQL NULL
        --
        -- json_typeof('null'::json) → null
        -- json_typeof(NULL::json) IS NULL → t
        CASE
            WHEN typeof_value = 'object' THEN
                -- It's an array/map, but since we're setting the value for a Provider, then it's an Object.
                func_name := 'si:setObject';
            WHEN typeof_value = 'array' THEN
                func_name := 'si:setArray';
            WHEN typeof_value = 'string' THEN
                func_name := 'si:setString';
            WHEN typeof_value = 'number' THEN
                -- This should be whatever our floating point func is when that becomes a thing, since
                -- jsonb_typeof doesn't differentiate between integer & float.
                func_name := 'si:setInteger';
            WHEN typeof_value = 'boolean' THEN
                func_name := 'si:setBoolean';
            WHEN typeof_value = 'null' THEN
                -- This should probably be different from 'si:unset' so we can differentiate between
                -- "this doesn't have a value/shouldn't exist" and "this should exist with the literal
                -- value 'nothing'".
                func_name := 'si:unset';
                func_args := 'null'::jsonb;
            WHEN typeof_value IS NULL THEN
                func_name := 'si:unset';
                func_args := 'null'::jsonb;
            ELSE
                RAISE 'attribute_value_update_for_context_raw_v1: Unknown jsonb_typeof(%) - %',
                    this_value,
                    typeof_value;
        END CASE;
    ELSE
        SELECT *
        INTO prop
        FROM props_v1(this_tenancy, this_visibility)
        WHERE id = (this_attribute_context ->> 'attribute_context_prop_id')::ident;
        IF NOT FOUND THEN
            RAISE 'Unable to find Prop(%) in Tenancy(%), Visibility(%)', (this_attribute_context ->> 'attribute_context_prop_id')::ident,
                                                                         this_tenancy,
                                                                         this_visibility;
        END IF;

        IF this_new_value IS NULL THEN
            func_name := 'si:unset';
            func_args := 'null'::jsonb;
        ELSIF prop.kind = 'array' THEN
            func_name := 'si:setArray';
        ELSIF prop.kind = 'boolean' THEN
            func_name := 'si:setBoolean';
        ELSIF prop.kind = 'integer' THEN
            func_name := 'si:setInteger';
        ELSIF prop.kind = 'map' THEN
            func_name := 'si:setMap';
        ELSIF prop.kind = 'object' THEN
            func_name := 'si:setObject';
        ELSIF prop.kind = 'string' THEN
            func_name := 'si:setString';
        ELSE
            RAISE 'Unknown Prop(%).kind(%) in Tenancy(%), Visibility(%)', prop.id, prop.kind, this_tenancy, this_visibility;
        END IF;
    END IF;

    IF func_args IS NULL THEN
        func_args := jsonb_build_object('value', this_new_value);
    END IF;

    SELECT *
    INTO func
    FROM funcs_v1(this_tenancy, this_visibility)
    WHERE name = func_name;
    IF NOT FOUND THEN
        RAISE 'Unable to find Func(%) in Tenancy(%), Visibility(%)', func_name,
                                                                     this_tenancy,
                                                                     this_visibility;
    END IF;

    SELECT new_func_binding_id, new_func_binding_return_value_id
    INTO func_binding_id, func_binding_return_value_id
    FROM func_binding_create_and_execute_v1(
        this_tenancy,
        this_visibility,
        func_args,
        func.id
    );

    PERFORM update_by_id_v1('attribute_values',
                            'func_binding_id',
                            this_tenancy,
                            this_visibility,
                            attribute_value_id,
                            func_binding_id);

    attribute_prototype_id := attribute_prototype_update_for_context_v1(this_tenancy,
                                                                        this_visibility,
                                                                        original_attribute_prototype.id,
                                                                        this_attribute_context,
                                                                        func.id,
                                                                        func_binding_id,
                                                                        func_binding_return_value_id,
                                                                        maybe_parent_attribute_value_id,
                                                                        attribute_value_id);
    IF attribute_prototype_id IS NULL THEN
        RAISE 'Unable create AttributePrototype: attribute_prototype_update_for_context_v1(%, %, %, %, %, %, %, %, %)',
              this_tenancy,
              this_visibility,
              original_attribute_prototype.id,
              this_attribute_context,
              func.id,
              func_binding_id,
              func_binding_return_value_id,
              maybe_parent_attribute_value_id,
              attribute_value_id;
    END IF;

    PERFORM set_belongs_to_v1(
        'attribute_value_belongs_to_attribute_prototype',
        this_tenancy,
        this_visibility,
        attribute_value_id,
        attribute_prototype_id
    );

    PERFORM update_by_id_v1('attribute_values',
                            'func_binding_return_value_id',
                            this_tenancy,
                            this_visibility,
                            attribute_value_id,
                            func_binding_return_value_id);

    -- If the value we just updated is a proxy, we need to seal it to prevent it from automatically updated
    -- by the AttributeValue it is proxying, since we overrode that value.
    IF av.proxy_for_attribute_value_id IS NOT NULL
        FROM attribute_values_v1(this_tenancy, this_visibility) AS av
        WHERE id = attribute_value_id
    THEN
        PERFORM update_by_id_v1('attribute_values',
                                'sealed_proxy',
                                this_tenancy,
                                this_visibility,
                                attribute_value_id,
                                true);
    END IF;

    PERFORM attribute_value_update_parent_index_map_v1(this_tenancy,
                                                       this_visibility,
                                                       attribute_value_id);

    -- Do we need to process the unprocessed value and populate nested values?  If the unprocessed value
    -- doesn't equal the value then we have a populated "container" (i.e. object, map, array) that contains
    -- values which need to be made into AttributeValues of their own.
    SELECT *
    INTO func_binding_return_value
    FROM func_binding_return_values_v1(this_tenancy, this_visibility)
    WHERE id = func_binding_return_value_id;
    IF func_binding_return_value.unprocessed_value IS NOT NULL
        AND func_binding_return_value.unprocessed_value != func_binding_return_value.value
    THEN
        PERFORM attribute_value_populate_nested_values_v1(this_tenancy,
                                                          this_visibility,
                                                          attribute_value_id,
                                                          this_attribute_context,
                                                          func_binding_return_value.unprocessed_value);
    END IF;

    new_attribute_value_id := attribute_value_id;
END;
$$ LANGUAGE PLPGSQL;

CREATE OR REPLACE FUNCTION index_map_push_v1(this_index_map          jsonb,
                                             this_attribute_value_id ident,
                                             this_value_key          text,
                                             OUT updated_index_map   jsonb
)
AS
$$
DECLARE
    base_index_map  jsonb;
    index_hashmap   jsonb;
    index_order     jsonb;
    insertion_value text;
    order_set       ident[];
BEGIN
    IF this_index_map IS NULL THEN
        base_index_map := jsonb_build_object('order', '[]'::jsonb,
                                             'key_map', '{}'::jsonb);
    ELSE
        base_index_map := this_index_map;
    END IF;

    -- Append this_attribute_value_id to the end or the order array.
    index_order := jsonb_extract_path(base_index_map, 'order') || to_jsonb(ARRAY[this_attribute_value_id]);
    -- This will de-duplicate the 'order' array of AttributeValueIds, while maintaining their original order.
    SELECT array_agg(value ORDER BY position)
    INTO order_set
    FROM (
        SELECT DISTINCT ON (value) value, position
        -- `WITH ORDINALITY` gives us the rows, with the numerical index of what order they appeared.
        FROM jsonb_array_elements_text(index_order) WITH ORDINALITY AS jaet(value, position)
        ORDER BY value, position
    ) AS p(value, position);

    insertion_value := CASE
                           WHEN this_value_key IS NULL THEN
                               uuid_generate_v4()::text
                           ELSE
                               this_value_key
                       END;
    -- Insert our new value into the AttributeValueId -> Key hashmap.
    index_hashmap := jsonb_extract_path(base_index_map, 'key_map') || jsonb_build_object(this_attribute_value_id, insertion_value);


    updated_index_map := jsonb_build_object('order', order_set,
                                            'key_map', index_hashmap);
END;
$$ LANGUAGE PLPGSQL;

CREATE OR REPLACE FUNCTION attribute_value_update_parent_index_map_v1(this_tenancy            jsonb,
                                                                      this_visibility         jsonb,
                                                                      this_attribute_value_id ident) RETURNS void
AS
$$
DECLARE
    child_key                 text;
    parent_attribute_value_id ident;
    parent_prop_kind          text;
    parent_prop_id            ident;
    parent_index_map          jsonb;
BEGIN
    SELECT av.id, index_map, p.kind
    INTO parent_attribute_value_id, parent_index_map, parent_prop_kind
    FROM attribute_values_v1(this_tenancy, this_visibility) AS av
    INNER JOIN attribute_value_belongs_to_attribute_value_v1(this_tenancy, this_visibility) AS avbtav
        ON avbtav.belongs_to_id = av.id
            AND avbtav.object_id = this_attribute_value_id
    INNER JOIN props_v1(this_tenancy, this_visibility) AS p
        ON p.id = av.attribute_context_prop_id;

    IF parent_attribute_value_id IS NULL
       OR (parent_prop_kind != 'array' AND parent_prop_kind != 'map') THEN
        RETURN;
    END IF;

    SELECT key
    INTO STRICT child_key
    FROM attribute_values_v1(this_tenancy, this_visibility) AS av
    WHERE id = this_attribute_value_id;

    PERFORM update_by_id_v1('attribute_values',
                            'index_map',
                            this_tenancy,
                            this_visibility,
                            parent_attribute_value_id,
                            index_map_push_v1(parent_index_map, this_attribute_value_id, child_key));
END;
$$ LANGUAGE PLPGSQL;

CREATE OR REPLACE FUNCTION attribute_value_populate_nested_values_v1(this_tenancy                   jsonb,
                                                                     this_visibility                jsonb,
                                                                     this_parent_attribute_value_id ident,
                                                                     this_update_attribute_context  jsonb,
                                                                     unprocessed_value              jsonb) RETURNS void
AS
$$
DECLARE
    attribute_value                    attribute_values%ROWTYPE;
    array_element                      jsonb;
    child_props                        props[];
    child_value_id                     ident;
    field_context                      jsonb;
    field_value                        jsonb;
    invalid_object_keys                text[];
    invalid_object_key                 text;
    map_key                            text;
    object_field_prop_id               ident;
    object_field_prop_name             text;
    object_field_prop_kind             text;
    object_keys                        text[];
    parent_attribute_value             attribute_values%ROWTYPE;
    parent_prop                        props%ROWTYPE;
    prop_keys                          text[];
    unset_func_id                      ident;
    unset_func_binding_id              ident;
    unset_func_binding_return_value_id ident;
    update_read_attribute_context      jsonb;
BEGIN
    SELECT *
    INTO parent_attribute_value
    FROM attribute_values_v1(this_tenancy, this_visibility) AS av
    WHERE id = this_parent_attribute_value_id;
    IF NOT FOUND THEN
        RAISE 'Unable to find parent AttributeValue(%) with Tenancy(%) and Visibility(%)', this_parent_attribute_value_id, this_tenancy, this_visibility;
    END IF;

    SELECT *
    INTO parent_prop
    FROM props_v1(this_tenancy, this_visibility)
    WHERE id = parent_attribute_value.attribute_context_prop_id;
    IF NOT FOUND THEN
        RAISE 'Unable to find parent Prop(%) with Tenancy(%) and Visibility(%)', parent_attribute_value.attribute_context_prop_id, this_tenancy, this_visibility;
    END IF;

    update_read_attribute_context := this_update_attribute_context || jsonb_build_object('attribute_context_prop_id', NULL);
    FOR child_value_id IN
        SELECT DISTINCT ON (
            avbtav.belongs_to_id,
            attribute_context_prop_id,
            COALESCE(key, '')
        )
            av.id
        FROM attribute_values_v1(this_tenancy, this_visibility) AS av
        INNER JOIN attribute_value_belongs_to_attribute_value_v1(this_tenancy, this_visibility) AS avbtav
            ON avbtav.object_id = av.id
                AND avbtav.belongs_to_id = this_parent_attribute_value_id
        WHERE exact_attribute_context_v1(update_read_attribute_context, av)
    LOOP
        PERFORM attribute_value_remove_value_and_children_v1(this_tenancy,
                                                             this_visibility,
                                                             child_value_id);
    END LOOP;

    IF parent_prop.kind = 'object' THEN
        IF jsonb_typeof(unprocessed_value) != 'object' THEN
            RAISE 'attribute_value_populate_nested_values_v1: Unexpected data kind(%) for Prop(%) Kind(%), Tenancy(%), Visibility(%)',
                  jsonb_typeof(unprocessed_value),
                  parent_prop.id,
                  parent_prop.kind,
                  this_tenancy,
                  this_visibility;
        END IF;

        SELECT array_agg(obj.key)
        INTO invalid_object_keys
        FROM jsonb_object_keys(unprocessed_value) AS obj(key)
        LEFT JOIN props_v1(this_tenancy, this_visibility) AS p
            ON p.name = obj.key
        LEFT JOIN prop_belongs_to_prop_v1(this_tenancy, this_visibility) AS pbtp
            ON pbtp.object_id = p.id
                AND pbtp.belongs_to_id = parent_prop.id
        WHERE p.name IS NULL;
        -- Can't use `IF NOT FOUND` because `array_agg` will always be `FOUND`, even if it is still
        -- `NULL` because it didn't aggregate anything.
        IF invalid_object_keys IS NOT NULL THEN
	    -- We used to RAISE an error here, but populating /root/resource_value from /root/resource/payload
	    -- gets annoying if we don't ignore extra values, because it requires a custom function that builds
	    -- the custom /root/resource_value instead of a generic one that just deserializes the payload
	    FOREACH invalid_object_key IN ARRAY invalid_object_keys
	    LOOP
		RAISE WARNING 'Unable to find child Prop(name = %) for Parent PropId(%)', invalid_object_key, parent_prop.id;
		unprocessed_value := unprocessed_value - invalid_object_key;
	    END LOOP;
        END IF;

        SELECT id
        INTO unset_func_id
        FROM find_by_attr_v1('funcs',
                             this_tenancy,
                             this_visibility,
                             'name',
                             'si:unset');

        IF unset_func_id IS NULL THEN
            RAISE 'attribute_value_populate_nested_values_v1: Unable to find Func(si:unset), Tenancy(%), Visibility(%)',
                  this_tenancy,
                  this_visibility;
        END IF;
        SELECT new_func_binding_id, new_func_binding_return_value_id
        INTO unset_func_binding_id, unset_func_binding_return_value_id
        FROM func_binding_create_and_execute_v1(
            this_tenancy,
            this_visibility,
            'null'::jsonb,
            unset_func_id
        );

        FOR object_field_prop_id, object_field_prop_name, object_field_prop_kind IN
            SELECT DISTINCT ON (props.id) props.id, props.name, props.kind
            FROM props_v1(this_tenancy, this_visibility) as props
            INNER JOIN prop_belongs_to_prop_v1(this_tenancy, this_visibility) as pbtp
                ON pbtp.belongs_to_id = parent_prop.id
                   AND pbtp.object_id = props.id
        LOOP
            field_value := jsonb_extract_path(unprocessed_value, object_field_prop_name);
            IF field_value IS NULL THEN
		IF object_field_prop_kind = 'object' THEN
		    field_value := '{}'::jsonb;
		ELSIF object_field_prop_kind = 'map' THEN
		    field_value := '{}'::jsonb;
		ELSIF object_field_prop_kind = 'array' THEN
		    field_value := '[]'::jsonb;
		END IF;
            END IF;

            field_context := this_update_attribute_context || jsonb_build_object('attribute_context_prop_id', object_field_prop_id);
            attribute_value := jsonb_populate_record(null::attribute_values,
                                                     attribute_value_find_with_parent_and_key_for_context_v1(this_tenancy,
                                                                                                             this_visibility,
                                                                                                             this_parent_attribute_value_id,
                                                                                                             NULL,
                                                                                                             field_context));
            IF attribute_value IS NULL THEN
                attribute_value := jsonb_populate_record(null::attribute_values,
                                                         attribute_value_new_v1(this_tenancy,
                                                                                this_visibility,
                                                                                unset_func_binding_id,
                                                                                unset_func_binding_return_value_id,
                                                                                field_context,
                                                                                NULL));
                PERFORM set_belongs_to_v1(
                    'attribute_value_belongs_to_attribute_value',
                    this_tenancy,
                    this_visibility,
                    attribute_value.id,
                    this_parent_attribute_value_id
                );
                PERFORM attribute_prototype_new_with_attribute_value_v1(this_tenancy,
                                                                        this_visibility,
                                                                        unset_func_id,
                                                                        field_context,
                                                                        NULL,
                                                                        this_parent_attribute_value_id,
                                                                        attribute_value.id);
            END IF;

            PERFORM attribute_value_update_for_context_without_child_proxies_v1(this_tenancy,
                                                                                this_visibility,
                                                                                attribute_value.id,
                                                                                this_parent_attribute_value_id,
                                                                                field_context,
                                                                                field_value,
                                                                                NULL);
        END LOOP;
    ELSIF parent_prop.kind = 'array' THEN
        IF jsonb_typeof(unprocessed_value) != 'array' THEN
            RAISE 'attribute_value_populate_nested_values_v1: Unexpected data kind(%) for Prop(%) Kind(%), Tenancy(%), Visibility(%)',
                  jsonb_typeof(unprocessed_value),
                  parent_prop.id,
                  parent_prop.kind,
                  this_tenancy,
                  this_visibility;
        END IF;

        FOR array_element IN
            SELECT element
            FROM jsonb_array_elements(unprocessed_value) AS x(element)
        LOOP
            PERFORM attribute_value_insert_for_context_without_child_proxies_v1(this_tenancy,
                                                                                this_visibility,
                                                                                this_update_attribute_context,
                                                                                this_parent_attribute_value_id,
                                                                                array_element,
                                                                                NULL);
        END LOOP;
    ELSIF parent_prop.kind = 'map' THEN
        IF jsonb_typeof(unprocessed_value) != 'object' THEN
            RAISE 'attribute_value_populate_nested_values_v1: Unexpected data kind(%) for Prop(%) Kind(%), Tenancy(%), Visibility(%)',
                  jsonb_typeof(unprocessed_value),
                  parent_prop.id,
                  parent_prop.kind,
                  this_tenancy,
                  this_visibility;
        END IF;

        FOR map_key IN
            SELECT key_name
            FROM jsonb_object_keys(unprocessed_value) AS o(key_name)
        LOOP
            PERFORM attribute_value_insert_for_context_without_child_proxies_v1(this_tenancy,
                                                                                this_visibility,
                                                                                this_update_attribute_context,
                                                                                this_parent_attribute_value_id,
                                                                                jsonb_extract_path(unprocessed_value, map_key),
                                                                                map_key);
        END LOOP;
    ELSE
        RAISE 'attribute_value_populate_nested_values_v1: Unexpected PropKind(%), AttributeValue(%), Tenancy(%), Visibility(%)',
              parent_prop.kind,
              this_parent_attribute_value_id,
              this_tenancy,
              this_visibility;
    END IF;
END;
$$ LANGUAGE PLPGSQL;

DROP FUNCTION IF EXISTS attribute_value_remove_proxies_v1;
CREATE OR REPLACE FUNCTION attribute_value_remove_proxies_v1(this_tenancy            jsonb,
                                                             this_visibility         jsonb,
                                                             this_attribute_value_id ident) RETURNS void
AS
$$
DECLARE
    found_proxy RECORD;
BEGIN
    FOR found_proxy IN
        SELECT av.*
        FROM attribute_values_v1(this_tenancy, this_visibility) AS av
        WHERE proxy_for_attribute_value_id = this_attribute_value_id
        ORDER BY id
    LOOP
        IF found_proxy.sealed_proxy = TRUE THEN
            PERFORM update_by_id_v1('attribute_values',
                                    'proxy_for_attribute_value_id',
                                    this_tenancy,
                                    this_visibility,
                                    found_proxy.id,
                                    NULL);
            PERFORM update_by_id_v1('attribute_values',
                                    'sealed_proxy',
                                    this_tenancy,
                                    this_visibility,
                                    found_proxy.id,
                                    FALSE);
        ELSE
            PERFORM attribute_value_remove_proxies_v1(this_tenancy,
                                                      this_visibility,
                                                      found_proxy.id);
            PERFORM unset_belongs_to_v1(
                'attribute_value_belongs_to_attribute_prototype',
                this_tenancy,
                this_visibility,
                found_proxy.id
            );
            PERFORM unset_belongs_to_v1(
                'attribute_value_belongs_to_attribute_value',
                this_tenancy,
                this_visibility,
                found_proxy.id 
            );
            PERFORM delete_by_id_v1('attribute_values',
                                    this_tenancy,
                                    this_visibility,
                                    found_proxy.id);
        END IF;
    END LOOP;
END;
$$ LANGUAGE PLPGSQL;

CREATE OR REPLACE FUNCTION attribute_value_remove_value_and_children_v1(
    this_tenancy                   jsonb,
    this_visibility                jsonb,
    this_parent_attribute_value_id ident
) RETURNS void
AS
$$
DECLARE
    tmp_attribute_value_id ident;
    attribute_prototype_id ident;
    attribute_value_count  bigint;
BEGIN
    FOR tmp_attribute_value_id, attribute_prototype_id IN 
        -- Build a list of (AttributeValueId, AttributePrototypeId), starting with the initial
        -- AttributeValueId passed in to the function, and gathering all AttributeValueId (and
        -- their AttributePrototypeId) that are children of any generation of that starting
        -- AttributeValueId.
        WITH RECURSIVE av_ap(av_id, ap_id) AS (
            -- Get the (AttributeValueId, AttributePrototypeId) for the initial AttributeValueId
            SELECT
                av.id AS av_id,
                ap.id AS ap_id
            FROM attribute_values_v1(this_tenancy, this_visibility) AS av
            LEFT JOIN attribute_value_belongs_to_attribute_prototype_v1(this_tenancy, this_visibility) AS avbtap
                ON av.id = avbtap.object_id
            LEFT JOIN attribute_prototypes_v1(this_tenancy, this_visibility) AS ap
                ON avbtap.belongs_to_id = ap.id
            WHERE av.id = this_parent_attribute_value_id
            UNION
            -- For every AttributeValueId that we've already gotten, get the
            -- (AttributeValueId, AttributePrototypeId) of its children.
            SELECT
                av.id AS av_id,
                ap.id AS ap_id
            FROM av_ap
            INNER JOIN attribute_value_belongs_to_attribute_value_v1(this_tenancy, this_visibility) AS avbtav
                ON av_ap.av_id = avbtav.belongs_to_id
            INNER JOIN attribute_values_v1(this_tenancy, this_visibility) AS av
                ON avbtav.object_id = av.id
            LEFT JOIN attribute_value_belongs_to_attribute_prototype_v1(this_tenancy, this_visibility) AS avbtap
                ON av.id = avbtap.object_id
            LEFT JOIN attribute_prototypes_v1(this_tenancy, this_visibility) AS ap
                ON avbtap.belongs_to_id = ap.id
        )
        -- Remove things "depth first" (newest to oldest)
        SELECT * FROM av_ap ORDER BY av_ap.av_id DESC
    LOOP
        IF attribute_prototype_id IS NULL THEN
            RAISE WARNING 'Unable to find AttributePrototype of parent AttributeValue(%) with Tenancy(%) and Visibility(%)',
                tmp_attribute_value_id,
                this_tenancy,
                this_visibility;
        END IF;

        PERFORM attribute_value_remove_proxies_v1(
            this_tenancy,
            this_visibility,
            tmp_attribute_value_id
        );

        PERFORM unset_belongs_to_v1(
            'attribute_value_belongs_to_attribute_prototype',
            this_tenancy,
            this_visibility,
            tmp_attribute_value_id
        );

        PERFORM unset_belongs_to_v1(
            'attribute_value_belongs_to_attribute_value',
            this_tenancy,
            this_visibility,
            tmp_attribute_value_id
        );

        PERFORM delete_by_id_v1(
            'attribute_values',
            this_tenancy,
            this_visibility,
            tmp_attribute_value_id
        );
    END LOOP;
END;
$$ LANGUAGE PLPGSQL;

CREATE OR REPLACE FUNCTION func_binding_return_value_get_by_func_binding_id_v1(this_tenancy         jsonb,
                                                                               this_visibility      jsonb,
                                                                               this_func_binding_id ident,
                                                                               OUT fbrv             jsonb
)
AS
$$
BEGIN
    SELECT to_jsonb(func_binding_return_values.*)
    INTO fbrv
    FROM func_binding_return_values_v1(this_tenancy, this_visibility) as func_binding_return_values
    WHERE func_binding_return_values.func_binding_id = this_func_binding_id;
END;
$$ LANGUAGE PLPGSQL PARALLEL SAFE;

CREATE OR REPLACE FUNCTION func_binding_execute_v1(
    this_tenancy                     jsonb,
    this_visibility                  jsonb,
    this_func_binding_id             ident,
    OUT func_binding_return_value_id ident
)
AS
$$
DECLARE
    func                    funcs%ROWTYPE;
    func_binding            func_bindings%ROWTYPE;
    func_execution_pk       ident;
    fbrv_id                 ident;
    tenancy                 jsonb;
    result_value            jsonb;
    result_value_processed  jsonb;
BEGIN
    -- binding.prepare_execution
    SELECT *
    INTO STRICT func_binding
    FROM func_bindings_v1(this_tenancy, this_visibility)
    WHERE id = this_func_binding_id;
    RAISE DEBUG 'func_binding_execute_v1: Found FuncBinding(%)', func_binding;

    SELECT funcs.*
    INTO STRICT func
    FROM funcs_v1(this_tenancy, this_visibility) AS funcs
    INNER JOIN func_binding_belongs_to_func_v1(this_tenancy, this_visibility)
        AS func_binding_belongs_to_func
        ON funcs.id = func_binding_belongs_to_func.belongs_to_id
            AND func_binding_belongs_to_func.object_id = func_binding.id;
    RAISE DEBUG 'func_binding_execute_v1: Found Func(%)', func;

    SELECT (fe.object ->> 'pk')::ident
    INTO STRICT func_execution_pk
    FROM func_execution_create_v1(
        this_tenancy,
        'Start'::text,
        func.id,
        func_binding.id,
        func_binding.args::jsonb,
        func_binding.backend_kind,
        func.backend_response_type,
        func.handler,
        func.code_base64
    ) AS fe;
    RAISE DEBUG 'func_binding_execute_v1: Found FuncExecution(%)', func_execution_pk;
    PERFORM func_execution_set_state_v1(func_execution_pk, 'Run');

    -- FuncDispatchContext::new(read_context)
    --   Don't need. Copies the veritech handle and set up an mpsc::channel (for streaming output)

    -- binding.critical_section
    result_value := func_binding.args -> 'value';
    CASE
        WHEN func_binding.backend_kind = 'Array' THEN
            result_value_processed := '[]'::json;
        WHEN func_binding.backend_kind = 'Boolean' THEN
            result_value_processed := result_value;
        WHEN func_binding.backend_kind = 'Identity' THEN
            result_value := func_binding.args -> 'identity';
            result_value_processed := result_value;
        WHEN func_binding.backend_kind = 'Integer' THEN
            result_value_processed := result_value;
        WHEN func_binding.backend_kind = 'Map' THEN
            result_value_processed := '{}'::json;
        WHEN func_binding.backend_kind = 'Object' THEN
            result_value_processed := '{}'::json;
        WHEN func_binding.backend_kind = 'String' THEN
            result_value_processed := result_value;
        WHEN func_binding.backend_kind = 'Unset' THEN
            result_value := NULL;
            result_value_processed := result_value;
        ELSE
            RAISE 'BackendKind(%) cannot be executed directly in PG', func_binding.backend_kind;
    END CASE;

    -- binding.postprocess_execution
    fbrv_id := (func_binding_return_value_create_v1(
        this_tenancy,
        this_visibility,
        result_value,
        result_value_processed,
        func.id,
        func_binding.id,
        func_execution_pk
    ) ->> 'id')::ident;
    RAISE DEBUG 'func_binding_execute_v1: Created FuncBindingReturnValue(%)', fbrv_id;
    -- execution.process_return_value
    PERFORM func_execution_set_return_value_v1(
        func_execution_pk,
        fbrv_id,
        result_value_processed,
        result_value
    );
    RAISE DEBUG 'func_binding_execute_v1: Set FBRV on execution';
    PERFORM func_execution_set_state_v1(func_execution_pk, 'Success');

    RAISE DEBUG 'func_binding_execute_v1: DONE';
    func_binding_return_value_id := fbrv_id;
END;
$$ LANGUAGE PLPGSQL;

CREATE OR REPLACE FUNCTION attribute_contexts_match_v1(
    a_prop_id              text,
    a_internal_provider_id text,
    a_external_provider_id text,
    a_component_id         text,
    b_prop_id              text,
    b_internal_provider_id text,
    b_external_provider_id text,
    b_component_id         text
)
RETURNS bool
LANGUAGE sql
IMMUTABLE PARALLEL SAFE CALLED ON NULL INPUT
AS $$
    SELECT  a_prop_id              = b_prop_id
        AND a_internal_provider_id = b_internal_provider_id
        AND a_external_provider_id = b_external_provider_id
        AND a_component_id         = b_component_id
$$;

CREATE OR REPLACE FUNCTION attribute_contexts_match_v1(
    record_a         jsonb,
    record_b         jsonb
)
RETURNS bool
LANGUAGE sql
IMMUTABLE PARALLEL SAFE CALLED ON NULL INPUT
AS $$
    SELECT attribute_contexts_match_v1(
        (record_a ->> 'attribute_context_prop_id')::ident,
        (record_a ->> 'attribute_context_internal_provider_id')::ident,
        (record_a ->> 'attribute_context_external_provider_id')::ident,
        (record_a ->> 'attribute_context_component_id')::ident,
        (record_b ->> 'attribute_context_prop_id')::ident,
        (record_b ->> 'attribute_context_internal_provider_id')::ident,
        (record_b ->> 'attribute_context_external_provider_id')::ident,
        (record_b ->> 'attribute_context_component_id')::ident
    )
$$;

CREATE OR REPLACE FUNCTION attribute_contexts_match_v1(
    record_a jsonb,
    record_b attribute_values
)
RETURNS bool
LANGUAGE sql
IMMUTABLE PARALLEL SAFE CALLED ON NULL INPUT
AS $$
    SELECT attribute_contexts_match_v1(record_a, to_jsonb(record_b))
$$;

CREATE OR REPLACE FUNCTION attribute_contexts_match_v1(
    record_a jsonb,
    record_b attribute_prototypes
)
RETURNS bool
LANGUAGE sql
IMMUTABLE PARALLEL SAFE CALLED ON NULL INPUT
AS $$
    SELECT attribute_contexts_match_v1(record_a, to_jsonb(record_b))
$$;

-- CREATE OR REPLACE FUNCTION attribute_contexts_match_v1(
--     record_a RECORD,
--     record_b RECORD
-- )
-- RETURNS bool
-- LANGUAGE sql
-- IMMUTABLE PARALLEL SAFE CALLED ON NULL INPUT
-- AS $$
--     SELECT attribute_contexts_match_v1(
--         record_a.attribute_context_prop_id,
--         record_a.attribute_context_internal_provider_id,
--         record_a.attribute_context_external_provider_id,
--         record_a.attribute_context_component_id,
--         record_a.attribute_context_system_id,
--         record_b.attribute_context_prop_id,
--         record_b.attribute_context_internal_provider_id,
--         record_b.attribute_context_external_provider_id,
--         record_b.attribute_context_component_id,
--         record_b.attribute_context_system_id
--     )
-- $$;


-- AttributeValue::update_for_context_without_creating_proxies
CREATE OR REPLACE FUNCTION attribute_value_update_for_context_without_child_proxies_v1(this_tenancy                         jsonb,
                                                                                       this_visibility                      jsonb,
                                                                                       this_attribute_value_id              ident,
                                                                                       this_maybe_parent_attribute_value_id ident,
                                                                                       this_attribute_context               jsonb,
                                                                                       this_new_value                       jsonb,
                                                                                       this_key                             text,
                                                                                       OUT new_attribute_value_id           ident
)
AS
$$
BEGIN
    new_attribute_value_id := attribute_value_update_for_context_raw_v1(this_tenancy,
                                                                        this_visibility,
                                                                        this_attribute_value_id,
                                                                        this_maybe_parent_attribute_value_id,
                                                                        this_attribute_context,
                                                                        this_new_value,
                                                                        this_key,
                                                                        false);
END;
$$ LANGUAGE PLPGSQL;

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
    attribute_value                 attribute_values%ROWTYPE;
    prop                            props%ROWTYPE;
    empty_value                     jsonb;
    unset_func_id                   ident;
    func_id                         ident;
    maybe_parent_attribute_value_id ident;
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
END;
$$ LANGUAGE PLPGSQL;

CREATE OR REPLACE FUNCTION attribute_value_vivify_value_and_parent_no_child_proxies_v1(this_tenancy               jsonb,
                                                                                       this_visibility            jsonb,
                                                                                       this_attribute_context     jsonb,
                                                                                       this_attribute_value_id    ident,
                                                                                       OUT new_attribute_value_id ident
)
AS
$$
BEGIN
    new_attribute_value_id := attribute_value_vivify_value_and_parent_values_raw_v1(this_tenancy,
                                                                                    this_visibility,
                                                                                    this_attribute_context,
                                                                                    this_attribute_value_id,
                                                                                    false);
END;
$$ LANGUAGE PLPGSQL;

CREATE OR REPLACE FUNCTION attribute_context_build_from_parts_v1(this_prop_id              ident,
                                                                 this_internal_provider_id ident,
                                                                 this_external_provider_id ident,
                                                                 this_component_id         ident,
                                                                 OUT new_attribute_context jsonb
)
AS
$$
BEGIN
    new_attribute_context := jsonb_build_object('attribute_context_prop_id',              this_prop_id,
                                                'attribute_context_internal_provider_id', this_internal_provider_id,
                                                'attribute_context_external_provider_id', this_external_provider_id,
                                                'attribute_context_component_id',         this_component_id);
END;
$$ LANGUAGE PLPGSQL IMMUTABLE PARALLEL SAFE;

CREATE OR REPLACE FUNCTION attribute_value_insert_for_context_without_child_proxies_v1(this_tenancy                   jsonb,
                                                                                       this_visibility                jsonb,
                                                                                       this_parent_attribute_context  jsonb,
                                                                                       this_parent_attribute_value_id ident,
                                                                                       this_value                     jsonb,
                                                                                       this_key                       text,
                                                                                       OUT new_attribute_value_id     ident
)
AS
$$
BEGIN
    new_attribute_value_id := attribute_value_insert_for_context_raw_v1(this_tenancy,
                                                                        this_visibility,
                                                                        this_parent_attribute_context,
                                                                        this_parent_attribute_value_id,
                                                                        this_value,
                                                                        this_key,
                                                                        FALSE);
END;
$$ LANGUAGE PLPGSQL;

DROP TYPE IF EXISTS av_insert_for_context_raw_child_prop_record_v1;
CREATE TYPE av_insert_for_context_raw_child_prop_record_v1 AS (
    parent_attribute_value_id ident,
    prop_json                 jsonb
);

CREATE OR REPLACE FUNCTION props_find_for_attribute_value_v1(this_tenancy            jsonb,
                                                             this_visibility         jsonb,
                                                             this_attribute_value_id ident,
                                                             OUT found_prop jsonb)
AS
$$
BEGIN
    SELECT DISTINCT ON (props.id) to_jsonb(props.*)
    INTO found_prop
    FROM props
    INNER JOIN attribute_values_v1(this_tenancy, this_visibility) AS av ON
        av.id = this_attribute_value_id
        AND av.attribute_context_prop_id = props.id
    WHERE in_tenancy_and_visible_v1(this_tenancy, this_visibility, props)
    ORDER BY
        props.id,
        props.visibility_change_set_pk DESC,
        props.visibility_deleted_at DESC NULLS FIRST;
END;
$$ LANGUAGE PLPGSQL PARALLEL SAFE;

CREATE OR REPLACE FUNCTION attribute_value_insert_for_context_raw_v1(this_tenancy                   jsonb,
                                                                     this_visibility                jsonb,
                                                                     this_parent_attribute_context  jsonb,
                                                                     this_parent_attribute_value_id ident,
                                                                     this_value                     jsonb,
                                                                     this_key                       text,
                                                                     this_create_child_proxies      bool,
                                                                     OUT new_attribute_value_id     ident
)
AS
$$
DECLARE
    attribute_value                    attribute_values%ROWTYPE;
    child_attribute_context            jsonb;
    unset_child_attribute_context      jsonb;
    child_prop                         props%ROWTYPE;
    child_prop_info                    jsonb;
    child_props                        jsonb[];
    inserted_attribute_context         jsonb;
    key                                text;
    new_child_props                    av_insert_for_context_raw_child_prop_record_v1[];
    object_child_props                 av_insert_for_context_raw_child_prop_record_v1[];
    parent_prop                        props%ROWTYPE;
    prop_attribute_value               attribute_values%ROWTYPE;
    prototype                          attribute_prototypes%ROWTYPE;
    unset_func_id                      ident;
    unset_func_binding_id              ident;
    unset_func_binding_return_value_id ident;
    unset_func_name                    text;
BEGIN
    parent_prop := jsonb_populate_record(null::props, props_find_for_attribute_value_v1(this_tenancy,
                                                                                        this_visibility,
                                                                                        this_parent_attribute_value_id));
    IF parent_prop.id IS NULL THEN
        RAISE 'attribute_value_insert_for_context_raw_v1: Unable to find Prop for '
              'AttributeValue(%) in Tenancy(%), Visibility(%)',
              this_parent_attribute_value_id,
              this_tenancy,
              this_visibility;
    ELSIF parent_prop.kind != 'array' AND parent_prop.kind != 'map' THEN
        RAISE 'attribute_value_insert_for_context_raw_v1: Unable to insert child value for '
              'AttributeValue(%) kind(%), in Tenancy(%), Visibility(%)',
              this_parent_attribute_value_id,
              parent_prop.kind,
              this_tenancy,
              this_visibility;
    END IF;

    SELECT DISTINCT ON (id) props.*
    INTO child_prop
    FROM props
    INNER JOIN (
        SELECT DISTINCT ON (object_id) object_id AS child_prop_id
        FROM prop_belongs_to_prop
        WHERE in_tenancy_and_visible_v1(this_tenancy, this_visibility, prop_belongs_to_prop)
              AND belongs_to_id = parent_prop.id
        ORDER BY object_id,
                 visibility_change_set_pk DESC,
                 visibility_deleted_at DESC NULLS FIRST
    ) AS pbtp ON pbtp.child_prop_id = props.id
    ORDER BY id,
             visibility_change_set_pk DESC,
             visibility_deleted_at DESC NULLS FIRST;
    IF child_prop IS NULL THEN
        RAISE 'attribute_value_insert_for_context_raw_v1: Unable to find child Prop of Prop(%) in Tenancy(%), Visibility(%)',
              parent_prop.id,
              this_tenancy,
              this_visibility;
    END IF;

    child_attribute_context := this_parent_attribute_context || jsonb_build_object('attribute_context_prop_id', child_prop.id);
    inserted_attribute_context := child_attribute_context;

    IF this_key IS NOT NULL THEN
        key := this_key;
    ELSIF parent_prop.kind = 'array' THEN
        key := uuid_generate_v4();
    ELSE
        key := NULL;
    END IF;

    unset_func_name := 'si:unset';
    SELECT id
    INTO unset_func_id
    FROM find_by_attr_v1('funcs',
                         this_tenancy,
                         this_visibility,
                         'name',
                         unset_func_name);
    IF unset_func_id IS NULL THEN
        RAISE 'attribute_value_insert_for_context_raw_v1: Unable to find Func(%) in Tenancy(%), Visibility(%)',
              unset_func_name,
              this_tenancy,
              this_visibility;
    END IF;
    SELECT new_func_binding_id, new_func_binding_return_value_id
    INTO unset_func_binding_id, unset_func_binding_return_value_id
    FROM func_binding_create_and_execute_v1(
        this_tenancy,
        this_visibility,
        'null'::jsonb,
        unset_func_id
    );

    attribute_value := jsonb_populate_record(NULL::attribute_values, attribute_value_new_v1(this_tenancy,
                                                                                            this_visibility,
                                                                                            unset_func_binding_id,
                                                                                            unset_func_binding_return_value_id,
                                                                                            child_attribute_context,
                                                                                            key));
    IF attribute_value.id IS NULL THEN
        RAISE 'attribute_value_insert_for_context_raw_v1: Unable to create AttributeValue(%, %, %, %, %, %)',
              this_tenancy,
              this_visibility,
              unset_func_binding_id,
              unset_func_binding_return_value_id,
              child_attribute_context,
              key;
    END IF;

    PERFORM attribute_prototype_new_with_attribute_value_v1(this_tenancy,
                                                            this_visibility,
                                                            unset_func_id,
                                                            child_attribute_context,
                                                            key,
                                                            this_parent_attribute_value_id,
                                                            attribute_value.id);

    -- Create unset AttributePrototypes & AttributeValues for child Props up until (inclusive) we reach an
    -- Array/Map.
    IF child_prop.kind = 'object' THEN
        SELECT array_agg(to_jsonb(cp.*))
        INTO child_props
        FROM (
            SELECT DISTINCT ON (id) attribute_value.id AS parent_attribute_value_id,
                                    to_jsonb(props.*) AS prop_json
            FROM props
            INNER JOIN (
                SELECT DISTINCT ON (object_id) object_id AS child_prop_id
                FROM prop_belongs_to_prop
                WHERE in_tenancy_and_visible_v1(this_tenancy, this_visibility, prop_belongs_to_prop)
                      AND belongs_to_id = child_prop.id
                ORDER BY object_id,
                         visibility_change_set_pk DESC,
                         visibility_deleted_at DESC NULLS FIRST
            ) AS pbtp ON pbtp.child_prop_id = props.id
            ORDER BY id,
                     visibility_change_set_pk DESC,
                     visibility_deleted_at DESC NULLS FIRST
        ) AS cp;

        WHILE child_props IS NOT NULL LOOP
            FOREACH child_prop_info IN ARRAY child_props LOOP
                child_prop := jsonb_populate_record(NULL::props, child_prop_info -> 'prop_json');
                unset_child_attribute_context := child_attribute_context || jsonb_build_object('attribute_context_prop_id', child_prop.id);
                prop_attribute_value := jsonb_populate_record(NULL::attribute_values,
                                                              attribute_value_new_v1(this_tenancy,
                                                                                     this_visibility,
                                                                                     unset_func_binding_id,
                                                                                     unset_func_binding_return_value_id,
                                                                                     unset_child_attribute_context,
                                                                                     NULL));
                PERFORM set_belongs_to_v1(
                    'attribute_value_belongs_to_attribute_value',
                    this_tenancy,
                    this_visibility,
                    prop_attribute_value.id,
                    (child_prop_info ->> 'parent_attribute_value_id')::ident
                );
                -- XXX: The Rust code was originally using `child_prop_info.parent_attribute_value_id` as BOTH
                -- OF the last two arguments here, then setting prop_attribute_value's AttributePrototype to be
                -- the newly created AttributePrototype. This seems wrong, so instead we're passing in
                -- prop_attribute_value.id as the "this is the AttributeValue the AttributePrototype should use
                -- as its pre-existing one", and not explicitly setting that AttributeValue's
                -- AttributePrototype as _new_with_attribute_value_v1 will do that for us.
                PERFORM attribute_prototype_new_with_attribute_value_v1(this_tenancy,
                                                                        this_visibility,
                                                                        unset_func_id,
                                                                        unset_child_attribute_context,
                                                                        NULL,
                                                                        (child_prop_info ->> 'parent_attribute_value_id')::ident,
                                                                        prop_attribute_value.id);

                IF child_prop.kind = 'object' THEN
                    SELECT array_agg(*)
                    INTO object_child_props
                    FROM (
                        SELECT DISTINCT ON (id) prop_attribute_value.id AS parent_attribute_value_id,
                                                to_jsonb(props.*) AS prop_json
                        FROM props
                        INNER JOIN (
                            SELECT DISTINCT ON (object_id) object_id AS child_prop_id
                            FROM prop_belongs_to_prop
                            WHERE in_tenancy_and_visible_v1(this_tenancy, this_visibility, prop_belongs_to_prop)
                                  AND belongs_to_id = child_prop.id
                            ORDER BY object_id,
                                     visibility_change_set_pk DESC,
                                     visibility_deleted_at DESC
                        ) AS pbtp ON pbtp.child_prop_id = props.id
                        WHERE in_tenancy_and_visible_v1(this_tenancy, this_visibility, props)
                        ORDER BY id,
                                 visibility_change_set_pk DESC,
                                 visibility_deleted_at DESC NULLS FIRST
                    ) AS ocp;

                    new_child_props := array_cat(new_child_props, object_child_props);
                END IF;
            END LOOP;
            child_props := new_child_props;
            new_child_props := NULL;
        END LOOP;
    END IF;

    new_attribute_value_id := attribute_value_update_for_context_raw_v1(this_tenancy,
                                                                        this_visibility,
                                                                        attribute_value.id,
                                                                        this_parent_attribute_value_id,
                                                                        inserted_attribute_context,
                                                                        this_value,
                                                                        key,
                                                                        this_create_child_proxies);
END;
$$ LANGUAGE PLPGSQL;
