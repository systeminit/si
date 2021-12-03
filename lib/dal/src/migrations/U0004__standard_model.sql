CREATE TABLE standard_models
(
    pk                         bigserial PRIMARY KEY,
    table_name                 text NOT NULL,
    table_type                 text NOT NULL,
    history_event_label_base   text NOT NULL,
    history_event_message_name text NOT NULL
);

ALTER TABLE standard_models
    ADD CONSTRAINT standard_models_check_valid_table_type
        CHECK (table_type IN ('model', 'belongs_to', 'many_to_many'));

CREATE OR REPLACE FUNCTION get_by_pk_v1(this_table_text text, this_pk bigint, OUT object json)
AS
$$
DECLARE
    this_table regclass;
BEGIN
    this_table := this_table_text::regclass;
    EXECUTE format('SELECT row_to_json(%1$I.*) AS object FROM %1$I WHERE %1$I.pk = %2$L', this_table,
                   this_pk) INTO object;
END ;
$$ LANGUAGE PLPGSQL STABLE;

CREATE OR REPLACE FUNCTION get_by_id_v1(this_table_text text, this_tenancy jsonb, this_visibility jsonb, this_id bigint)
    RETURNS TABLE
            (
                id                         bigint,
                visibility_change_set_pk   bigint,
                visibility_edit_session_pk bigint,
                object                     json
            )
AS
$$
DECLARE
    this_table regclass;
BEGIN
    this_table := this_table_text::regclass;
    RETURN QUERY EXECUTE format('SELECT DISTINCT ON (%1$I.id)' ||
                                '   %1$I.id,' ||
                                '   %1$I.visibility_change_set_pk,' ||
                                '   %1$I.visibility_edit_session_pk,' ||
                                '   row_to_json(%1$I.*) AS object' ||
                                ' FROM %1$I' ||
                                ' WHERE %1$I.id = %2$L' ||
                                '  AND in_tenancy_v1(%3$L, ' ||
                                '                    %1$I.tenancy_universal, ' ||
                                '                    %1$I.tenancy_billing_account_ids,' ||
                                '                    %1$I.tenancy_organization_ids,' ||
                                '                    %1$I.tenancy_workspace_ids)' ||
                                '  AND is_visible_v1(%4$L,' ||
                                '                    %1$I.visibility_change_set_pk,' ||
                                '                    %1$I.visibility_edit_session_pk,' ||
                                '                    %1$I.visibility_deleted)' ||
                                ' ORDER BY id, visibility_change_set_pk DESC, visibility_edit_session_pk DESC' ||
                                ' LIMIT 1'
        , this_table, this_id, this_tenancy, this_visibility);
END ;
$$ LANGUAGE PLPGSQL STABLE;

CREATE OR REPLACE FUNCTION update_by_id_v1(this_table_text text,
                                           this_column text,
                                           this_tenancy jsonb,
                                           this_visibility jsonb,
                                           this_id bigint,
                                           this_value text,
                                           OUT updated_at timestamp with time zone)
AS
$$
DECLARE
    this_table                   regclass;
    this_visibility_row          visibility_record_v1;
    copy_change_set_column_names text;
