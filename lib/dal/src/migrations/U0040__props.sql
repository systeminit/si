CREATE TABLE props
(
    pk                          ident primary key default ident_create_v1(),
    id                          ident not null default ident_create_v1(),
    tenancy_universal           bool                     NOT NULL,
    tenancy_billing_account_ids ident[],
    tenancy_organization_ids    ident[],
    tenancy_workspace_ids       ident[],
    visibility_change_set_pk    ident                   NOT NULL DEFAULT ident_nil_v1(),
    visibility_deleted_at       timestamp with time zone,
    created_at                  timestamp with time zone NOT NULL DEFAULT CLOCK_TIMESTAMP(),
    updated_at                  timestamp with time zone NOT NULL DEFAULT CLOCK_TIMESTAMP(),
    name                        text                     NOT NULL,
    kind                        text                     NOT NULL,
    index                       bigserial                NOT NULL,
    widget_kind                 text                     NOT NULL,
    widget_options              jsonb,
    doc_link                    text,
    hidden                      bool                     NOT NULL DEFAULT FALSE
);
SELECT standard_model_table_constraints_v1('props');
SELECT many_to_many_table_create_v1('prop_many_to_many_schema_variants', 'props',
                                    'schema_variants');
SELECT belongs_to_table_create_v1('prop_belongs_to_prop', 'props', 'props');

INSERT INTO standard_models (table_name, table_type, history_event_label_base, history_event_message_name)
VALUES ('props', 'model', 'prop', 'Prop'),
       ('prop_belongs_to_prop', 'belongs_to', 'prop.child_prop', 'Parent Prop <> Child Prop'),
       ('prop_many_to_many_schema_variants', 'many_to_many', 'prop.schema_variant', 'Prop <> Schema Variant');

-- Limit values of props.kind to a known set of variants. Is this required? No! But such a constraint
-- might be useful elsewhere
ALTER TABLE props
    ADD CONSTRAINT valid_kind_check CHECK (kind IN ('array', 'boolean', 'map', 'integer', 'object', 'string'));

CREATE OR REPLACE FUNCTION prop_create_v1(
    this_tenancy jsonb,
    this_visibility jsonb,
    this_name text,
    this_kind text,
    this_widget_kind text,
    this_widget_options jsonb,
    OUT object json) AS
$$
DECLARE
    this_tenancy_record    tenancy_record_v1;
    this_visibility_record visibility_record_v1;
    this_new_row           props%ROWTYPE;
BEGIN
    this_tenancy_record := tenancy_json_to_columns_v1(this_tenancy);
    this_visibility_record := visibility_json_to_columns_v1(this_visibility);

    INSERT INTO props (tenancy_universal, tenancy_billing_account_ids, tenancy_organization_ids,
                       tenancy_workspace_ids,
                       visibility_change_set_pk, visibility_deleted_at,
                       name, kind, widget_kind, widget_options)
    VALUES (this_tenancy_record.tenancy_universal, this_tenancy_record.tenancy_billing_account_ids,
            this_tenancy_record.tenancy_organization_ids, this_tenancy_record.tenancy_workspace_ids,
            this_visibility_record.visibility_change_set_pk,
            this_visibility_record.visibility_deleted_at, this_name, this_kind, this_widget_kind, this_widget_options)
    RETURNING * INTO this_new_row;

    object := row_to_json(this_new_row);
END;
$$ LANGUAGE PLPGSQL VOLATILE;

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

    SELECT right_object_id
    INTO STRICT schema_variant_id
    FROM prop_many_to_many_schema_variants_v1($1, $2) as pmtmsv
    WHERE pmtmsv.left_object_id = root_prop_id;
END;
$$ LANGUAGE PLPGSQL STABLE;

CREATE OR REPLACE FUNCTION find_root_prop_id_v1(
    this_tenancy jsonb,
    this_visibility jsonb,
    prop_id ident,
    OUT root_prop_id ident) AS
$$
DECLARE
    current_prop_id ident;
BEGIN
    current_prop_id := prop_id;
    root_prop_id := current_prop_id;
    LOOP
        SELECT belongs_to_id
        INTO current_prop_id
        FROM prop_belongs_to_prop_v1($1, $2)  
        WHERE object_id=current_prop_id;

        IF NOT FOUND THEN
            EXIT;
        END IF;

        root_prop_id := current_prop_id;
    END LOOP;
END;
$$ LANGUAGE PLPGSQL STABLE;

