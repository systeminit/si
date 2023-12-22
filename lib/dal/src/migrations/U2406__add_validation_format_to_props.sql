-- Add the new columns to the props table.
ALTER TABLE props
    ADD COLUMN validation_format text;

-- The new prop creation function automatically builds the path using the parent and sets the belongs to relationship.
CREATE OR REPLACE FUNCTION prop_create_v3(
    this_tenancy jsonb,
    this_visibility jsonb,
    this_name text,
    this_kind text,
    this_widget_kind text,
    this_widget_options jsonb,
    this_schema_variant_id ident,
    this_parent_prop_id ident,
    this_documentation text,
    this_validation_format text,
    OUT object json) AS
$$
DECLARE
    this_tenancy_record    tenancy_record_v1;
    this_visibility_record visibility_record_v1;
    this_new_row           props%ROWTYPE;
    this_path              text;
    this_parent_kind       text;
BEGIN
    this_tenancy_record := tenancy_json_to_columns_v1(this_tenancy);
    this_visibility_record := visibility_json_to_columns_v1(this_visibility);

    -- Set the path according to the lineage. If there's no parent, then we know we are the root
    -- prop. We also need to ensure that the provided parent is either an object, a map or an
    -- array.
    IF this_parent_prop_id IS NULL
    THEN
        this_path = this_name;
    ELSE
        SELECT kind, path || E'\x0B' || this_name
        INTO STRICT this_parent_kind, this_path
        FROM props_v1(this_tenancy, this_visibility) AS props
        WHERE props.id = this_parent_prop_id;

        IF this_parent_kind != 'object' AND this_parent_kind != 'array' AND this_parent_kind != 'map'
        THEN
            RAISE EXCEPTION 'prop create: provided parent is not a valid kind: %', this_parent_kind;
        END IF;
    END IF;

    -- Create and populate the row.
    INSERT INTO props (tenancy_workspace_pk,
                       visibility_change_set_pk,
                       name, kind, widget_kind, widget_options, schema_variant_id, path, documentation,
                       validation_format)
    VALUES (this_tenancy_record.tenancy_workspace_pk,
            this_visibility_record.visibility_change_set_pk,
            this_name, this_kind, this_widget_kind, this_widget_options, this_schema_variant_id, this_path,
            this_documentation, this_validation_format)
    RETURNING * INTO this_new_row;

    -- Now that we have the row, we can set the parent prop.
    IF this_parent_prop_id IS NOT NULL THEN
        PERFORM set_belongs_to_v1(
                'prop_belongs_to_prop',
                this_tenancy,
                this_visibility,
                this_new_row.id,
                this_parent_prop_id
                );
    END IF;

    object := row_to_json(this_new_row);
END;
$$ LANGUAGE PLPGSQL VOLATILE;