BEGIN
    this_table := this_table_text::regclass;
    this_visibility_row = visibility_json_to_columns_v1(this_visibility);

    /* First, try the update - if it works, we're all set. */
    EXECUTE format('UPDATE %1$I SET %2$I = %8$L, updated_at = now() WHERE id = %7$L ' ||
                   '  AND in_tenancy_v1(%3$L, ' ||
                   '                    %1$I.tenancy_universal, ' ||
                   '                    %1$I.tenancy_billing_account_ids,' ||
                   '                    %1$I.tenancy_organization_ids,' ||
                   '                    %1$I.tenancy_workspace_ids)' ||
                   '  AND %1$I.visibility_change_set_pk = %4$L ' ||
                   '  AND %1$I.visibility_edit_session_pk = %5$L ' ||
                   '  AND %1$I.visibility_deleted = %6$L' ||
                   ' RETURNING updated_at',
                   this_table, this_column, this_tenancy, this_visibility_row.visibility_change_set_pk,
                   this_visibility_row.visibility_edit_session_pk, this_visibility_row.visibility_deleted, this_id,
                   this_value) INTO updated_at;

    /* If updated_at is still null, that is because the update found no rows. We need to first copy the last known
       good data, and then update it. */
    IF updated_at IS NULL THEN
        /* Check if we are doing an update to the edit session visibility. If we are, then we need to
           copy the change set row if it exists, and if that doesn't exist, copy the head row. If
           neither exist, that is an error.

           If we aren't checking the change set and edit session, then we are pulling from head, so we
           can just copy head. */
        IF this_visibility_row.visibility_change_set_pk != '-1' AND
           this_visibility_row.visibility_edit_session_pk != '-1' THEN

            SELECT string_agg(information_schema.columns.column_name::text, ',')
            FROM information_schema.columns
            WHERE information_schema.columns.table_name = this_table
              AND information_schema.columns.column_name NOT IN
                  (this_column, 'visibility_change_set_pk', 'visibility_edit_session_pk', 'pk', 'created_at',
                   'updated_at')
            INTO copy_change_set_column_names;

            EXECUTE format('INSERT INTO %1$I (%2$s, visibility_change_set_pk, visibility_edit_session_pk, %3$s) ' ||
                           ' SELECT %4$L, %5$L, %6$L, %3$s FROM %1$I WHERE ' ||
                           ' %1$I.id = %7$L' ||
                           ' AND %1$I.visibility_change_set_pk = %5$L ' ||
                           ' AND %1$I.visibility_edit_session_pk = -1 ' ||
                           ' RETURNING updated_at',
                           this_table,
                           this_column,
                           copy_change_set_column_names,
                           this_value,
                           this_visibility_row.visibility_change_set_pk,
                           this_visibility_row.visibility_edit_session_pk,
                           this_id) INTO updated_at;
            IF updated_at IS NULL THEN
                EXECUTE format('INSERT INTO %1$I (%2$s, visibility_change_set_pk, visibility_edit_session_pk, %3$s) ' ||
                               ' SELECT %4$L, %5$L, %6$L, %3$s FROM %1$I WHERE ' ||
                               ' %1$I.id = %7$L' ||
                               ' AND %1$I.visibility_change_set_pk = -1 ' ||
                               ' AND %1$I.visibility_edit_session_pk = -1 ' ||
                               ' RETURNING updated_at',
                               this_table,
                               this_column,
                               copy_change_set_column_names,
                               this_value,
                               this_visibility_row.visibility_change_set_pk,
                               this_visibility_row.visibility_edit_session_pk,
                               this_id) INTO updated_at;
            END IF;
        ELSE
            EXECUTE format('INSERT INTO %1$I (%2$s, visibility_change_set_pk, visibility_edit_session_pk, %3$s) ' ||
                           ' SELECT %4$L, %5$L, %6$L, %3$s FROM %1$I WHERE ' ||
                           ' %1$I.id = %7$L' ||
                           ' AND %1$I.visibility_change_set_pk = -1 ' ||
                           ' AND %1$I.visibility_edit_session_pk = -1 ' ||
                           ' RETURNING updated_at',
                           this_table,
                           this_column,
                           copy_change_set_column_names,
                           this_value,
                           this_visibility_row.visibility_change_set_pk,
                           this_visibility_row.visibility_edit_session_pk,
                           this_id) INTO updated_at;
        END IF;
    END IF;
END ;
$$ LANGUAGE PLPGSQL VOLATILE;

-- update_by_id_v1 (BOOL)
CREATE OR REPLACE FUNCTION update_by_id_v1(this_table_text text,
                                           this_column text,
                                           this_tenancy jsonb,
                                           this_visibility jsonb,
                                           this_id bigint,
                                           this_value bool,
                                           OUT updated_at timestamp with time zone)
AS
$$
BEGIN
    SELECT update_by_id_v1(this_table_text,
                           this_column,
                           this_tenancy,
                           this_visibility,
                           this_id,
                           CAST(this_value as text))
    INTO updated_at;
END ;
$$ LANGUAGE PLPGSQL VOLATILE;

CREATE OR REPLACE FUNCTION delete_by_pk_v1(this_table_text text,
                                           this_tenancy jsonb,
                                           this_pk bigint,
                                           OUT updated_at timestamp with time zone)
AS
$$
DECLARE
    this_table regclass;
BEGIN
    this_table := this_table_text::regclass;
    EXECUTE format('UPDATE %1$I SET visibility_deleted = true, updated_at = now() ' ||
                   'WHERE pk = %3$L ' ||
                   '  AND in_tenancy_v1(%2$L, ' ||
                   '                    %1$I.tenancy_universal, ' ||
                   '                    %1$I.tenancy_billing_account_ids,' ||
                   '                    %1$I.tenancy_organization_ids,' ||
                   '                    %1$I.tenancy_workspace_ids)' ||
                   ' RETURNING updated_at',
                   this_table, this_tenancy, this_pk) INTO updated_at;
