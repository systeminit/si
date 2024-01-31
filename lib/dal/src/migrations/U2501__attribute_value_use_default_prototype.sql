-- "Normmally", this would only be used for "prop" AttributeValues in a Component.
-- It is possible for Components to "override" the function used by Providers, but
-- there is currently no way for the frontend to achieve this (no API, and no UI
-- affordances).
--
-- Right now, this only works for "prop" AttributeValues in a Component. The full
-- support for resetting the prototype on AttributeValues will be implemented in
-- the new engine work.
CREATE OR REPLACE FUNCTION attribute_value_use_default_prototype_v1(
    this_tenancy jsonb,
    this_visibility jsonb,
    this_attribute_value_id ident,
    OUT changed bool
)
AS
$$
DECLARE
    this_attribute_value           attribute_values%ROWTYPE;
    this_prop_attribute_value      attribute_values%ROWTYPE;
    this_prop_attribute_prototype  attribute_prototypes%ROWTYPE;
    this_value_attribute_prototype attribute_prototypes%ROWTYPE;
BEGIN
    changed := false;

    SELECT av.*
    INTO this_attribute_value
    FROM attribute_values_v1(this_tenancy, this_visibility) AS av
    WHERE av.id = this_attribute_value_id
        AND av.attribute_context_component_id != ident_nil_v1()
        -- internal/external provider id will be ident_nil_v1() as long as prop_id is NOT.
        AND av.attribute_context_prop_id != ident_nil_v1();
    IF NOT FOUND THEN
        RETURN;
    END IF;

    -- The AttributePrototype for the Component's AttributeValue.
    SELECT ap.*
    INTO STRICT this_value_attribute_prototype
    FROM attribute_prototypes_v1(this_tenancy, this_visibility) AS ap
        INNER JOIN attribute_value_belongs_to_attribute_prototype_v1(this_tenancy, this_visibility) AS avbtap
            ON ap.id = avbtap.belongs_to_id
    WHERE avbtap.object_id = this_attribute_value_id;

    -- The AttributePrototype for the ScemaVariant
    SELECT ap.*
    INTO STRICT this_prop_attribute_prototype
    FROM attribute_prototypes_v1(this_tenancy, this_visibility) AS ap
        INNER JOIN attribute_value_belongs_to_attribute_prototype_v1(this_tenancy, this_visibility) AS avbtap
            ON ap.id = avbtap.belongs_to_id
        INNER JOIN attribute_values_v1(this_tenancy, this_visibility) AS av
            ON avbtap.object_id = av.id
    -- internal/external provider id will be ident_nil_v1() as long as prop_id is NOT.
    WHERE av.attribute_context_prop_id = this_attribute_value.attribute_context_prop_id
        AND av.attribute_context_component_id = ident_nil_v1();

    IF this_prop_attribute_prototype.id = this_value_attribute_prototype.id THEN
        -- We're already using the "default". Nothing to do.
        RETURN;
    END IF;

    SELECT av.*
    INTO STRICT this_prop_attribute_value
    FROM attribute_values_v1(this_tenancy, this_visibility) AS av
        INNER JOIN attribute_values_v1(this_tenancy, this_visibility) AS given_av
            ON av.attribute_context_prop_id = given_av.attribute_context_prop_id
    WHERE av.attribute_context_component_id = ident_nil_v1()
        AND given_av.id = this_attribute_value_id;


    -- PERFORM unset_belongs_to_v1(
    --     'attribute_value_belongs_to_attribute_prototype',
    --     this_tenancy,
    --     this_visibility,
    --     this_attribute_value_id
    -- );
    PERFORM set_belongs_to_v1(
        'attribute_value_belongs_to_attribute_prototype',
        this_tenancy,
        this_visibility,
        this_attribute_value_id,
        this_prop_attribute_prototype.id
    );
    PERFORM update_by_id_v1(
      'attribute_values',
      'func_binding_return_value_id',
      this_tenancy,
      this_visibility,
      this_attribute_value_id,
      this_prop_attribute_value.func_binding_return_value_id
    );
    PERFORM update_by_id_v1(
      'attribute_values',
      'func_binding_id',
      this_tenancy,
      this_visibility,
      this_attribute_value_id,
      this_prop_attribute_value.func_binding_id
    );
    changed := true;
END
$$ LANGUAGE PLPGSQL;
