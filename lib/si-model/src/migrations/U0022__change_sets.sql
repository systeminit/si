CREATE TABLE change_sets
(
    pk                          bigserial PRIMARY KEY,
    id                          bigserial                NOT NULL,
    name                        text                     NOT NULL,
    note                        text,
    status                      text                     NOT NULL,
    tenancy_universal           bool                     NOT NULL,
    tenancy_billing_account_pks bigint[],
    tenancy_organization_pks    bigint[],
    tenancy_workspace_pks       bigint[],
    created_at                  timestamp with time zone NOT NULL DEFAULT NOW(),
    updated_at                  timestamp with time zone NOT NULL DEFAULT NOW()
);

CREATE OR REPLACE FUNCTION change_set_create_v1(this_name text,
                                                this_note text,
                                                this_status text,
                                                this_tenancy jsonb,
                                                OUT object json) AS
$$
DECLARE
    this_tenancy_record tenancy_record_v1;
    this_new_row        change_sets%ROWTYPE;
BEGIN
    SELECT * FROM tenancy_json_to_columns_v1(this_tenancy) INTO this_tenancy_record;
    INSERT INTO change_sets (name, note, status, tenancy_universal, tenancy_billing_account_pks,
                             tenancy_organization_pks, tenancy_workspace_pks)
    VALUES (this_name, this_note, this_status,
            this_tenancy_record.tenancy_universal, this_tenancy_record.tenancy_billing_account_pks,
            this_tenancy_record.tenancy_organization_pks, this_tenancy_record.tenancy_workspace_pks)
    RETURNING * INTO this_new_row;
    object := row_to_json(this_new_row);
END;
$$ LANGUAGE PLPGSQL VOLATILE;

CREATE TYPE change_set_update_type_v1 as
(
    pk bigint,
    id bigint,
    tenancy_universal           bool,
    tenancy_billing_account_pks bigint[],
    tenancy_organization_pks    bigint[],
    tenancy_workspace_pks       bigint[]
);

CREATE OR REPLACE FUNCTION change_set_apply_v1(this_change_set_pk bigint,
                                                this_actor jsonb,
                                                OUT timestamp_updated_at timestamp with time zone) AS
$$
DECLARE
    standard_model      standard_models%ROWTYPE;
    this_table_name     regclass;
    insert_column_names text;
    update_set_names    text;
    query               text;
    updated_model       change_set_update_type_v1;
BEGIN
    UPDATE change_sets
    SET status     = 'Applied',
        updated_at = now()
    WHERE pk = this_change_set_pk
    RETURNING updated_at INTO timestamp_updated_at;

    FOR standard_model IN SELECT * FROM standard_models
        LOOP
            this_table_name := standard_model.table_name::regclass;

            SELECT string_agg(information_schema.columns.column_name::text, ',')
            FROM information_schema.columns
            WHERE information_schema.columns.table_name = standard_model.table_name
              AND information_schema.columns.column_name NOT IN
                  ('visibility_change_set_pk', 'visibility_edit_session_pk', 'pk', 'created_at', 'updated_at')
            INTO insert_column_names;

            SELECT string_agg(information_schema.columns.column_name::text || ' = EXCLUDED.' ||
                              information_schema.columns.column_name::text, ', ')
            FROM information_schema.columns
            WHERE information_schema.columns.table_name = standard_model.table_name
              AND information_schema.columns.column_name NOT IN
                  ('pk', 'id', 'tenancy_universal', 'tenancy_billing_account_pks', 'tenancy_organization_pks',
                   'tenancy_workspace_pks', 'visibility_change_set_pk', 'visibility_edit_session_pk', 'pk',
                   'created_at', 'updated_at')
            INTO update_set_names;

            query := format('INSERT INTO %1$I (%2$s) ' ||
                            'SELECT %2$s FROM %1$I WHERE %1$I.visibility_change_set_pk = %3$L AND %1$I.visibility_edit_session_pk = -1 ' ||
                            'ON CONFLICT (id, visibility_change_set_pk, visibility_edit_session_pk) ' ||
                            'DO UPDATE SET updated_at = now(), %4$s ' ||
                            'RETURNING pk, id, tenancy_universal, tenancy_billing_account_pks, tenancy_organization_pks, tenancy_workspace_pks',
                            this_table_name, insert_column_names, this_change_set_pk, update_set_names);
            RAISE WARNING '%', query;

            FOR updated_model IN EXECUTE query
                LOOP
                    PERFORM history_event_create_v1(standard_model.history_event_label_base || '.change_set.apply',
                                                    this_actor,
                                                    standard_model.history_event_message_name || ' update applied by change set',
                                                    jsonb_build_object(
                                                            'pk', updated_model.pk,
                                                            'id', updated_model.id,
                                                            'change_set_pk', this_change_set_pk
                                                        ),
                                                    jsonb_build_object(
                                                            'tenancy_universal', updated_model.tenancy_universal,
                                                            'tenancy_billing_account_pks', updated_model.tenancy_billing_account_pks,
                                                            'tenancy_organization_pks', updated_model.tenancy_organization_pks,
                                                            'tenancy_workspace_pks', updated_model.tenancy_workspace_pks
                                                        )
                        );
                END LOOP;
        END LOOP;
END;
$$ LANGUAGE PLPGSQL VOLATILE;