END;
$$ LANGUAGE PLPGSQL VOLATILE;

CREATE OR REPLACE FUNCTION undelete_by_pk_v1(this_table_text text,
                                             this_tenancy jsonb,
                                             this_pk bigint,
                                             OUT updated_at timestamp with time zone)
AS
$$
DECLARE
    this_table regclass;
BEGIN
    this_table := this_table_text::regclass;
    EXECUTE format('UPDATE %1$I SET visibility_deleted = false, updated_at = now() ' ||
                   'WHERE pk = %3$L ' ||
                   '  AND in_tenancy_v1(%2$L, ' ||
                   '                    %1$I.tenancy_universal, ' ||
                   '                    %1$I.tenancy_billing_account_ids,' ||
                   '                    %1$I.tenancy_organization_ids,' ||
                   '                    %1$I.tenancy_workspace_ids)' ||
                   ' RETURNING updated_at',
                   this_table, this_tenancy, this_pk) INTO updated_at;
END;
$$ LANGUAGE PLPGSQL VOLATILE;

CREATE OR REPLACE FUNCTION list_models_v1(this_table_text text, this_tenancy jsonb, this_visibility jsonb)
    RETURNS TABLE
            (
                id                         bigint,
                visibility_change_set_pk   bigint,
                visibility_edit_session_pk bigint,
                object                     json
            )
AS
$$
DECLARE
    this_table regclass;
BEGIN
    this_table := this_table_text::regclass;
    RETURN QUERY EXECUTE format('SELECT DISTINCT ON (%1$I.id)' ||
                                '   %1$I.id,' ||
                                '   %1$I.visibility_change_set_pk,' ||
                                '   %1$I.visibility_edit_session_pk,' ||
                                '   row_to_json(%1$I.*) AS object' ||
                                ' FROM %1$I ' ||
                                'WHERE ' ||
                                '  in_tenancy_v1(%2$L, ' ||
                                '                    %1$I.tenancy_universal, ' ||
                                '                    %1$I.tenancy_billing_account_ids,' ||
                                '                    %1$I.tenancy_organization_ids,' ||
                                '                    %1$I.tenancy_workspace_ids)' ||
                                '  AND is_visible_v1(%3$L,' ||
                                '                    %1$I.visibility_change_set_pk,' ||
                                '                    %1$I.visibility_edit_session_pk,' ||
                                '                    %1$I.visibility_deleted)' ||
                                ' ORDER BY id, visibility_change_set_pk DESC, visibility_edit_session_pk DESC'
        , this_table, this_tenancy, this_visibility);
END ;
$$ LANGUAGE PLPGSQL STABLE;

CREATE OR REPLACE FUNCTION belongs_to_v1(this_table_text text,
                                         this_tenancy jsonb,
                                         this_visibility jsonb,
                                         this_retrieve_table text,
                                         this_object_id bigint)
    RETURNS TABLE
            (
                id                         bigint,
                visibility_change_set_pk   bigint,
                visibility_edit_session_pk bigint,
                object                     json
            )
AS
$$
DECLARE
    this_table regclass;
BEGIN
    this_table := this_table_text::regclass;
    RETURN QUERY EXECUTE format('SELECT DISTINCT ON (%1$I.id)' ||
                                '   %1$I.id,' ||
                                '   %1$I.visibility_change_set_pk,' ||
                                '   %1$I.visibility_edit_session_pk,' ||
                                '   row_to_json(%5$I.*) AS object' ||
                                ' FROM %1$I' ||
                                ' INNER JOIN %5$I ON %5$I.id = %1$I.belongs_to_id ' ||
                                '  AND in_tenancy_v1(%2$L, ' ||
                                '                    %5$I.tenancy_universal, ' ||
                                '                    %5$I.tenancy_billing_account_ids,' ||
                                '                    %5$I.tenancy_organization_ids,' ||
                                '                    %5$I.tenancy_workspace_ids)' ||
                                '  AND is_visible_v1(%3$L,' ||
                                '                    %5$I.visibility_change_set_pk,' ||
                                '                    %5$I.visibility_edit_session_pk,' ||
                                '                    %5$I.visibility_deleted)' ||
                                ' WHERE %1$I.object_id = %4$L' ||
                                '  AND in_tenancy_v1(%2$L, ' ||
                                '                    %1$I.tenancy_universal, ' ||
                                '                    %1$I.tenancy_billing_account_ids,' ||
                                '                    %1$I.tenancy_organization_ids,' ||
                                '                    %1$I.tenancy_workspace_ids)' ||
                                '  AND is_visible_v1(%3$L,' ||
                                '                    %1$I.visibility_change_set_pk,' ||
                                '                    %1$I.visibility_edit_session_pk,' ||
                                '                    %1$I.visibility_deleted)' ||
                                ' ORDER BY id, visibility_change_set_pk DESC, visibility_edit_session_pk DESC' ||
                                ' LIMIT 1'
        , this_table, this_tenancy, this_visibility, this_object_id, this_retrieve_table);
