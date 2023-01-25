CREATE TABLE schema_variants
(
    pk                          ident primary key                 default ident_create_v1(),
    id                          ident                    not null default ident_create_v1(),
    tenancy_billing_account_pks ident[],
    tenancy_organization_pks    ident[],
    tenancy_workspace_ids       ident[],
    visibility_change_set_pk    ident                    NOT NULL DEFAULT ident_nil_v1(),
    visibility_deleted_at       timestamp with time zone,
    created_at                  timestamp with time zone NOT NULL DEFAULT CLOCK_TIMESTAMP(),
    updated_at                  timestamp with time zone NOT NULL DEFAULT CLOCK_TIMESTAMP(),
    name                        text                     NOT NULL,
    link                        text,
    color                       bigint,
    finalized_once              bool                     NOT NULL DEFAULT FALSE
);
SELECT standard_model_table_constraints_v1('schema_variants');
SELECT belongs_to_table_create_v1('schema_variant_belongs_to_schema', 'schema_variants', 'schemas');

INSERT INTO standard_models (table_name, table_type, history_event_label_base, history_event_message_name)
VALUES ('schema_variants', 'model', 'schema_variant', 'Schema Variant'),
       ('schema_variant_belongs_to_schema', 'belongs_to', 'schema_variant.schema', 'Schema Variant <> Schema');

CREATE OR REPLACE FUNCTION schema_variant_create_v1(
    this_tenancy jsonb,
    this_visibility jsonb,
    this_name text,
    OUT object json) AS
$$
DECLARE
    this_tenancy_record    tenancy_record_v1;
    this_visibility_record visibility_record_v1;
    this_new_row           schema_variants%ROWTYPE;
BEGIN
    this_tenancy_record := tenancy_json_to_columns_v1(this_tenancy);
    this_visibility_record := visibility_json_to_columns_v1(this_visibility);

    INSERT INTO schema_variants (tenancy_billing_account_pks, tenancy_organization_pks,
                                 tenancy_workspace_ids,
                                 visibility_change_set_pk, visibility_deleted_at, name)
    VALUES (this_tenancy_record.tenancy_billing_account_pks,
            this_tenancy_record.tenancy_organization_pks, this_tenancy_record.tenancy_workspace_ids,
            this_visibility_record.visibility_change_set_pk, this_visibility_record.visibility_deleted_at, this_name)
    RETURNING * INTO this_new_row;

    object := row_to_json(this_new_row);
END;
$$ LANGUAGE PLPGSQL VOLATILE;
