CREATE TABLE edit_sessions
(
    pk                          bigserial PRIMARY KEY,
    id                          bigserial                NOT NULL,
    name                        text                     NOT NULL,
    note                        text,
    status                      text                     NOT NULL,
    change_set_pk               bigint REFERENCES change_sets (pk),
    tenancy_universal           bool                     NOT NULL,
    tenancy_billing_account_ids bigint[],
    tenancy_organization_ids    bigint[],
    tenancy_workspace_ids       bigint[],
    created_at                  timestamp with time zone NOT NULL DEFAULT NOW(),
    updated_at                  timestamp with time zone NOT NULL DEFAULT NOW()
);

CREATE OR REPLACE FUNCTION edit_session_create_v1(this_name text,
                                                  this_note text,
                                                  this_status text,
                                                  this_change_set_pk bigint,
                                                  this_tenancy jsonb,
                                                  OUT object json) AS
$$
DECLARE
    this_tenancy_record tenancy_record_v1;
    this_new_row        edit_sessions%ROWTYPE;
BEGIN
    SELECT * FROM tenancy_json_to_columns_v1(this_tenancy) INTO this_tenancy_record;
    INSERT INTO edit_sessions (name, note, status, change_set_pk, tenancy_universal,
                               tenancy_billing_account_ids,
                               tenancy_organization_ids, tenancy_workspace_ids)
    VALUES (this_name, this_note, this_status, this_change_set_pk,
            this_tenancy_record.tenancy_universal, this_tenancy_record.tenancy_billing_account_ids,
            this_tenancy_record.tenancy_organization_ids, this_tenancy_record.tenancy_workspace_ids)
    RETURNING * INTO this_new_row;
    object := row_to_json(this_new_row);
END;
$$ LANGUAGE PLPGSQL VOLATILE;

CREATE TYPE edit_session_update_type_v1 as
(
    pk                          bigint,
    id                          bigint,
    tenancy_universal           bool,
    tenancy_billing_account_ids bigint[],
    tenancy_organization_ids    bigint[],
    tenancy_workspace_ids       bigint[]
);

CREATE OR REPLACE FUNCTION edit_session_save_v1(this_edit_session_pk bigint,
                                                this_actor jsonb,
                                                OUT timestamp_updated_at timestamp with time zone) AS
$$
DECLARE
    standard_model      standard_models%ROWTYPE;
    this_table_name     regclass;
    insert_column_names text;
    update_set_names    text;
    query               text;
    updated_model       edit_session_update_type_v1;
BEGIN
    UPDATE edit_sessions
    SET status     = 'Saved',
        updated_at = now()
    WHERE pk = this_edit_session_pk
    RETURNING updated_at INTO timestamp_updated_at;

    FOR standard_model IN SELECT * FROM standard_models
        LOOP
            this_table_name := standard_model.table_name::regclass;

            SELECT string_agg(information_schema.columns.column_name::text, ',')
            FROM information_schema.columns
            WHERE information_schema.columns.table_name = standard_model.table_name
              AND information_schema.columns.column_name NOT IN
                  ('visibility_edit_session_pk', 'pk', 'created_at', 'updated_at')
            INTO insert_column_names;

            SELECT string_agg(information_schema.columns.column_name::text || ' = EXCLUDED.' ||
                              information_schema.columns.column_name::text, ', ')
            FROM information_schema.columns
            WHERE information_schema.columns.table_name = standard_model.table_name
              AND information_schema.columns.column_name NOT IN
                  ('pk', 'id', 'tenancy_universal', 'tenancy_billing_account_ids', 'tenancy_organization_ids',
                   'tenancy_workspace_ids', 'visibility_change_set_pk', 'visibility_edit_session_pk', 'pk',
                   'created_at', 'updated_at')
            INTO update_set_names;

            query := format('INSERT INTO %1$I (%2$s) ' ||
                            'SELECT %2$s FROM %1$I WHERE %1$I.visibility_edit_session_pk = %3$L ' ||
                            'ON CONFLICT (id, visibility_change_set_pk, visibility_edit_session_pk) ' ||
                            'DO UPDATE SET updated_at = now(), %4$s ' ||
                            'RETURNING pk, id, tenancy_universal, tenancy_billing_account_ids, tenancy_organization_ids, tenancy_workspace_ids',
                            this_table_name, insert_column_names, this_edit_session_pk, update_set_names);
            RAISE WARNING '%', query;

            FOR updated_model IN EXECUTE query
                LOOP
                    PERFORM history_event_create_v1(standard_model.history_event_label_base || '.edit_session.save',
                                                    this_actor,
                                                    standard_model.history_event_message_name ||
                                                    ' saved in edit session',
                                                    jsonb_build_object(
                                                            'pk', updated_model.pk,
                                                            'id', updated_model.id,
                                                            'edit_session_pk', this_edit_session_pk
                                                        ),
                                                    jsonb_build_object(
                                                            'tenancy_universal', updated_model.tenancy_universal,
                                                            'tenancy_billing_account_ids',
                                                            updated_model.tenancy_billing_account_ids,
                                                            'tenancy_organization_ids',
                                                            updated_model.tenancy_organization_ids,
                                                            'tenancy_workspace_ids', updated_model.tenancy_workspace_ids
                                                        )
                        );
                END LOOP;
        END LOOP;
END;
$$ LANGUAGE PLPGSQL VOLATILE;