END;
$$ LANGUAGE PLPGSQL STABLE;

CREATE OR REPLACE FUNCTION has_many_v1(this_table_text text,
                                       this_tenancy jsonb,
                                       this_visibility jsonb,
                                       this_retrieve_table text,
                                       this_belongs_to_id bigint)
    RETURNS TABLE
            (
                id                         bigint,
                visibility_change_set_pk   bigint,
                visibility_edit_session_pk bigint,
                object                     json
            )
AS
$$
DECLARE
    this_table regclass;
    query      text;
BEGIN
    this_table := this_table_text::regclass;
    query := format('SELECT DISTINCT ON (%1$I.id)' ||
                    '   %1$I.id,' ||
                    '   %1$I.visibility_change_set_pk,' ||
                    '   %1$I.visibility_edit_session_pk,' ||
                    '   row_to_json(%5$I.*) AS object' ||
                    ' FROM %1$I' ||
                    ' INNER JOIN %5$I ON %5$I.id = %1$I.object_id ' ||
                    '  AND in_tenancy_v1(%2$L, ' ||
                    '                    %5$I.tenancy_universal, ' ||
                    '                    %5$I.tenancy_billing_account_ids,' ||
                    '                    %5$I.tenancy_organization_ids,' ||
                    '                    %5$I.tenancy_workspace_ids)' ||
                    '  AND is_visible_v1(%3$L,' ||
                    '                    %5$I.visibility_change_set_pk,' ||
                    '                    %5$I.visibility_edit_session_pk,' ||
                    '                    %5$I.visibility_deleted)' ||
                    ' WHERE %1$I.belongs_to_id = %4$L' ||
                    '  AND in_tenancy_v1(%2$L, ' ||
                    '                    %1$I.tenancy_universal, ' ||
                    '                    %1$I.tenancy_billing_account_ids,' ||
                    '                    %1$I.tenancy_organization_ids,' ||
                    '                    %1$I.tenancy_workspace_ids)' ||
                    '  AND is_visible_v1(%3$L,' ||
                    '                    %1$I.visibility_change_set_pk,' ||
                    '                    %1$I.visibility_edit_session_pk,' ||
                    '                    %1$I.visibility_deleted)' ||
                    ' ORDER BY id, visibility_change_set_pk DESC, visibility_edit_session_pk DESC'
        , this_table, this_tenancy, this_visibility, this_belongs_to_id, this_retrieve_table);

    RAISE DEBUG 'has_many query: %', query;

    RETURN QUERY EXECUTE query;
END;
$$ LANGUAGE PLPGSQL STABLE;

CREATE OR REPLACE FUNCTION many_to_many_v1(this_table_text text,
                                           this_tenancy jsonb,
                                           this_visibility jsonb,
                                           this_left_table text,
                                           this_right_table text,
                                           this_left_object_id bigint,
                                           this_right_object_id bigint)
    RETURNS TABLE
            (
                id                         bigint,
                visibility_change_set_pk   bigint,
                visibility_edit_session_pk bigint,
                object                     json
            )
AS
$$
DECLARE
    this_table           regclass;
    query                text;
    this_return_table    text;
    this_query_object_id bigint;
    this_join_column     text;
    this_query_column    text;
