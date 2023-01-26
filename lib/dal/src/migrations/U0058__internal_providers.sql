CREATE TABLE internal_providers
(
    pk                          ident primary key default ident_create_v1(),
    id                          ident not null default ident_create_v1(),
    tenancy_workspace_pks       ident[],
    visibility_change_set_pk    ident                   NOT NULL DEFAULT ident_nil_v1(),
    visibility_deleted_at       timestamp with time zone,
    created_at                  timestamp with time zone NOT NULL DEFAULT CLOCK_TIMESTAMP(),
    updated_at                  timestamp with time zone NOT NULL DEFAULT CLOCK_TIMESTAMP(),
    prop_id                     ident                   NOT NULL,
    schema_variant_id           ident                   NOT NULL,
    attribute_prototype_id      ident,
    name                        text                     NOT NULL,
    inbound_type_definition     text,
    outbound_type_definition    text
);

CREATE UNIQUE INDEX unique_implicit_internal_providers
    ON internal_providers (prop_id,
                           schema_variant_id,
                           tenancy_workspace_pks,
                           visibility_change_set_pk,
                           (visibility_deleted_at IS NULL))
    WHERE visibility_deleted_at IS NULL
        AND NOT prop_id = ident_nil_v1();

CREATE UNIQUE INDEX unique_explicit_internal_providers
    ON internal_providers (name,
                           schema_variant_id,
                           tenancy_workspace_pks,
                           visibility_change_set_pk,
                           (visibility_deleted_at IS NULL))
    WHERE visibility_deleted_at IS NULL
        AND prop_id = ident_nil_v1();

CREATE INDEX ON internal_providers (prop_id);
CREATE INDEX ON internal_providers (schema_variant_id);
CREATE INDEX ON internal_providers (attribute_prototype_id);

SELECT standard_model_table_constraints_v1('internal_providers');
SELECT belongs_to_table_create_v1('socket_belongs_to_internal_provider', 'sockets', 'internal_providers');

INSERT INTO standard_models (table_name, table_type, history_event_label_base, history_event_message_name)
VALUES ('internal_providers', 'model', 'internal_provider', 'Input Provider'),
       ('socket_belongs_to_internal_provider', 'belongs_to', 'socket.internal_provider', 'Socket <> Internal Provider');

CREATE OR REPLACE FUNCTION internal_provider_create_v1(
    this_tenancy jsonb,
    this_visibility jsonb,
    this_prop_id ident,
    this_schema_variant_id ident,
    this_name text,
    this_inbound_type_definition text,
    this_outbound_type_definition text,
    OUT object json) AS
$$
DECLARE
    this_tenancy_record    tenancy_record_v1;
    this_visibility_record visibility_record_v1;
    this_new_row           internal_providers%ROWTYPE;
BEGIN
    this_tenancy_record := tenancy_json_to_columns_v1(this_tenancy);
    this_visibility_record := visibility_json_to_columns_v1(this_visibility);

    INSERT INTO internal_providers (tenancy_workspace_pks,
                                    visibility_change_set_pk,
                                    visibility_deleted_at,
                                    prop_id,
                                    schema_variant_id,
                                    name,
                                    inbound_type_definition,
                                    outbound_type_definition)
    VALUES (this_tenancy_record.tenancy_workspace_pks,
            this_visibility_record.visibility_change_set_pk,
            this_visibility_record.visibility_deleted_at,
            this_prop_id,
            this_schema_variant_id,
            this_name,
            this_inbound_type_definition,
            this_outbound_type_definition)
    RETURNING * INTO this_new_row;

    object := row_to_json(this_new_row);
END;
$$ LANGUAGE PLPGSQL VOLATILE;
