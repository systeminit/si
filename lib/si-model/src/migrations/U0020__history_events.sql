CREATE TABLE history_events
(
    pk                         bigserial PRIMARY KEY,
    label                      ltree                    NOT NULL,
    actor                      jsonb                    NOT NULL,
    message                    text                     NOT NULL,
    data                       jsonb                    NOT NULL,
    tenancy_universal          bool,
    tenancy_billing_account_pks bigint[],
    tenancy_organization_pks    bigint[],
    tenancy_workspace_pks       bigint[],
    created_at                 timestamp with time zone NOT NULL DEFAULT NOW(),
    updated_at                 timestamp with time zone NOT NULL DEFAULT NOW()
);

CREATE OR REPLACE FUNCTION history_event_create_v1(this_label text,
                                                   this_actor jsonb,
                                                   this_message text,
                                                   this_data jsonb,
                                                   this_tenancy jsonb,
                                                   OUT object json) AS
$$
DECLARE
    this_tenancy_record tenancy_record_v1;
    this_new_row        history_events%ROWTYPE;
BEGIN
    SELECT * FROM tenancy_json_to_columns_v1(this_tenancy) INTO this_tenancy_record;
    RAISE WARNING 'made it';
    INSERT INTO history_events (label, actor, message, data, tenancy_universal, tenancy_billing_account_pks,
                                tenancy_organization_pks, tenancy_workspace_pks)
    VALUES (text2ltree(this_label), this_actor, this_message, this_data,
            this_tenancy_record.tenancy_universal, this_tenancy_record.tenancy_billing_account_pks,
            this_tenancy_record.tenancy_organization_pks, this_tenancy_record.tenancy_workspace_pks)
    RETURNING * INTO this_new_row;
    RAISE WARNING 'nope';
    object := row_to_json(this_new_row);
END;
$$ LANGUAGE PLPGSQL VOLATILE;