BEGIN
    IF this_left_object_id IS NOT NULL THEN
        this_return_table := this_right_table;
        this_query_object_id := this_left_object_id;
        this_join_column := 'right_object_id';
        this_query_column := 'left_object_id';
    ELSIF this_right_object_id IS NOT NULL THEN
        this_return_table := this_left_table;
        this_query_object_id := this_right_object_id;
        this_join_column := 'left_object_id';
        this_query_column := 'right_object_id';
    ELSE
        RAISE EXCEPTION 'cannot retrieve many to many relationship without a left or right object id';
    END IF;

    this_table := this_table_text::regclass;
    query := format('SELECT DISTINCT ON (%1$I.id)' ||
                    '   %1$I.id,' ||
                    '   %1$I.visibility_change_set_pk,' ||
                    '   %1$I.visibility_edit_session_pk,' ||
                    '   row_to_json(%5$I.*) AS object' ||
                    ' FROM %1$I' ||
                    ' INNER JOIN %5$I ON %5$I.id = %1$I.%6$I ' ||
                    '  AND is_visible_v1(%3$L,' ||
                    '                    %5$I.visibility_change_set_pk,' ||
                    '                    %5$I.visibility_edit_session_pk,' ||
                    '                    %5$I.visibility_deleted)' ||
                    ' WHERE %1$I.%7$I = %4$L' ||
                    '  AND in_tenancy_v1(%2$L, ' ||
                    '                    %1$I.tenancy_universal, ' ||
                    '                    %1$I.tenancy_billing_account_ids,' ||
                    '                    %1$I.tenancy_organization_ids,' ||
                    '                    %1$I.tenancy_workspace_ids)' ||
                    '  AND is_visible_v1(%3$L,' ||
                    '                    %1$I.visibility_change_set_pk,' ||
                    '                    %1$I.visibility_edit_session_pk,' ||
                    '                    %1$I.visibility_deleted)' ||
                    ' ORDER BY id, visibility_change_set_pk DESC, visibility_edit_session_pk DESC'
        , this_table, this_tenancy, this_visibility, this_query_object_id, this_return_table, this_join_column,
                    this_query_column);

    RAISE DEBUG 'many_to_many query: %', query;

    RETURN QUERY EXECUTE query;
END;
$$ LANGUAGE PLPGSQL STABLE;


CREATE OR REPLACE FUNCTION check_id_in_table_v1(this_table_name text, this_id bigint, OUT result bool) AS
$$
DECLARE
    check_query text;
BEGIN
    check_query := format('SELECT true FROM %1$I WHERE id = %2$L', this_table_name, this_id);
    EXECUTE check_query INTO result;
    IF result IS NULL THEN
        result := false;
    END IF;
END;
$$ LANGUAGE plpgsql STABLE;

CREATE OR REPLACE FUNCTION belongs_to_table_create_v1(this_table_name text,
                                                      this_object_table text,
                                                      this_belongs_to_table text)
    RETURNS VOID
AS
$$
DECLARE
    create_table text;
BEGIN
    create_table := format('CREATE TABLE %1$I (' ||
                           ' pk                          bigserial PRIMARY KEY, ' ||
                           ' id                          bigserial                NOT NULL, ' ||
                           ' object_id                   bigint                   NOT NULL, ' ||
                           ' belongs_to_id               bigint                   NOT NULL, ' ||
                           ' tenancy_universal           bool                     NOT NULL, ' ||
                           ' tenancy_billing_account_ids bigint[], ' ||
                           ' tenancy_organization_ids    bigint[], ' ||
                           ' tenancy_workspace_ids       bigint[], ' ||
                           ' visibility_change_set_pk    bigint                   NOT NULL DEFAULT -1, ' ||
                           ' visibility_edit_session_pk  bigint                   NOT NULL DEFAULT -1, ' ||
                           ' visibility_deleted          bool, ' ||
                           ' created_at                  timestamp with time zone NOT NULL DEFAULT NOW(), ' ||
                           ' updated_at                  timestamp with time zone NOT NULL DEFAULT NOW() ' ||
                           '); ' ||
                           'ALTER TABLE %1$I ' ||
                           '     ADD CONSTRAINT %1$s_visibility UNIQUE (id, visibility_change_set_pk, visibility_edit_session_pk);'
                               ' ALTER TABLE %1$I '
                               '     ADD CONSTRAINT %1$s_valid_combinations CHECK (' ||
                           '             (visibility_edit_session_pk = -1 AND visibility_change_set_pk = -1) ' ||
                           '             OR ' ||
                           '             (visibility_edit_session_pk > 0 AND visibility_change_set_pk > 0) ' ||
                           '             OR ' ||
                           '             (visibility_edit_session_pk = -1 AND visibility_change_set_pk > 0) ' ||
                           '         ); ' ||
                           ' ALTER TABLE %1$I ' ||
                           '     ADD CONSTRAINT %1$s_object_id_is_valid ' ||
                           '         CHECK (check_id_in_table_v1(%2$L, object_id)); ' ||
                           ' ALTER TABLE %1$I ' ||
                           '     ADD CONSTRAINT %1$s_belongs_to_id_is_valid ' ||
                           '         CHECK (check_id_in_table_v1(%3$L, belongs_to_id));' ||
                           ' ALTER TABLE %1$I ADD CONSTRAINT %1$s_has_only_one_entry ' ||
                           '     UNIQUE (id, object_id, visibility_change_set_pk, visibility_edit_session_pk) ',
                           this_table_name,
                           this_object_table,
                           this_belongs_to_table);
    RAISE DEBUG 'create_table query: %', create_table;

    EXECUTE create_table;
