CREATE TABLE external_providers
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
    schema_id                   ident                   NOT NULL,
    schema_variant_id           ident                   NOT NULL,
    attribute_prototype_id      ident,
    name                        text                     NOT NULL,
    type_definition             text
);

CREATE UNIQUE INDEX unique_external_providers
    ON external_providers (name,
                           schema_id,
                           schema_variant_id,
                           tenancy_universal,
                           tenancy_billing_account_ids,
                           tenancy_organization_ids,
                           tenancy_workspace_ids,
                           visibility_change_set_pk,
                           (visibility_deleted_at IS NULL))
    WHERE visibility_deleted_at IS NULL;

CREATE INDEX ON external_providers (schema_id);
CREATE INDEX ON external_providers (schema_variant_id);
CREATE INDEX ON external_providers (attribute_prototype_id);

SELECT standard_model_table_constraints_v1('external_providers');
SELECT belongs_to_table_create_v1('socket_belongs_to_external_provider', 'sockets', 'external_providers');

INSERT INTO standard_models (table_name, table_type, history_event_label_base, history_event_message_name)
VALUES ('external_providers', 'model', 'external_provider', 'Output Provider'),
       ('socket_belongs_to_external_provider', 'belongs_to', 'socket.external_provider', 'Socket <> External Provider');

CREATE OR REPLACE FUNCTION external_provider_create_v1(
    this_tenancy jsonb,
    this_visibility jsonb,
    this_schema_id ident,
    this_schema_variant_id ident,
    this_name text,
    this_type_definition text,
    OUT object json) AS
$$
DECLARE
    this_tenancy_record    tenancy_record_v1;
    this_visibility_record visibility_record_v1;
    this_new_row           external_providers%ROWTYPE;
BEGIN
    this_tenancy_record := tenancy_json_to_columns_v1(this_tenancy);
    this_visibility_record := visibility_json_to_columns_v1(this_visibility);

    INSERT INTO external_providers (tenancy_universal,
                                    tenancy_billing_account_ids,
                                    tenancy_organization_ids,
                                    tenancy_workspace_ids,
                                    visibility_change_set_pk,
                                    visibility_deleted_at,
                                    schema_id,
                                    schema_variant_id,
                                    name,
                                    type_definition)
    VALUES (this_tenancy_record.tenancy_universal,
            this_tenancy_record.tenancy_billing_account_ids,
            this_tenancy_record.tenancy_organization_ids,
            this_tenancy_record.tenancy_workspace_ids,
            this_visibility_record.visibility_change_set_pk,
            this_visibility_record.visibility_deleted_at,
            this_schema_id,
            this_schema_variant_id,
            this_name,
            this_type_definition)
    RETURNING * INTO this_new_row;

    object := row_to_json(this_new_row);
END;
$$ LANGUAGE PLPGSQL VOLATILE;
