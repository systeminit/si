CREATE TABLE history_events
(
    pk                          ident primary key default ident_create_v1(),
    label                       text                     NOT NULL,
    actor                       jsonb                    NOT NULL,
    message                     text                     NOT NULL,
    data                        jsonb                    NOT NULL,
    tenancy_workspace_pk        ident,
    created_at                  timestamp with time zone NOT NULL DEFAULT CLOCK_TIMESTAMP(),
    updated_at                  timestamp with time zone NOT NULL DEFAULT CLOCK_TIMESTAMP()
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
    RAISE DEBUG 'PROFILING history_event_create_v1 PRE TENANCY SELECT: %', clock_timestamp();
    SELECT * FROM tenancy_json_to_columns_v1(this_tenancy) INTO this_tenancy_record;
    RAISE DEBUG 'PROFILING history_event_create_v1 POST TENANCY SELECT: %', clock_timestamp();
    RAISE DEBUG 'PROFILING history_event_create_v1 PRE INSERT: %', clock_timestamp();
    INSERT INTO history_events (label, actor, message, data, tenancy_workspace_pk)
    VALUES (this_label, this_actor, this_message, this_data, this_tenancy_record.tenancy_workspace_pk)
    RETURNING * INTO this_new_row;
    RAISE DEBUG 'PROFILING history_event_create_v1 POST INSERT: %', clock_timestamp();
    RAISE DEBUG 'PROFILING history_event_create_v1 PRE ROW TO JSON: %', clock_timestamp();
    object := row_to_json(this_new_row);
    RAISE DEBUG 'PROFILING history_event_create_v1 POST ROW TO JSON: %', clock_timestamp();
END;
$$ LANGUAGE PLPGSQL VOLATILE;