END;
$$ LANGUAGE PLPGSQL VOLATILE;

CREATE OR REPLACE FUNCTION standard_model_table_constraints_v1(this_table_name text) RETURNS VOID AS
$$
DECLARE
    alter_query text;
BEGIN
    alter_query := format('ALTER TABLE %1$I ' ||
                          '        ADD CONSTRAINT %1$s_visibility UNIQUE (id, visibility_change_set_pk, visibility_edit_session_pk); ' ||
                          'ALTER TABLE %1$I ' ||
                          '    ADD CONSTRAINT %1$s_visibility_valid_combinations CHECK ( ' ||
                          '            (visibility_edit_session_pk = -1 AND visibility_change_set_pk = -1) ' ||
                          '            OR ' ||
                          '            (visibility_edit_session_pk > 0 AND visibility_change_set_pk > 0) ' ||
                          '            OR ' ||
                          '            (visibility_edit_session_pk = -1 AND visibility_change_set_pk > 0) ' ||
                          '        ); ',
                          this_table_name
        );
    RAISE DEBUG 'alter table query: %', alter_query;
    EXECUTE alter_query;
END;
$$ LANGUAGE plpgsql VOLATILE;

CREATE OR REPLACE FUNCTION check_for_duplicate_association_v1(this_table_name text,
                                                              this_left_object_id bigint,
                                                              this_right_object_id bigint,
                                                              this_visibility_change_set_pk bigint,
                                                              this_visibility_edit_session_pk bigint,
                                                              this_visibility_deleted bool,
                                                              OUT result bool) AS
$$
DECLARE
    check_query text;
BEGIN
    check_query := format('SELECT false FROM %1$I ' ||
                          '  WHERE left_object_id = %2$L ' ||
                          '    AND right_object_id = %3$L ' ||
                          '    AND (visibility_change_set_pk = %4$L OR visibility_change_set_pk = -1)' ||
                          '    AND (visibility_edit_session_pk = %5$L OR visibility_edit_session_pk = -1)' ||
                          '    AND visibility_deleted = %6$L ',
                          this_table_name,
                          this_left_object_id,
                          this_right_object_id,
                          this_visibility_change_set_pk,
                          this_visibility_edit_session_pk,
                          this_visibility_deleted);
    EXECUTE check_query INTO result;
    IF result IS NULL THEN
        result := true;
    END IF;
END;
$$ LANGUAGE plpgsql STABLE;

CREATE OR REPLACE FUNCTION many_to_many_table_create_v1(this_table_name text,
                                                        this_left_object_table text,
                                                        this_right_object_table text)
    RETURNS VOID
AS
$$
DECLARE
    create_table text;
