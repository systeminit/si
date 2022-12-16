CREATE TABLE status_updates
(
    pk                          ident primary key default ident_create_v1(),
    tenancy_universal           bool                     NOT NULL,
    tenancy_billing_account_ids ident[],
    tenancy_organization_ids    ident[],
    tenancy_workspace_ids       ident[],
    created_at                  timestamp with time zone NOT NULL DEFAULT NOW(),
    updated_at                  timestamp with time zone NOT NULL DEFAULT NOW(),
    finished_at                 timestamp with time zone,
    change_set_pk               ident                    NOT NULL DEFAULT -1,
    data                        jsonb                    NOT NULL
);

CREATE OR REPLACE FUNCTION status_update_create_v1(this_attribute_value_id ident,
                                                   this_change_set_pk ident,
                                                   this_actor jsonb,
                                                   this_tenancy jsonb,
                                                   OUT object json) AS
$$
DECLARE
    this_tenancy_record    tenancy_record_v1;
    this_data              jsonb;
    this_new_row           status_updates%ROWTYPE;
BEGIN
    this_tenancy_record := tenancy_json_to_columns_v1(this_tenancy);

    this_data := jsonb_build_object('actor', this_actor,
                                    'attribute_value_id', this_attribute_value_id,
                                    'dependent_values_metadata', '{}'::jsonb,
                                    'queued_dependent_value_ids', '[]'::jsonb,
                                    'running_dependent_value_ids', '[]'::jsonb,
                                    'completed_dependent_value_ids', '[]'::jsonb);

    INSERT INTO status_updates (tenancy_universal, tenancy_billing_account_ids, tenancy_organization_ids,
                                tenancy_workspace_ids, change_set_pk, data)
    VALUES (this_tenancy_record.tenancy_universal, this_tenancy_record.tenancy_billing_account_ids,
            this_tenancy_record.tenancy_organization_ids, this_tenancy_record.tenancy_workspace_ids,
            this_change_set_pk, this_data)
    RETURNING * INTO this_new_row;

    object := row_to_json(this_new_row);
END;
$$ LANGUAGE PLPGSQL VOLATILE;
