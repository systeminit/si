CREATE TABLE change_sets
(
    pk                          ident primary key default ident_create_v1(),
    name                        text                     NOT NULL,
    note                        text,
    status                      text                     NOT NULL,
    tenancy_workspace_pk        ident,
    created_at                  timestamp with time zone NOT NULL DEFAULT CLOCK_TIMESTAMP(),
    updated_at                  timestamp with time zone NOT NULL DEFAULT CLOCK_TIMESTAMP()
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
    INSERT INTO change_sets (name, note, status, tenancy_workspace_pk)
    VALUES (this_name, this_note, this_status, this_tenancy_record.tenancy_workspace_pk)
    RETURNING * INTO this_new_row;
    object := row_to_json(this_new_row);
END;
$$ LANGUAGE PLPGSQL VOLATILE;

CREATE TYPE change_set_update_type_v1 as
(
    pk                          ident,
    id                          ident,
    tenancy_workspace_pk        ident
);

CREATE OR REPLACE FUNCTION change_set_apply_v1(this_change_set_pk ident,
                                               this_actor jsonb,
                                               this_tenancy jsonb,
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
        updated_at = clock_timestamp()
    WHERE pk = this_change_set_pk
    RETURNING updated_at INTO timestamp_updated_at;

    FOR standard_model IN SELECT * FROM standard_models
        LOOP
            this_table_name := standard_model.table_name::regclass;

            SELECT string_agg(information_schema.columns.column_name::text, ',')
            FROM information_schema.columns
            WHERE information_schema.columns.table_name = standard_model.table_name
              AND information_schema.columns.column_name NOT IN
                  ('visibility_change_set_pk', 'pk', 'created_at', 'updated_at')
              AND information_schema.columns.is_generated = 'NEVER'
            INTO insert_column_names;

            SELECT string_agg(information_schema.columns.column_name::text || ' = EXCLUDED.' ||
                              information_schema.columns.column_name::text, ', ')
            FROM information_schema.columns
            WHERE information_schema.columns.table_name = standard_model.table_name
              AND information_schema.columns.column_name NOT IN
                  ('pk', 'id', 'tenancy_workspace_pk', 'visibility_change_set_pk', 'created_at', 'updated_at')
              AND information_schema.columns.is_generated = 'NEVER'
            INTO update_set_names;

            -- Ok, this looks neat, huh? What's going on?
            --
            -- If we've deleted something in a change set, then we want those
            -- rows to conflict in head when we try to insert into head in the
            -- next query below (i.e. we're looking to trigger the ON CONFLICT
            -- behavior).
            --
            -- This will likely not do the correct thing if we have a deleted
            -- and a not-deleted version of a record in a changeset
            EXECUTE format('UPDATE %1$I ' ||
                           '  SET visibility_deleted_at = clock_timestamp(), updated_at = clock_timestamp() ' ||
                           'WHERE visibility_change_set_pk = ident_nil_v1() ' ||
                           '  AND visibility_deleted_at IS NULL ' ||
                           '  AND in_tenancy_v1(%3$L, tenancy_workspace_pk) ' ||
                           '  AND id IN ( ' ||
                           '      SELECT id ' ||
                           '      FROM %1$I ' ||
                           '      WHERE visibility_change_set_pk = %2$L ' ||
                           '        AND in_tenancy_v1(%3$L, tenancy_workspace_pk) ' ||
                           '        AND visibility_deleted_at IS NOT NULL ' ||
                           '  )', this_table_name, this_change_set_pk, this_tenancy);

            query := format('INSERT INTO %1$I (%2$s) ' ||
                            'SELECT %2$s FROM %1$I WHERE %1$I.visibility_change_set_pk = %3$L ' ||
                            '                            AND in_tenancy_v1(%5$L, tenancy_workspace_pk) ' ||
                            'ON CONFLICT (id, ' ||
                            '              tenancy_workspace_pk, ' ||
                            '              visibility_change_set_pk) ' ||
                            'DO UPDATE SET updated_at = clock_timestamp(), %4$s ' ||
                            'RETURNING pk, id, tenancy_workspace_pk',
                            this_table_name, insert_column_names, this_change_set_pk, update_set_names, this_tenancy);

            FOR updated_model IN EXECUTE query
                LOOP
                    PERFORM history_event_create_v1(standard_model.history_event_label_base || '.change_set.apply',
                                                    this_actor,
                                                    standard_model.history_event_message_name ||
                                                    ' update applied by change set',
                                                    jsonb_build_object(
                                                            'pk', updated_model.pk,
                                                            'id', updated_model.id,
                                                            'change_set_pk', this_change_set_pk
                                                        ),
                                                    jsonb_build_object('tenancy_workspace_pk', updated_model.tenancy_workspace_pk)
                        );
                END LOOP;
        END LOOP;
END;
$$ LANGUAGE PLPGSQL VOLATILE;