BEGIN
    create_table := format('CREATE TABLE %1$I (' ||
                           ' pk                          bigserial PRIMARY KEY, ' ||
                           ' id                          bigserial                NOT NULL, ' ||
                           ' left_object_id              bigint                   NOT NULL, ' ||
                           ' right_object_id             bigint                   NOT NULL, ' ||
                           ' tenancy_universal           bool                     NOT NULL, ' ||
                           ' tenancy_billing_account_ids bigint[], ' ||
                           ' tenancy_organization_ids    bigint[], ' ||
                           ' tenancy_workspace_ids       bigint[], ' ||
                           ' visibility_change_set_pk    bigint                   NOT NULL DEFAULT -1, ' ||
                           ' visibility_edit_session_pk  bigint                   NOT NULL DEFAULT -1, ' ||
                           ' visibility_deleted          bool, ' ||
                           ' created_at                  timestamp with time zone NOT NULL DEFAULT NOW(), ' ||
                           ' updated_at                  timestamp with time zone NOT NULL DEFAULT NOW() ' ||
                           '); ' ||
                           'ALTER TABLE %1$I ' ||
                           '     ADD CONSTRAINT %1$s_visibility UNIQUE (id, visibility_change_set_pk, visibility_edit_session_pk);'
                               ' ALTER TABLE %1$I '
                               '     ADD CONSTRAINT %1$s_valid_combinations CHECK (' ||
                           '             (visibility_edit_session_pk = -1 AND visibility_change_set_pk = -1) ' ||
                           '             OR ' ||
                           '             (visibility_edit_session_pk > 0 AND visibility_change_set_pk > 0) ' ||
                           '             OR ' ||
                           '             (visibility_edit_session_pk = -1 AND visibility_change_set_pk > 0) ' ||
                           '         ); ' ||
                           ' ALTER TABLE %1$I ' ||
                           '     ADD CONSTRAINT %1$s_left_object_id_is_valid ' ||
                           '         CHECK (check_id_in_table_v1(%2$L, left_object_id)); ' ||
                           ' ALTER TABLE %1$I ' ||
                           '     ADD CONSTRAINT %1$s_right_object_id_is_valid ' ||
                           '         CHECK (check_id_in_table_v1(%3$L, right_object_id));' ||
                           ' ALTER TABLE %1$I ADD CONSTRAINT %1$s_has_only_one_entry_per_pair ' ||
                           '     UNIQUE (id, left_object_id, right_object_id, visibility_change_set_pk, visibility_edit_session_pk); ' ||
                           ' ALTER TABLE %1$I ' ||
                           '     ADD CONSTRAINT %1$s_no_duplicate_associations ' ||
                           '         CHECK (check_for_duplicate_association_v1(%1$L, left_object_id, right_object_id, visibility_change_set_pk, visibility_edit_session_pk, visibility_deleted)); ',
                           this_table_name,
                           this_left_object_table,
                           this_right_object_table);
    RAISE DEBUG 'create_table query: %', create_table;

    EXECUTE create_table;
END;
$$ LANGUAGE PLPGSQL VOLATILE;

CREATE OR REPLACE FUNCTION associate_many_to_many_v1(this_table_name text,
                                                     this_tenancy jsonb,
                                                     this_visibility jsonb,
                                                     this_left_object_id bigint,
                                                     this_right_object_id bigint
) RETURNS VOID AS
$$
DECLARE
    insert_query           text;
    this_tenancy_record    tenancy_record_v1;
    this_visibility_record visibility_record_v1;
BEGIN
    this_tenancy_record := tenancy_json_to_columns_v1(this_tenancy);
    this_visibility_record := visibility_json_to_columns_v1(this_visibility);

    insert_query :=
            format(' INSERT INTO %1$I (left_object_id, right_object_id, tenancy_universal, ' ||
                   '                  tenancy_billing_account_ids, tenancy_organization_ids, ' ||
                   '                  tenancy_workspace_ids, visibility_change_set_pk, ' ||
                   '                  visibility_edit_session_pk, visibility_deleted) ' ||
                   ' VALUES (%2$L, ' ||
                   '         %3$L, ' ||
                   '         %4$L, ' ||
                   '         %5$L, ' ||
                   '         %6$L, ' ||
                   '         %7$L, ' ||
                   '         %8$L, ' ||
                   '         %9$L, ' ||
                   '         %10$L)' ||
                   'ON CONFLICT (id, visibility_change_set_pk, visibility_edit_session_pk)' ||
                   '   DO UPDATE SET visibility_deleted = false ',
                   this_table_name,
                   this_left_object_id,
                   this_right_object_id,
                   this_tenancy_record.tenancy_universal,
                   this_tenancy_record.tenancy_billing_account_ids,
                   this_tenancy_record.tenancy_organization_ids,
                   this_tenancy_record.tenancy_workspace_ids,
                   this_visibility_record.visibility_change_set_pk,
                   this_visibility_record.visibility_edit_session_pk,
                   this_visibility_record.visibility_deleted
                );
    RAISE DEBUG 'associate many to many: %', insert_query;
    EXECUTE insert_query;
END;
$$ LANGUAGE plpgsql VOLATILE;

CREATE OR REPLACE FUNCTION disassociate_many_to_many_v1(this_table_name text,
                                                        this_tenancy jsonb,
                                                        this_visibility jsonb,
                                                        this_left_object_id bigint,
                                                        this_right_object_id bigint
) RETURNS VOID AS
$$
DECLARE
    update_query text;
