CREATE TABLE key_pairs
(
    pk                          ident primary key default ident_create_v1(),
    id                          ident not null default ident_create_v1(),
    tenancy_billing_account_pks ident[],
    tenancy_organization_ids    ident[],
    tenancy_workspace_ids       ident[],
    visibility_change_set_pk    ident                   NOT NULL DEFAULT ident_nil_v1(),
    visibility_deleted_at       timestamp with time zone,
    created_at                  timestamp with time zone NOT NULL DEFAULT CLOCK_TIMESTAMP(),
    updated_at                  timestamp with time zone NOT NULL DEFAULT CLOCK_TIMESTAMP(),
    name                        text                     NOT NULL,
    billing_account_pk          ident                    NOT NULL,
    created_lamport_clock       bigserial                NOT NULL,
    public_key                  text                     NOT NULL,
    secret_key                  text                     NOT NULL
);
SELECT standard_model_table_constraints_v1('key_pairs');

INSERT INTO standard_models (table_name, table_type, history_event_label_base, history_event_message_name)

VALUES ('key_pairs', 'model', 'key_pair', 'Key Pair');

CREATE OR REPLACE FUNCTION key_pair_create_v1(
    this_tenancy jsonb,
    this_visibility jsonb,
    this_name text,
    this_billing_account_pk ident,
    this_public_key text,
    this_secret_key text,
    OUT object json) AS
$$
DECLARE
    this_tenancy_record    tenancy_record_v1;
    this_visibility_record visibility_record_v1;
    this_new_row           key_pairs%ROWTYPE;
BEGIN
    this_tenancy_record := tenancy_json_to_columns_v1(this_tenancy);
    this_visibility_record := visibility_json_to_columns_v1(this_visibility);

    INSERT INTO key_pairs (tenancy_billing_account_pks, tenancy_organization_ids,
                           tenancy_workspace_ids, visibility_change_set_pk,
                           visibility_deleted_at, name, billing_account_pk, public_key, secret_key)
    VALUES (this_tenancy_record.tenancy_billing_account_pks,
            this_tenancy_record.tenancy_organization_ids, this_tenancy_record.tenancy_workspace_ids,
            this_visibility_record.visibility_change_set_pk, this_visibility_record.visibility_deleted_at,
            this_name, this_billing_account_pk, this_public_key, this_secret_key)
    RETURNING * INTO this_new_row;
    object := row_to_json(this_new_row);
END;
$$ LANGUAGE PLPGSQL VOLATILE;
