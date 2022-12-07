CREATE TABLE billing_accounts
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
    name                        text                     NOT NULL,
    description                 text
);
CREATE UNIQUE INDEX unique_billing_account_name_live ON billing_accounts (
	name,
	visibility_change_set_pk,
	(visibility_deleted_at IS NULL))
    WHERE visibility_deleted_at IS NULL;
SELECT standard_model_table_constraints_v1('billing_accounts');

INSERT INTO standard_models (table_name, table_type, history_event_label_base, history_event_message_name)
VALUES ('billing_accounts', 'model', 'billing_account', 'Billing Account');

CREATE OR REPLACE FUNCTION billing_account_create_v1(
    this_tenancy jsonb,
    this_visibility jsonb,
    this_name text,
    this_description text,
    OUT object json) AS
$$
DECLARE
    this_tenancy_record    tenancy_record_v1;
    this_visibility_record visibility_record_v1;
    this_new_row           billing_accounts%ROWTYPE;
BEGIN
    this_tenancy_record := tenancy_json_to_columns_v1(this_tenancy);
    this_visibility_record := visibility_json_to_columns_v1(this_visibility);

    INSERT INTO billing_accounts (name, description, tenancy_universal, tenancy_billing_account_ids,
                                  tenancy_organization_ids, tenancy_workspace_ids, visibility_change_set_pk,
                                  visibility_deleted_at)
    VALUES (this_name, this_description,
            this_tenancy_record.tenancy_universal, this_tenancy_record.tenancy_billing_account_ids,
            this_tenancy_record.tenancy_organization_ids, this_tenancy_record.tenancy_workspace_ids,
            this_visibility_record.visibility_change_set_pk, this_visibility_record.visibility_deleted_at)
    RETURNING * INTO this_new_row;
    object := row_to_json(this_new_row);
END;
$$ LANGUAGE PLPGSQL VOLATILE;