BEGIN

    update_query :=
            format('UPDATE %1$I SET visibility_deleted = true ' ||
                   '  WHERE left_object_id = %2$L ' ||
                   '    AND right_object_id = %3$L ' ||
                   '    AND in_tenancy_v1(%4$L, ' ||
                   '                    %1$I.tenancy_universal, ' ||
                   '                    %1$I.tenancy_billing_account_ids,' ||
                   '                    %1$I.tenancy_organization_ids,' ||
                   '                    %1$I.tenancy_workspace_ids)' ||
                   '    AND is_visible_v1(%5$L,' ||
                   '                    %1$I.visibility_change_set_pk,' ||
                   '                    %1$I.visibility_edit_session_pk,' ||
                   '                    %1$I.visibility_deleted)',
                   this_table_name,
                   this_left_object_id,
                   this_right_object_id,
                   this_tenancy,
                   this_visibility
                );
    RAISE DEBUG 'disassociate many to many: %', update_query;
    EXECUTE update_query;
END;
$$ LANGUAGE plpgsql VOLATILE;

CREATE OR REPLACE FUNCTION set_belongs_to_v1(this_table_name text,
                                             this_tenancy jsonb,
                                             this_visibility jsonb,
                                             this_object_id bigint,
                                             this_belongs_to_id bigint
) RETURNS VOID AS
$$
DECLARE
    insert_query           text;
    this_tenancy_record    tenancy_record_v1;
    this_visibility_record visibility_record_v1;
BEGIN
    this_tenancy_record := tenancy_json_to_columns_v1(this_tenancy);
    this_visibility_record := visibility_json_to_columns_v1(this_visibility);

    insert_query :=
            format(' INSERT INTO %1$I (object_id, belongs_to_id, tenancy_universal, ' ||
                   '                  tenancy_billing_account_ids, tenancy_organization_ids, ' ||
                   '                  tenancy_workspace_ids, visibility_change_set_pk, ' ||
                   '                  visibility_edit_session_pk, visibility_deleted) ' ||
                   ' VALUES (%2$L, ' ||
                   '         %3$L, ' ||
                   '         %4$L, ' ||
                   '         %5$L, ' ||
                   '         %6$L, ' ||
                   '         %7$L, ' ||
                   '         %8$L, ' ||
                   '         %9$L, ' ||
                   '         %10$L)' ||
                   'ON CONFLICT (id, visibility_change_set_pk, visibility_edit_session_pk)' ||
                   '   DO UPDATE SET visibility_deleted = false, belongs_to_id = %3$L',
                   this_table_name,
                   this_object_id,
                   this_belongs_to_id,
                   this_tenancy_record.tenancy_universal,
                   this_tenancy_record.tenancy_billing_account_ids,
                   this_tenancy_record.tenancy_organization_ids,
                   this_tenancy_record.tenancy_workspace_ids,
                   this_visibility_record.visibility_change_set_pk,
                   this_visibility_record.visibility_edit_session_pk,
                   this_visibility_record.visibility_deleted
                );
    RAISE DEBUG 'set belongs to: %', insert_query;
    EXECUTE insert_query;
END;
$$ LANGUAGE plpgsql VOLATILE;

CREATE OR REPLACE FUNCTION unset_belongs_to_v1(this_table_name text,
                                               this_tenancy jsonb,
                                               this_visibility jsonb,
                                               this_object_id bigint
) RETURNS VOID AS
$$
DECLARE
    update_query text;
BEGIN

    update_query :=
            format('UPDATE %1$I SET visibility_deleted = true ' ||
                   '  WHERE object_id = %2$L ' ||
                   '    AND in_tenancy_v1(%3$L, ' ||
                   '                    %1$I.tenancy_universal, ' ||
                   '                    %1$I.tenancy_billing_account_ids,' ||
                   '                    %1$I.tenancy_organization_ids,' ||
                   '                    %1$I.tenancy_workspace_ids)' ||
                   '    AND is_visible_v1(%4$L,' ||
                   '                    %1$I.visibility_change_set_pk,' ||
                   '                    %1$I.visibility_edit_session_pk,' ||
                   '                    %1$I.visibility_deleted)',
                   this_table_name,
                   this_object_id,
                   this_tenancy,
                   this_visibility
                );
    RAISE DEBUG 'unset belongs to: %', update_query;
    EXECUTE update_query;
END;
$$ LANGUAGE plpgsql VOLATILE;

