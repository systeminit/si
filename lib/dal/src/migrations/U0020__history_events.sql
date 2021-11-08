CREATE TABLE history_events
(
    pk                          bigserial PRIMARY KEY,
    label                       text                     NOT NULL,
    actor                       jsonb                    NOT NULL,
    message                     text                     NOT NULL,
    data                        jsonb                    NOT NULL,
    tenancy_universal           bool,
    tenancy_billing_account_ids bigint[],
    tenancy_organization_ids    bigint[],
    tenancy_workspace_ids       bigint[],
    created_at                  timestamp with time zone NOT NULL DEFAULT NOW(),
    updated_at                  timestamp with time zone NOT NULL DEFAULT NOW()
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
    RAISE LOG 'PROFILING history_event_create_v1 PRE TENANCY SELECT: %', clock_timestamp();
    SELECT * FROM tenancy_json_to_columns_v1(this_tenancy) INTO this_tenancy_record;
    RAISE LOG 'PROFILING history_event_create_v1 POST TENANCY SELECT: %', clock_timestamp();
    RAISE LOG 'PROFILING history_event_create_v1 PRE INSERT: %', clock_timestamp();
    INSERT INTO history_events (label, actor, message, data, tenancy_universal, tenancy_billing_account_ids,
                                tenancy_organization_ids, tenancy_workspace_ids)
    VALUES (this_label, this_actor, this_message, this_data,
            this_tenancy_record.tenancy_universal, this_tenancy_record.tenancy_billing_account_ids,
            this_tenancy_record.tenancy_organization_ids, this_tenancy_record.tenancy_workspace_ids)
    RETURNING * INTO this_new_row;
    RAISE LOG 'PROFILING history_event_create_v1 POST INSERT: %', clock_timestamp();
    RAISE LOG 'PROFILING history_event_create_v1 PRE ROW TO JSON: %', clock_timestamp();
    object := row_to_json(this_new_row);
    RAISE LOG 'PROFILING history_event_create_v1 POST ROW TO JSON: %', clock_timestamp();
END;
$$ LANGUAGE PLPGSQL VOLATILE;
