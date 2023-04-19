-- NOTE(nick): this migration was originally numbered "U2130", but since "U2132" came in first and PR#2134 was a small
-- test refactoring PR of mine, I decided to number this migration as "U2134".

-- Add the root prop column to the schema variants table.
ALTER TABLE schema_variants ADD COLUMN root_prop_id ident;

-- Populate the new column.
UPDATE schema_variants
    SET root_prop_id = prop_many_to_many_schema_variants.left_object_id
    FROM prop_many_to_many_schema_variants
WHERE  schema_variants.id = prop_many_to_many_schema_variants.right_object_id;

-- Re-create the function. We can do this because the signature did not change.
CREATE OR REPLACE FUNCTION find_schema_variant_id_for_prop_v1(
    this_tenancy jsonb,
    this_visibility jsonb,
    prop_id ident,
    OUT schema_variant_id ident) AS
$$
DECLARE
    root_prop_id ident;
BEGIN
    SELECT find_root_prop_id_v1(
        this_tenancy,
        this_visibility,
        prop_id
    )
    INTO STRICT root_prop_id;

    SELECT id
    INTO STRICT schema_variant_id
    FROM schema_variants_v1($1, $2) as schema_variants
    WHERE schema_variants.root_prop_id = root_prop_id;
END;
$$ LANGUAGE PLPGSQL VOLATILE;

-- Re-create the function. We can do this because the signature did not change.
CREATE OR REPLACE FUNCTION attribute_value_id_for_prop_and_context_v1(
    this_tenancy jsonb,
    this_visibility jsonb,
    this_context jsonb,
    this_prop_id ident
)
    RETURNS ident
    LANGUAGE SQL
    STABLE
    PARALLEL SAFE
AS
$$
SELECT DISTINCT ON (
    av.attribute_context_prop_id,
    COALESCE(avbtav.belongs_to_id, ident_nil_v1()),
    COALESCE(av.key, '')
    ) av.id
FROM attribute_values_v1(this_tenancy, this_visibility) AS av
         LEFT JOIN attribute_value_belongs_to_attribute_value_v1(this_tenancy, this_visibility) AS avbtav
                   ON av.id = avbtav.object_id
         INNER JOIN schema_variants_v1(this_tenancy, this_visibility) AS schema_variants
                    ON av.attribute_context_prop_id = schema_variants.root_prop_id
WHERE in_attribute_context_v1(this_context, av)
  -- NOTE(nick): why is this named "this_prop_id"?
  AND schema_variants.id = this_prop_id
ORDER BY av.attribute_context_prop_id,
         COALESCE(avbtav.belongs_to_id, ident_nil_v1()),
         COALESCE(av.key, ''),
         av.visibility_change_set_pk DESC,
         av.visibility_deleted_at DESC NULLS FIRST,
         av.attribute_context_internal_provider_id DESC,
         av.attribute_context_external_provider_id DESC,
         av.attribute_context_component_id DESC
$$;

-- Create a new view that doesn't use the old table.
CREATE OR REPLACE VIEW components_with_attributes_v2 AS
SELECT components.id                                     AS component_id,
       component_belongs_to_schema.belongs_to_id         AS schema_id,
       schemas.name                                      AS schema_name,
       component_belongs_to_schema_variant.belongs_to_id AS schema_variant_id,
       schema_variants.name                              AS schema_variant_name,
       schema_variants.root_prop_id                      AS root_prop_id,
       internal_providers.id                             AS internal_provider_id,
       attribute_values.id                               AS attribute_value_id,
       func_binding_return_values.tenancy_workspace_pk,
       func_binding_return_values.visibility_change_set_pk,
       func_binding_return_values.visibility_deleted_at,
       func_binding_return_values.id                     AS func_binding_return_value_id,
       func_binding_return_values.value                  AS prop_values
FROM components
         LEFT JOIN component_belongs_to_schema ON component_belongs_to_schema.object_id = components.id
         LEFT JOIN component_belongs_to_schema_variant ON component_belongs_to_schema_variant.object_id = components.id
         LEFT JOIN schemas ON schemas.id = component_belongs_to_schema.belongs_to_id
         LEFT JOIN schema_variants ON schema_variants.id = component_belongs_to_schema_variant.belongs_to_id
         LEFT JOIN internal_providers ON internal_providers.prop_id = schema_variants.root_prop_id
         LEFT JOIN attribute_values
                   ON attribute_values.attribute_context_internal_provider_id = internal_providers.id AND
                      attribute_values.attribute_context_component_id = components.id
         LEFT JOIN func_binding_return_values
                   ON func_binding_return_values.id = attribute_values.func_binding_return_value_id;

-- NOTE(nick,fletcher): we can perform the more "destructive" actions below because at the time
-- that this migration was written, users can only run one sdf at a time and must restart the
-- entire stack. In the future, we will want multiple migrations to ensure this works.

-- Drop and delete all dependencies on "components_with_attributes".
DROP FUNCTION in_tenancy_and_visible_v1(jsonb,jsonb,components_with_attributes);
DROP FUNCTION in_tenancy_v1(jsonb,components_with_attributes);
DROP FUNCTION is_visible_v1(jsonb,components_with_attributes);

-- Now we can drop "components_with_attributes".
DROP VIEW components_with_attributes;

-- Drop and delete all dependencies on "prop_many_to_many_schema_variants".
DROP FUNCTION in_tenancy_and_visible_v1(jsonb,jsonb,prop_many_to_many_schema_variants);
DROP FUNCTION in_tenancy_v1(jsonb,prop_many_to_many_schema_variants);
DROP FUNCTION is_visible_v1(jsonb,prop_many_to_many_schema_variants);
DROP FUNCTION prop_many_to_many_schema_variants_v1(jsonb,jsonb);
DELETE FROM standard_models WHERE standard_models.table_name = 'prop_many_to_many_schema_variants';

-- Now we can drop "prop_many_to_many_schema_variants".
DROP TABLE prop_many_to_many_schema_variants;