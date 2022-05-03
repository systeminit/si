CREATE TABLE attribute_prototype_arguments
(
    pk bigserial PRIMARY KEY,
    id bigserial NOT NULL,
    tenancy_universal bool NOT NULL,
    tenancy_billing_account_ids bigint[],
    tenancy_organization_ids bigint[],
    tenancy_workspace_ids bigint[],
    visibility_change_set_pk bigint,
    visibility_edit_session_pk bigint,
    visibility_deleted bool,
    created_at timestamp with time zone NOT NULL DEFAULT NOW(),
    updated_at timestamp with time zone NOT NULL DEFAULT NOW(),
    name text NOT NULL,
    internal_provider_id bigint NOT NULL
);
SELECT standard_model_table_constraints_v1('attribute_prototype_arguments');
SELECT belongs_to_table_create_v1('attribute_prototype_argument_belongs_to_attribute_prototype', 'attribute_prototype_arguments', 'attribute_prototypes');

INSERT INTO standard_models (table_name, table_type, history_event_label_base, history_event_message_name)
    VALUES ('attribute_prototype_arguments', 'model', 'attribute_prototype_argument', 'Attribute Prototype Argument'),
           ('attribute_prototype_argument_belongs_to_attribute_prototype', 'belongs_to', 'attribute_prototype_argument.attribute_prototype', 'Attribute Prototype Argument <> Attribute Prototype');

CREATE OR REPLACE FUNCTION attribute_prototype_argument_create_v1(
    this_tenancy jsonb,
    this_visibility jsonb,
    this_name text,
    this_internal_provider_id bigint,
    OUT object json) AS
$$
DECLARE
    this_tenancy_record           tenancy_record_v1;
    this_visibility_record        visibility_record_v1;
    this_new_row                  attribute_prototype_arguments%ROWTYPE;
BEGIN
    this_tenancy_record := tenancy_json_to_columns_v1(this_tenancy);
    this_visibility_record := visibility_json_to_columns_v1(this_visibility);

    INSERT INTO attribute_prototype_arguments (tenancy_universal,
                               tenancy_billing_account_ids,
                               tenancy_organization_ids,
                               tenancy_workspace_ids,
                               visibility_change_set_pk,
                               visibility_edit_session_pk,
                               visibility_deleted,
                               name,
                               internal_provider_id)
    VALUES (this_tenancy_record.tenancy_universal,
            this_tenancy_record.tenancy_billing_account_ids,
            this_tenancy_record.tenancy_organization_ids,
            this_tenancy_record.tenancy_workspace_ids,
            this_visibility_record.visibility_change_set_pk,
            this_visibility_record.visibility_edit_session_pk,
            this_visibility_record.visibility_deleted,
            this_name,
            this_internal_provider_id)
    RETURNING * INTO this_new_row;

    object := row_to_json(this_new_row);
END;
$$ LANGUAGE PLPGSQL VOLATILE;
