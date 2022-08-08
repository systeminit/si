CREATE TABLE attribute_values
(
    pk                                     bigserial PRIMARY KEY,
    id                                     bigserial                NOT NULL,
    tenancy_universal                      bool                     NOT NULL,
    tenancy_billing_account_ids            bigint[],
    tenancy_organization_ids               bigint[],
    tenancy_workspace_ids                  bigint[],
    visibility_change_set_pk               bigint                   NOT NULL DEFAULT -1,
    visibility_deleted_at                  timestamp with time zone,
    attribute_context_prop_id              bigint,
    attribute_context_internal_provider_id bigint,
    attribute_context_external_provider_id bigint,
    attribute_context_schema_id            bigint,
    attribute_context_schema_variant_id    bigint,
    attribute_context_component_id         bigint,
    attribute_context_system_id            bigint,
    created_at                             timestamp with time zone NOT NULL DEFAULT NOW(),
    updated_at                             timestamp with time zone NOT NULL DEFAULT NOW(),
    proxy_for_attribute_value_id           bigint,
    sealed_proxy                           bool                     NOT NULL DEFAULT False,
    func_binding_id                        bigint                   NOT NULL,
    func_binding_return_value_id           bigint                   NOT NULL,
    index_map                              jsonb,
    key                                    text
);
SELECT standard_model_table_constraints_v1('attribute_values');
SELECT belongs_to_table_create_v1('attribute_value_belongs_to_attribute_value', 'attribute_values', 'attribute_values');
SELECT belongs_to_table_create_v1('attribute_value_belongs_to_attribute_prototype', 'attribute_values',
                                  'attribute_prototypes');

INSERT INTO standard_models (table_name, table_type, history_event_label_base, history_event_message_name)
VALUES ('attribute_values', 'model', 'attribute_value', 'Attribute Value'),
       ('attribute_value_belongs_to_attribute_value', 'belongs_to', 'attribute_value.child_attribute_value',
        'Parent Attribute Value <> Child Attribute Value'),
       ('attribute_value_belongs_to_attribute_prototype', 'belongs_to', 'attribute_prototype.attribute_value',
        'Attribute Prototype <> Attribute Value');

CREATE OR REPLACE FUNCTION attribute_value_create_v1(
    this_tenancy jsonb,
    this_visibility jsonb,
    this_attribute_context jsonb,
    this_func_binding_id bigint,
    this_func_binding_return_value_id bigint,
    this_key text,
    OUT object json) AS
$$
DECLARE
    this_tenancy_record           tenancy_record_v1;
    this_visibility_record        visibility_record_v1;
    this_attribute_context_record attribute_context_record_v1;
    this_new_row                  attribute_values%ROWTYPE;
BEGIN
    this_tenancy_record := tenancy_json_to_columns_v1(this_tenancy);
    this_visibility_record := visibility_json_to_columns_v1(this_visibility);
    this_attribute_context_record := attribute_context_json_to_columns_v1(this_attribute_context);

    INSERT INTO attribute_values (tenancy_universal,
                                  tenancy_billing_account_ids,
                                  tenancy_organization_ids,
                                  tenancy_workspace_ids,
                                  visibility_change_set_pk,
                                  visibility_deleted_at,
                                  attribute_context_prop_id,
                                  attribute_context_internal_provider_id,
                                  attribute_context_external_provider_id,
                                  attribute_context_schema_id,
                                  attribute_context_schema_variant_id,
                                  attribute_context_component_id,
                                  attribute_context_system_id,
                                  func_binding_id,
                                  func_binding_return_value_id,
                                  key)
    VALUES (this_tenancy_record.tenancy_universal,
            this_tenancy_record.tenancy_billing_account_ids,
            this_tenancy_record.tenancy_organization_ids,
            this_tenancy_record.tenancy_workspace_ids,
            this_visibility_record.visibility_change_set_pk,
            this_visibility_record.visibility_deleted_at,
            this_attribute_context_record.attribute_context_prop_id,
            this_attribute_context_record.attribute_context_internal_provider_id,
            this_attribute_context_record.attribute_context_external_provider_id,
            this_attribute_context_record.attribute_context_schema_id,
            this_attribute_context_record.attribute_context_schema_variant_id,
            this_attribute_context_record.attribute_context_component_id,
            this_attribute_context_record.attribute_context_system_id,
            this_func_binding_id,
            this_func_binding_return_value_id,
            this_key)
    RETURNING * INTO this_new_row;

    object := row_to_json(this_new_row);
END;
$$ LANGUAGE PLPGSQL VOLATILE;

CREATE OR REPLACE FUNCTION attribute_value_remove_proxies_v1(
    this_tenancy jsonb,
    this_visibility jsonb,
    this_attribute_value_id bigint,
    OUT succeeded boolean) AS
$$
DECLARE
    this_tenancy_record    tenancy_record_v1;
    this_visibility_record visibility_record_v1;
    this_found_proxy       RECORD;
BEGIN
    succeeded := True;
    this_tenancy_record := tenancy_json_to_columns_v1(this_tenancy);
    this_visibility_record := visibility_json_to_columns_v1(this_visibility);

    FOR this_found_proxy IN
        SELECT attribute_values.*
        FROM attribute_values
        WHERE proxy_for_attribute_value_id = this_attribute_value_id
        LOOP
            IF this_found_proxy.sealed_proxy = True THEN
                EXECUTE format('SELECT update_by_id_v1(%1$L, %2$L, %3$L, %4$L, %5$L, %6$L)',
                               'attribute_values', 'proxy_for_attribute_value_id',
                               this_tenancy, this_visibility, this_found_proxy.id, NULL);
                EXECUTE format('SELECT update_by_id_v1(%1$L, %2$L, %3$L, %4$L, %5$L, %6$L)',
                               'attribute_values', 'sealed_proxy',
                               this_tenancy, this_visibility, this_found_proxy.id, False);
            ELSE
                EXECUTE format('SELECT attribute_value_remove_proxies_v1(%1$L, %2$L, %3$L)',
                               this_tenancy, this_visibility, this_found_proxy.id);
                EXECUTE format('SELECT delete_by_id_v1(%1$L, %2$L, %3$L, %4$L)',
                               'attribute_values',
                               this_tenancy, this_visibility, this_found_proxy.id);
            END IF;
        END LOOP;
END;
$$ LANGUAGE PLPGSQL VOLATILE;
