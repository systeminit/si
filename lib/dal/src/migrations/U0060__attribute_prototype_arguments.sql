CREATE TABLE attribute_prototype_arguments
(
    pk                          ident primary key default ident_create_v1(),
    id                          ident not null default ident_create_v1(),
    tenancy_universal           bool                     NOT NULL,
    tenancy_billing_account_ids ident[],
    tenancy_organization_ids    ident[],
    tenancy_workspace_ids       ident[],
    visibility_change_set_pk    ident                   NOT NULL DEFAULT ident_nil_v1(),
    visibility_deleted_at       timestamp with time zone,
    created_at                  timestamp with time zone NOT NULL DEFAULT NOW(),
    updated_at                  timestamp with time zone NOT NULL DEFAULT NOW(),
    func_argument_id            ident                   NOT NULL,
    attribute_prototype_id      ident                   NOT NULL,
    internal_provider_id        ident                   NOT NULL,
    external_provider_id        ident                   NOT NULL,
    tail_component_id           ident                   NOT NULL,
    head_component_id           ident                   NOT NULL
);

CREATE UNIQUE INDEX intra_component_argument
    ON attribute_prototype_arguments (attribute_prototype_id,
                                      func_argument_id,
                                      internal_provider_id,
                                      tenancy_universal,
                                      tenancy_billing_account_ids,
                                      tenancy_organization_ids,
                                      tenancy_workspace_ids,
                                      visibility_change_set_pk,
                                      (visibility_deleted_at IS NULL))
    WHERE visibility_deleted_at IS NULL
        AND (external_provider_id = ident_nil_v1()
            OR head_component_id = ident_nil_v1()
            OR tail_component_id = ident_nil_v1());

CREATE UNIQUE INDEX inter_component_argument
    ON attribute_prototype_arguments (attribute_prototype_id,
                                      func_argument_id,
                                      external_provider_id,
                                      tail_component_id,
                                      head_component_id,
                                      tenancy_universal,
                                      tenancy_billing_account_ids,
                                      tenancy_organization_ids,
                                      tenancy_workspace_ids,
                                      visibility_change_set_pk,
                                      (visibility_deleted_at IS NULL))
    WHERE visibility_deleted_at IS NULL
        AND internal_provider_id = ident_nil_v1();

CREATE INDEX ON attribute_prototype_arguments (attribute_prototype_id);
CREATE INDEX ON attribute_prototype_arguments (external_provider_id);
CREATE INDEX ON attribute_prototype_arguments (head_component_id);
CREATE INDEX ON attribute_prototype_arguments (internal_provider_id);

SELECT standard_model_table_constraints_v1('attribute_prototype_arguments');
INSERT INTO standard_models (table_name, table_type, history_event_label_base, history_event_message_name)
VALUES ('attribute_prototype_arguments', 'model', 'attribute_prototype_argument', 'Attribute Prototype Argument');

CREATE OR REPLACE FUNCTION attribute_prototype_argument_create_v1(
    this_tenancy jsonb,
    this_visibility jsonb,
    this_attribute_prototype_argument_id ident,
    this_func_argument_id ident,
    this_internal_provider_id ident,
    this_external_provider_id ident,
    this_tail_component_id ident,
    this_head_component_id ident,
    OUT object json) AS
$$
DECLARE
    this_tenancy_record    tenancy_record_v1;
    this_visibility_record visibility_record_v1;
    this_new_row           attribute_prototype_arguments%ROWTYPE;
BEGIN
    this_tenancy_record := tenancy_json_to_columns_v1(this_tenancy);
    this_visibility_record := visibility_json_to_columns_v1(this_visibility);

    INSERT INTO attribute_prototype_arguments (tenancy_universal,
                                               tenancy_billing_account_ids,
                                               tenancy_organization_ids,
                                               tenancy_workspace_ids,
                                               visibility_change_set_pk,
                                               visibility_deleted_at,
                                               attribute_prototype_id,
                                               func_argument_id,
                                               internal_provider_id,
                                               external_provider_id,
                                               tail_component_id,
                                               head_component_id)
    VALUES (this_tenancy_record.tenancy_universal,
            this_tenancy_record.tenancy_billing_account_ids,
            this_tenancy_record.tenancy_organization_ids,
            this_tenancy_record.tenancy_workspace_ids,
            this_visibility_record.visibility_change_set_pk,
            this_visibility_record.visibility_deleted_at,
            this_attribute_prototype_argument_id,
            this_func_argument_id,
            this_internal_provider_id,
            this_external_provider_id,
            this_tail_component_id,
            this_head_component_id)
    RETURNING * INTO this_new_row;

    RAISE DEBUG 'attribute_prototype_argument_create_v1: Created AttributePrototypeArgument(%)', this_new_row;

    object := row_to_json(this_new_row);
END;
$$ LANGUAGE PLPGSQL VOLATILE;
