CREATE TABLE standard_models
(
    pk                         ident primary key default ident_create_v1(),
    table_name                 text NOT NULL,
    table_type                 text NOT NULL,
    history_event_label_base   text NOT NULL,
    history_event_message_name text NOT NULL
);

ALTER TABLE standard_models
    ADD CONSTRAINT standard_models_check_valid_table_type
        CHECK (table_type IN ('model', 'belongs_to', 'many_to_many'));

CREATE OR REPLACE FUNCTION get_by_pk_v1(this_table_text text, this_pk ident, OUT object json)
AS
$$
DECLARE
    this_table regclass;
BEGIN
    this_table := this_table_text::regclass;
    EXECUTE format('SELECT row_to_json(%1$I.*) AS object FROM %1$I WHERE %1$I.pk = %2$L', this_table, this_pk)
    INTO object;
END ;
$$ LANGUAGE PLPGSQL STABLE;

CREATE OR REPLACE FUNCTION get_by_id_v1(this_table_text text, this_tenancy jsonb, this_visibility jsonb, this_id ident)
    RETURNS TABLE
            (
                id                       ident,
                visibility_change_set_pk ident,
                visibility_deleted_at    timestamp with time zone,
                object                   json
            )
AS
$$
DECLARE
    this_table regclass;
BEGIN
    this_table := this_table_text::regclass;
    RETURN QUERY EXECUTE format('SELECT '
                                '   table_alias.id, '
                                '   table_alias.visibility_change_set_pk, '
                                '   table_alias.visibility_deleted_at, '
                                '   row_to_json(table_alias.*) AS object '
                                ' FROM %1$I_v1(%3$L, %4$L) AS table_alias '
                                ' WHERE table_alias.id = %2$L '
                                ' ORDER BY id, visibility_change_set_pk DESC, visibility_deleted_at DESC NULLS FIRST '
        , this_table, this_id, this_tenancy, this_visibility);
END ;
$$ LANGUAGE PLPGSQL STABLE;

CREATE OR REPLACE FUNCTION find_by_attr_v1(this_table_text text, this_tenancy jsonb, this_visibility jsonb,
                                           this_attr_name text, this_value text)
    RETURNS TABLE
            (
                id                       ident,
                visibility_change_set_pk ident,
                object                   json
            )
AS
$$
DECLARE
    this_table regclass;
BEGIN
    this_table := this_table_text::regclass;
    RETURN QUERY EXECUTE format('SELECT '
                                '   table_alias.id, '
                                '   table_alias.visibility_change_set_pk, '
                                '   row_to_json(table_alias.*) AS object '
                                ' FROM %1$I_v1(%2$L, %3$L) AS table_alias '
                                ' WHERE table_alias.%4$I = %5$L '
                                ' ORDER BY id, visibility_change_set_pk DESC '
        , this_table, this_tenancy, this_visibility, this_attr_name, this_value);
END ;
$$ LANGUAGE PLPGSQL STABLE;

CREATE OR REPLACE FUNCTION find_by_attr_null_v1(this_table_text text, this_tenancy jsonb, this_visibility jsonb,
                                                this_attr_name text)
    RETURNS TABLE
            (
                id                       ident,
                visibility_change_set_pk ident,
                object                   json
            )
AS
$$
DECLARE
    this_table regclass;
BEGIN
    this_table := this_table_text::regclass;
    RETURN QUERY EXECUTE format('SELECT '
                                '   table_alias.id, '
                                '   table_alias.visibility_change_set_pk, '
                                '   row_to_json(table_alias.*) AS object '
                                ' FROM %1$I_v1(%2$L, %3$L) AS table_alias '
                                ' WHERE table_alias.%4$I IS NULL '
                                ' ORDER BY id, visibility_change_set_pk DESC '
        , this_table, this_tenancy, this_visibility, this_attr_name);
END ;
$$ LANGUAGE PLPGSQL STABLE;

CREATE OR REPLACE FUNCTION find_by_attr_in_v1(this_table_text text, this_tenancy jsonb, this_visibility jsonb,
                                              this_attr_name text, this_value text[])
    RETURNS TABLE
            (
                id                       ident,
                visibility_change_set_pk ident,
                object                   json
            )
AS
$$
DECLARE
    this_table regclass;
BEGIN
    this_table := this_table_text::regclass;
    RETURN QUERY EXECUTE format('SELECT '
                                '   table_alias.id, '
                                '   table_alias.visibility_change_set_pk, '
                                '   row_to_json(table_alias.*) AS object '
                                ' FROM %1$I_v1(%2$L, %3$L) AS table_alias '
                                ' WHERE table_alias.%4$I = ANY (%5$L) '
                                ' ORDER BY id, visibility_change_set_pk DESC '
        , this_table, this_tenancy, this_visibility, this_attr_name, this_value);
END ;
$$ LANGUAGE PLPGSQL STABLE;


CREATE OR REPLACE FUNCTION find_by_attr_not_in_v1(this_table_text text, this_tenancy jsonb, this_visibility jsonb,
                                                  this_attr_name text, this_value text[])
    RETURNS TABLE
            (
                id                       ident,
                visibility_change_set_pk ident,
                object                   json
            )
AS
$$
DECLARE
    this_table regclass;
BEGIN
    this_table := this_table_text::regclass;
    RETURN QUERY EXECUTE format('SELECT '
                                '   table_alias.id, '
                                '   table_alias.visibility_change_set_pk, '
                                '   row_to_json(table_alias.*) AS object '
                                ' FROM %1$I_v1(%2$L, %3$L) AS table_alias '
                                ' WHERE NOT table_alias.%4$I = ANY (%5$L) '
                                ' ORDER BY id, visibility_change_set_pk DESC '
        , this_table, this_tenancy, this_visibility, this_attr_name, this_value);
END ;
$$ LANGUAGE PLPGSQL STABLE;


CREATE OR REPLACE FUNCTION update_by_id_v1(
    this_table         text,
    this_column        text,
    this_read_tenancy  jsonb,
    this_write_tenancy jsonb,
    this_visibility    jsonb,
    this_id            ident,
    this_value         text,
    OUT updated_at     timestamp with time zone)
AS
$$
DECLARE
    this_visibility_row          visibility_record_v1;
    this_write_tenancy_row       tenancy_record_v1;
    copy_change_set_column_names text;
    debugging_record_info        record;
BEGIN
    this_visibility_row = visibility_json_to_columns_v1(this_visibility);
    this_write_tenancy_row = tenancy_json_to_columns_v1(this_write_tenancy);

    IF this_visibility_row.visibility_deleted_at IS NOT NULL THEN
        RAISE EXCEPTION 'update_by_id_v1: cannot update column % on table % for a deleted record',
            this_column,
            this_table;
    END IF;

    /* First, try the update - if it works, we're all set. */
    EXECUTE format('UPDATE %1$I SET %2$I = %6$L, updated_at = clock_timestamp() WHERE id = %5$L '
                   '  AND in_tenancy_v1(%3$L, '
                   '                    %1$I.tenancy_billing_account_pks, '
                   '                    %1$I.tenancy_organization_pks, '
                   '                    %1$I.tenancy_workspace_pks) '
                   '  AND %1$I.visibility_change_set_pk = %4$L '
                   '  AND %1$I.visibility_deleted_at IS NULL '
                   ' RETURNING updated_at',
                   this_table,
                   this_column,
                   this_write_tenancy,
                   this_visibility_row.visibility_change_set_pk,
                   this_id,
                   this_value) INTO updated_at;

    /* If updated_at is still null, that is because the update found no rows. We need to first copy the last known
       good data, and then update it. */
    IF updated_at IS NULL THEN
        /* Check if we are doing an update to the edit session visibility. If we are, then we need to
           copy the change set row if it exists, and if that doesn't exist, copy the head row. If
           neither exist, that is an error.

           If we aren't checking the change set and edit session, then we are pulling from head, so we
           can just copy head. */
        IF this_visibility_row.visibility_change_set_pk != ident_nil_v1() THEN

            SELECT string_agg(information_schema.columns.column_name::text, ',')
            FROM information_schema.columns
            WHERE information_schema.columns.table_name = this_table
              AND information_schema.columns.column_name NOT IN
                  (
                    this_column,
                    'visibility_change_set_pk',
                    'pk',
                    'created_at',
                    'updated_at',
                    'tenancy_billing_account_pks',
                    'tenancy_organization_pks',
                    'tenancy_workspace_pks'
                  )
              AND information_schema.columns.is_generated = 'NEVER'
            INTO copy_change_set_column_names;
            EXECUTE format('INSERT INTO %1$I ( '
                           '    %2$s, '
                           '    visibility_change_set_pk, '
                           '    tenancy_billing_account_pks, '
                           '    tenancy_organization_pks, '
                           '    tenancy_workspace_pks, '
                           '    %3$s) '
                           ' SELECT %4$L, %5$L, %8$L, %9$L, %10$L, %3$s FROM %1$I WHERE '
                           ' %1$I.id = %6$L '
                           ' AND in_tenancy_v1(%7$L, '
                           '                   %1$I.tenancy_billing_account_pks, '
                           '                   %1$I.tenancy_organization_pks, '
                           '                   %1$I.tenancy_workspace_pks) '
                           ' AND %1$I.visibility_change_set_pk = ident_nil_v1() '
                           ' AND %1$I.visibility_deleted_at IS NULL '
                           ' RETURNING updated_at',
                           this_table,
                           this_column,
                           copy_change_set_column_names,
                           this_value,
                           this_visibility_row.visibility_change_set_pk,
                           this_id,
                           this_read_tenancy,
                           this_write_tenancy_row.tenancy_billing_account_pks,
                           this_write_tenancy_row.tenancy_organization_pks,
                           this_write_tenancy_row.tenancy_workspace_pks
                        ) INTO updated_at;
        END IF;

        -- If updated_at is still null, then there is a provided tenancy
        -- that is not suitable--this could be an application bug!
        IF updated_at IS NULL THEN
            EXECUTE format('SELECT * FROM %1$I WHERE id = %2$L', this_table, this_id)
                INTO debugging_record_info;
            RAISE EXCEPTION
                'update_by_id_v1: cannot update column % on table % of record % to value % (likely a tenancy issue). Tenancy(%), Visibility(%), %(%)',
                this_column,
                this_table,
                this_id,
                this_value,
                this_write_tenancy,
                this_visibility,
                this_table,
                debugging_record_info;

        END IF;
    END IF;
END ;
$$ LANGUAGE PLPGSQL VOLATILE;

-- update_by_id_v1 (jsonb)
CREATE OR REPLACE FUNCTION update_by_id_v1(this_table_text text,
                                           this_column text,
                                           this_read_tenancy jsonb,
                                           this_write_tenancy jsonb,
                                           this_visibility jsonb,
                                           this_id ident,
                                           this_value jsonb,
                                           OUT updated_at timestamp with time zone)
AS
$$
BEGIN
    SELECT update_by_id_v1(this_table_text,
                           this_column,
                           this_read_tenancy,
                           this_write_tenancy,
                           this_visibility,
                           this_id,
                           CAST(this_value as text))
    INTO updated_at;
END ;
$$ LANGUAGE PLPGSQL VOLATILE;

-- update_by_id_v1 (bool)
CREATE OR REPLACE FUNCTION update_by_id_v1(this_table_text text,
                                           this_column text,
                                           this_read_tenancy jsonb,
                                           this_write_tenancy jsonb,
                                           this_visibility jsonb,
                                           this_id ident,
                                           this_value bool,
                                           OUT updated_at timestamp with time zone)
AS
$$
BEGIN
    SELECT update_by_id_v1(this_table_text,
                           this_column,
                           this_read_tenancy,
                           this_write_tenancy,
                           this_visibility,
                           this_id,
                           CAST(this_value as text))
    INTO updated_at;
END ;
$$ LANGUAGE PLPGSQL VOLATILE;

-- update_by_id_v1 (bigint)
CREATE OR REPLACE FUNCTION update_by_id_v1(this_table_text text,
                                           this_column text,
                                           this_read_tenancy jsonb,
                                           this_write_tenancy jsonb,
                                           this_visibility jsonb,
                                           this_id ident,
                                           this_value bigint,
                                           OUT updated_at timestamp with time zone)
AS
$$
BEGIN
    SELECT update_by_id_v1(this_table_text,
                           this_column,
                           this_read_tenancy,
                           this_write_tenancy,
                           this_visibility,
                           this_id,
                           CAST(this_value as text))
    INTO updated_at;
END ;
$$ LANGUAGE PLPGSQL VOLATILE;

-- update_by_id_v1 (timestamp with time zone)
CREATE OR REPLACE FUNCTION update_by_id_v1(this_table_text text,
                                           this_column text,
                                           this_read_tenancy jsonb,
                                           this_write_tenancy jsonb,
                                           this_visibility jsonb,
                                           this_id ident,
                                           this_value timestamp with time zone,
                                           OUT updated_at timestamp with time zone)
AS
$$
BEGIN
    SELECT update_by_id_v1(this_table_text,
                           this_column,
                           this_read_tenancy,
                           this_write_tenancy,
                           this_visibility,
                           this_id,
                           CAST(this_value as text))
    INTO updated_at;
END ;
$$ LANGUAGE PLPGSQL VOLATILE;

CREATE OR REPLACE FUNCTION delete_by_id_v1(this_table_text text,
                                           this_read_tenancy jsonb,
                                           this_write_tenancy jsonb,
                                           this_visibility jsonb,
                                           this_id ident,
                                           OUT updated_at timestamp with time zone)
AS
$$
BEGIN
    SELECT update_by_id_v1(this_table_text,
                           'visibility_deleted_at',
                           this_read_tenancy,
                           this_write_tenancy,
                           this_visibility,
                           this_id,
                           CAST(clock_timestamp() as text))
    INTO updated_at;
END ;
$$ LANGUAGE PLPGSQL VOLATILE;

CREATE OR REPLACE FUNCTION delete_by_pk_v1(this_table_text text,
                                           this_tenancy jsonb,
                                           this_pk ident,
                                           OUT updated_at timestamp with time zone)
AS
$$
DECLARE
    this_table regclass;
BEGIN
    this_table := this_table_text::regclass;
    EXECUTE format('UPDATE %1$I SET visibility_deleted_at = clock_timestamp(), updated_at = clock_timestamp() '
                   'WHERE pk = %3$L '
                   '  AND in_tenancy_v1(%2$L, '
                   '                    %1$I.tenancy_billing_account_pks, '
                   '                    %1$I.tenancy_organization_pks, '
                   '                    %1$I.tenancy_workspace_pks) '
                   ' RETURNING updated_at',
                   this_table, this_tenancy, this_pk) INTO updated_at;
END;
$$ LANGUAGE PLPGSQL VOLATILE;

CREATE OR REPLACE FUNCTION undelete_by_pk_v1(this_table_text text,
                                             this_tenancy jsonb,
                                             this_pk ident,
                                             OUT updated_at timestamp with time zone)
AS
$$
DECLARE
    this_table regclass;
BEGIN
    this_table := this_table_text::regclass;
    EXECUTE format('UPDATE %1$I SET visibility_deleted_at = NULL, updated_at = clock_timestamp() '
                   'WHERE pk = %3$L '
                   '  AND in_tenancy_v1(%2$L, '
                   '                    %1$I.tenancy_billing_account_pks, '
                   '                    %1$I.tenancy_organization_pks, '
                   '                    %1$I.tenancy_workspace_pks) '
                   ' RETURNING updated_at',
                   this_table, this_tenancy, this_pk) INTO updated_at;
END;
$$ LANGUAGE PLPGSQL VOLATILE;


CREATE OR REPLACE FUNCTION hard_delete_by_pk_v1(
    this_table_text text,
    this_pk ident,
    OUT object json
)
AS
$$
DECLARE
    this_table regclass;
BEGIN
    this_table := this_table_text::regclass;

    EXECUTE format(
        'DELETE FROM %1$I WHERE %1$I.pk = %2$L RETURNING row_to_json(%1$I);', 
        this_table, 
        this_pk
    ) INTO object;
END;
$$ LANGUAGE PLPGSQL;

CREATE OR REPLACE FUNCTION list_models_v1(this_table_text text, this_tenancy jsonb, this_visibility jsonb)
    RETURNS TABLE
            (
                id                       ident,
                visibility_change_set_pk ident,
                visibility_deleted_at    timestamp with time zone,
                object                   json
            )
AS
$$
DECLARE
    this_table regclass;
BEGIN
    this_table := this_table_text::regclass;
    RETURN QUERY EXECUTE format('SELECT '
                                '   table_alias.id, '
                                '   table_alias.visibility_change_set_pk, '
                                '   table_alias.visibility_deleted_at, '
                                '   row_to_json(table_alias.*) AS object '
                                ' FROM %1$I_v1(%2$L, %3$L) AS table_alias '
        , this_table, this_tenancy, this_visibility);
END ;
$$ LANGUAGE PLPGSQL STABLE;

CREATE OR REPLACE FUNCTION belongs_to_v1(this_table_text text,
                                         this_tenancy jsonb,
                                         this_visibility jsonb,
                                         this_retrieve_table text,
                                         this_object_id ident)
    RETURNS TABLE
            (
                id                       ident,
                visibility_change_set_pk ident,
                visibility_deleted_at    timestamp with time zone,
                object                   json
            )
AS
$$
DECLARE
    this_table regclass;
BEGIN
    this_table := this_table_text::regclass;
    RETURN QUERY EXECUTE format('SELECT '
                                '   table_alias.id, '
                                '   table_alias.visibility_change_set_pk, '
                                '   table_alias.visibility_deleted_at, '
                                '   row_to_json(retrieve_table_alias.*) AS object '
                                ' FROM %1$I_v1(%2$L, %3$L) AS table_alias '
                                ' INNER JOIN %5$I_v1(%2$L, %3$L) AS retrieve_table_alias '
                                '   ON retrieve_table_alias.id = table_alias.belongs_to_id '
                                ' WHERE table_alias.object_id = %4$L '
                                ' ORDER BY retrieve_table_alias.id, '
                                '          retrieve_table_alias.visibility_change_set_pk DESC, '
                                '          retrieve_table_alias.visibility_deleted_at DESC NULLS FIRST '
                                ' LIMIT 1 '
        , this_table, this_tenancy, this_visibility, this_object_id, this_retrieve_table);
END;
$$ LANGUAGE PLPGSQL STABLE;

CREATE OR REPLACE FUNCTION has_many_v1(this_table_text text,
                                       this_tenancy jsonb,
                                       this_visibility jsonb,
                                       this_retrieve_table text,
                                       this_belongs_to_id ident)
    RETURNS TABLE
            (
                id                       ident,
                visibility_change_set_pk ident,
                visibility_deleted_at    timestamp with time zone,
                object                   json
            )
AS
$$
DECLARE
    this_table regclass;
    query      text;
BEGIN
    this_table := this_table_text::regclass;
    query := format('SELECT '
                    '   table_alias.id, '
                    '   table_alias.visibility_change_set_pk, '
                    '   table_alias.visibility_deleted_at, '
                    '   row_to_json(retrieve_table_alias.*) AS object '
                    ' FROM %1$I_v1(%2$L, %3$L) AS table_alias '
                    ' INNER JOIN %5$I_v1(%2$L, %3$L) AS retrieve_table_alias '
                    '     ON retrieve_table_alias.id = table_alias.object_id '
                    ' WHERE table_alias.belongs_to_id = %4$L '
                    ' ORDER BY table_alias.id, '
                    '          table_alias.visibility_change_set_pk DESC, '
                    '          table_alias.visibility_deleted_at DESC NULLS FIRST '
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
                                           this_left_object_id ident,
                                           this_right_object_id ident)
    RETURNS TABLE
            (
                id                       ident,
                visibility_change_set_pk ident,
                visibility_deleted_at    timestamp with time zone,
                object                   json
            )
AS
$$
DECLARE
    this_table           regclass;
    query                text;
    this_return_table    text;
    this_query_object_id ident;
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
    query := format('SELECT '
                    '   table_alias.id, '
                    '   table_alias.visibility_change_set_pk, '
                    '   table_alias.visibility_deleted_at, '
                    '   row_to_json(return_table_alias.*) AS object '
                    ' FROM %1$I_v1(%2$L, %3$L) AS table_alias '
                    ' INNER JOIN %5$I_v1(%2$L, %3$L) AS return_table_alias '
                    '     ON return_table_alias.id = table_alias.%6$I '
                    ' WHERE table_alias.%7$I = %4$L '
                    ' ORDER BY table_alias.id, '
                    '          table_alias.visibility_change_set_pk DESC, '
                    '          table_alias.visibility_deleted_at DESC NULLS FIRST '
        , this_table, this_tenancy, this_visibility, this_query_object_id, this_return_table, this_join_column,
                    this_query_column);

    RAISE DEBUG 'many_to_many query: %', query;

    RETURN QUERY EXECUTE query;
END;
$$ LANGUAGE PLPGSQL STABLE;


CREATE OR REPLACE FUNCTION check_id_in_table_v1(this_table_name text, this_id ident, OUT result bool) AS
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
    create_table := format('CREATE TABLE %1$I ( '
                           ' pk                          ident primary key default ident_create_v1(), '
                           ' id                          ident not null default ident_create_v1(), '
                           ' object_id                   ident                   NOT NULL, '
                           ' belongs_to_id               ident                   NOT NULL, '
                           ' tenancy_billing_account_pks ident[], '
                           ' tenancy_organization_pks    ident[], '
                           ' tenancy_workspace_pks       ident[], '
                           ' visibility_change_set_pk    ident                   NOT NULL DEFAULT ident_nil_v1(), '
                           ' visibility_deleted_at       timestamp with time zone, '
                           ' created_at                  timestamp with time zone NOT NULL DEFAULT CLOCK_TIMESTAMP(), '
                           ' updated_at                  timestamp with time zone NOT NULL DEFAULT CLOCK_TIMESTAMP() '
                           '); '
                           'CREATE UNIQUE INDEX %1$s_visibility_tenancy ON %1$I (id, '
                           '                                    tenancy_billing_account_pks, '
                           '                                    tenancy_workspace_pks, '
                           '                                    tenancy_organization_pks, '
                           '                                    visibility_change_set_pk, '
                           '                                    (visibility_deleted_at IS NULL)) '
                           '                    WHERE visibility_deleted_at IS NULL; '
                           'ALTER TABLE %1$I '
                           '    ADD CONSTRAINT %1$s_object_id_is_valid '
                           '        CHECK (check_id_in_table_v1(%2$L, object_id)); '
                           'ALTER TABLE %1$I '
                           '    ADD CONSTRAINT %1$s_belongs_to_id_is_valid '
                           '        CHECK (check_id_in_table_v1(%3$L, belongs_to_id)); '
                           'CREATE UNIQUE INDEX %1$s_single_association ON %1$I (object_id, '
                           '                                        tenancy_billing_account_pks, '
                           '                                        tenancy_workspace_pks, '
                           '                                        tenancy_organization_pks, '
                           '                                        visibility_change_set_pk, '
                           '                                        (visibility_deleted_at IS NULL)) '
                           '                    WHERE visibility_deleted_at IS NULL; '
                           'CREATE INDEX ON %1$I (object_id); '
                           'CREATE INDEX ON %1$I (belongs_to_id); '
                           'CREATE FUNCTION is_visible_v1( '
                           '    check_visibility jsonb, '
                           '    reference %1$I '
                           ') '
                           'RETURNS bool '
                           'LANGUAGE sql '
                           'IMMUTABLE PARALLEL SAFE CALLED ON NULL INPUT '
                           'AS $is_visible_fn$ '
                           '    SELECT is_visible_v1( '
                           '        check_visibility, '
                           '        reference.visibility_change_set_pk, '
                           '        reference.visibility_deleted_at '
                           '    ) '
                           '$is_visible_fn$; '
                           'CREATE FUNCTION in_tenancy_v1( '
                           '    read_tenancy jsonb, '
                           '    record_to_check %1$I '
                           ') '
                           'RETURNS bool '
                           'LANGUAGE sql '
                           'IMMUTABLE PARALLEL SAFE CALLED ON NULL INPUT '
                           'AS $in_tenancy_fn$ '
                           '    SELECT in_tenancy_v1( '
                           '        read_tenancy, '
                           '        record_to_check.tenancy_billing_account_pks, '
                           '        record_to_check.tenancy_organization_pks, '
                           '        record_to_check.tenancy_workspace_pks '
                           '    ) '
                           '$in_tenancy_fn$; '
                           'CREATE FUNCTION in_tenancy_and_visible_v1( '
                           '    read_tenancy jsonb, '
                           '    check_visibility jsonb, '
                           '    record_to_check %1$I '
                           ') '
                           'RETURNS bool '
                           'LANGUAGE sql '
                           'IMMUTABLE PARALLEL SAFE CALLED ON NULL INPUT '
                           'AS $in_tenancy_and_visible_fn$ '
                           '    SELECT '
                           '        in_tenancy_v1( '
                           '            read_tenancy, '
                           '            record_to_check.tenancy_billing_account_pks, '
                           '            record_to_check.tenancy_organization_pks, '
                           '            record_to_check.tenancy_workspace_pks '
                           '        ) '
                           '        AND is_visible_v1( '
                           '            check_visibility, '
                           '            record_to_check.visibility_change_set_pk, '
                           '            record_to_check.visibility_deleted_at '
                           '        ) '
                           '$in_tenancy_and_visible_fn$; '
                           'CREATE FUNCTION %1$I_v1 ( '
                           '    this_read_tenancy jsonb, '
                           '    this_visibility jsonb '
                           ') '
                           'RETURNS SETOF %1$I '
                           'LANGUAGE sql '
                           'STABLE PARALLEL SAFE CALLED ON NULL INPUT '
                           'AS $table_view_fn$ '
                           '    SELECT DISTINCT ON (object_id) %1$I.* '
                           '    FROM %1$I '
                           '    WHERE in_tenancy_and_visible_v1(this_read_tenancy, this_visibility, %1$I) '
                           '    ORDER BY '
                           '        object_id, '
                           '        visibility_change_set_pk DESC, '
                           '        visibility_deleted_at DESC NULLS FIRST '
                           '$table_view_fn$; ',
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
    alter_query := format('CREATE UNIQUE INDEX %1$s_visibility_tenancy ON %1$I (id, '
                          '                                    tenancy_billing_account_pks, '
                          '                                    tenancy_workspace_pks, '
                          '                                    tenancy_organization_pks, '
                          '                                    visibility_change_set_pk, '
                          '                                    (visibility_deleted_at IS NULL)) '
                          '                    WHERE visibility_deleted_at IS NULL; '
                          'CREATE INDEX ON %1$I (id); '
                          'CREATE INDEX ON %1$I (visibility_deleted_at NULLS FIRST); '
                          'CREATE INDEX ON %1$I (visibility_change_set_pk); '
                          'CREATE FUNCTION is_visible_v1( '
                          '    check_visibility jsonb, '
                          '    reference %1$I '
                          ') '
                          'RETURNS bool '
                          'LANGUAGE sql '
                          'IMMUTABLE PARALLEL SAFE CALLED ON NULL INPUT '
                          'AS $is_visible_fn$ '
                          '    SELECT is_visible_v1( '
                          '        check_visibility, '
                          '        reference.visibility_change_set_pk, '
                          '        reference.visibility_deleted_at '
                          '    ) '
                          '$is_visible_fn$; '
                          'CREATE FUNCTION in_tenancy_v1( '
                          '    read_tenancy jsonb, '
                          '    record_to_check %1$I '
                          ') '
                          'RETURNS bool '
                          'LANGUAGE sql '
                          'IMMUTABLE PARALLEL SAFE CALLED ON NULL INPUT '
                          'AS $in_tenancy_fn$ '
                          '    SELECT in_tenancy_v1( '
                          '        read_tenancy, '
                          '        record_to_check.tenancy_billing_account_pks, '
                          '        record_to_check.tenancy_organization_pks, '
                          '        record_to_check.tenancy_workspace_pks '
                          '    ) '
                          '$in_tenancy_fn$; '
                          'CREATE FUNCTION in_tenancy_and_visible_v1( '
                          '    read_tenancy jsonb, '
                          '    check_visibility jsonb, '
                          '    record_to_check %1$I '
                          ') '
                          'RETURNS bool '
                          'LANGUAGE sql '
                          'IMMUTABLE PARALLEL SAFE CALLED ON NULL INPUT '
                          'AS $in_tenancy_and_visible_fn$ '
                          '    SELECT '
                          '        in_tenancy_v1( '
                          '            read_tenancy, '
                          '            record_to_check.tenancy_billing_account_pks, '
                          '            record_to_check.tenancy_organization_pks, '
                          '            record_to_check.tenancy_workspace_pks '
                          '        ) '
                          '        AND is_visible_v1( '
                          '            check_visibility, '
                          '            record_to_check.visibility_change_set_pk, '
                          '            record_to_check.visibility_deleted_at '
                          '        ) '
                          '$in_tenancy_and_visible_fn$; '
                          'CREATE FUNCTION %1$I_v1( '
                          '  this_read_tenancy jsonb, '
                          '  this_visibility jsonb '
                          ') '
                          'RETURNS SETOF %1$I '
                          'LANGUAGE SQL '
                          'STABLE PARALLEL SAFE CALLED ON NULL INPUT '
                          'AS $table_view_fn$ '
                          'SELECT DISTINCT ON (table_version.id) '
                          '  table_version.* '
                          'FROM %1$I AS table_version '
                          'LEFT JOIN %1$I AS change_set_version '
                          '  ON table_version.id = change_set_version.id '
                          '    AND (this_visibility ->> ''visibility_change_set_pk'')::ident != ident_nil_v1() '
                          '    AND change_set_version.visibility_change_set_pk = (this_visibility ->> ''visibility_change_set_pk'')::ident '
                          '    AND table_version.visibility_change_set_pk = ident_nil_v1() '
                          '    AND change_set_version.visibility_deleted_at IS NOT NULL '
                          'WHERE '
                          '  in_tenancy_and_visible_v1(this_read_tenancy, this_visibility, table_version) '
                          '  AND change_set_version.id IS NULL '
                          'ORDER BY table_version.id, table_version.visibility_change_set_pk DESC, visibility_deleted_at DESC NULLS FIRST '
                          '$table_view_fn$; ',
                          this_table_name
        );
    RAISE DEBUG 'alter table query: %', alter_query;
    EXECUTE alter_query;
END;
$$ LANGUAGE plpgsql VOLATILE;

CREATE OR REPLACE FUNCTION many_to_many_table_create_v1(this_table_name text,
                                                        this_left_object_table text,
                                                        this_right_object_table text)
    RETURNS VOID
AS
$$
DECLARE
    create_table text;
BEGIN
    create_table := format('CREATE TABLE %1$I ( '
                           ' pk                          ident primary key default ident_create_v1(), '
                           ' id                          ident not null default ident_create_v1(), '
                           ' left_object_id              ident                   NOT NULL, '
                           ' right_object_id             ident                   NOT NULL, '
                           ' tenancy_billing_account_pks ident[], '
                           ' tenancy_organization_pks    ident[], '
                           ' tenancy_workspace_pks       ident[], '
                           ' visibility_change_set_pk    ident                   NOT NULL DEFAULT ident_nil_v1(), '
                           ' visibility_deleted_at       timestamp with time zone, '
                           ' created_at                  timestamp with time zone NOT NULL DEFAULT CLOCK_TIMESTAMP(), '
                           ' updated_at                  timestamp with time zone NOT NULL DEFAULT CLOCK_TIMESTAMP() '
                           '); '
                           'CREATE UNIQUE INDEX %1$s_visibility_tenancy ON %1$I (id, '
                           '                                    tenancy_billing_account_pks, '
                           '                                    tenancy_workspace_pks, '
                           '                                    tenancy_organization_pks, '
                           '                                    visibility_change_set_pk, '
                           '                                    (visibility_deleted_at IS NULL)) '
                           '                    WHERE visibility_deleted_at IS NULL; '
                           'ALTER TABLE %1$I '
                           '    ADD CONSTRAINT %1$s_left_object_id_is_valid '
                           '        CHECK (check_id_in_table_v1(%2$L, left_object_id)); '
                           'ALTER TABLE %1$I '
                           '    ADD CONSTRAINT %1$s_right_object_id_is_valid '
                           '        CHECK (check_id_in_table_v1(%3$L, right_object_id)); '
                           'CREATE UNIQUE INDEX %1$s_no_duplicate_associations ON %1$I (left_object_id, '
                           '                                    right_object_id, '
                           '                                    tenancy_billing_account_pks, '
                           '                                    tenancy_workspace_pks, '
                           '                                    tenancy_organization_pks, '
                           '                                    visibility_change_set_pk, '
                           '                                    (visibility_deleted_at IS NULL)) '
                           '                    WHERE visibility_deleted_at IS NULL; '
                           'CREATE INDEX ON %1$I (left_object_id); '
                           'CREATE INDEX ON %1$I (right_object_id); '
                           'CREATE FUNCTION is_visible_v1( '
                           '    check_visibility jsonb, '
                           '    reference %1$I '
                           ') '
                           'RETURNS bool '
                           'LANGUAGE sql '
                           'IMMUTABLE PARALLEL SAFE CALLED ON NULL INPUT '
                           'AS $is_visible_fn$ '
                           '    SELECT is_visible_v1( '
                           '        check_visibility, '
                           '        reference.visibility_change_set_pk, '
                           '        reference.visibility_deleted_at '
                           '    ) '
                           '$is_visible_fn$; '
                           'CREATE FUNCTION in_tenancy_v1( '
                           '    read_tenancy jsonb, '
                           '    record_to_check %1$I '
                           ') '
                           'RETURNS bool '
                           'LANGUAGE sql '
                           'IMMUTABLE PARALLEL SAFE CALLED ON NULL INPUT '
                           'AS $in_tenancy_fn$ '
                           '    SELECT in_tenancy_v1( '
                           '        read_tenancy, '
                           '        record_to_check.tenancy_billing_account_pks, '
                           '        record_to_check.tenancy_organization_pks, '
                           '        record_to_check.tenancy_workspace_pks '
                           '    ) '
                           '$in_tenancy_fn$; '
                           'CREATE FUNCTION in_tenancy_and_visible_v1( '
                           '    read_tenancy jsonb, '
                           '    check_visibility jsonb, '
                           '    record_to_check %1$I '
                           ') '
                           'RETURNS bool '
                           'LANGUAGE sql '
                           'IMMUTABLE PARALLEL SAFE CALLED ON NULL INPUT '
                           'AS $in_tenancy_and_visible_fn$ '
                           '    SELECT '
                           '        in_tenancy_v1( '
                           '            read_tenancy, '
                           '            record_to_check.tenancy_billing_account_pks, '
                           '            record_to_check.tenancy_organization_pks, '
                           '            record_to_check.tenancy_workspace_pks '
                           '        ) '
                           '        AND is_visible_v1( '
                           '            check_visibility, '
                           '            record_to_check.visibility_change_set_pk, '
                           '            record_to_check.visibility_deleted_at '
                           '        ) '
                           '$in_tenancy_and_visible_fn$; '
                           'CREATE FUNCTION %1$I_v1 ( '
                           '    this_read_tenancy jsonb, '
                           '    this_visibility jsonb '
                           ') '
                           'RETURNS SETOF %1$I '
                           'LANGUAGE sql '
                           'STABLE PARALLEL SAFE CALLED ON NULL INPUT '
                           'AS $table_view_fn$ '
                           '    SELECT DISTINCT ON (id) %1$I.* '
                           '    FROM %1$I '
                           '    WHERE in_tenancy_and_visible_v1(this_read_tenancy, this_visibility, %1$I) '
                           '    ORDER BY id, visibility_change_set_pk DESC, visibility_deleted_at DESC NULLS FIRST '
                           '$table_view_fn$;',
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
                                                     this_left_object_id ident,
                                                     this_right_object_id ident
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
            format('INSERT INTO %1$I (left_object_id, right_object_id, '
                   '                  tenancy_billing_account_pks, tenancy_organization_pks, '
                   '                  tenancy_workspace_pks, visibility_change_set_pk, '
                   '                  visibility_deleted_at) '
                   'VALUES (%2$L, '
                   '        %3$L, '
                   '        %4$L, '
                   '        %5$L, '
                   '        %6$L, '
                   '        %7$L, '
                   '        %8$L)',
                   this_table_name,
                   this_left_object_id,
                   this_right_object_id,
                   this_tenancy_record.tenancy_billing_account_pks,
                   this_tenancy_record.tenancy_organization_pks,
                   this_tenancy_record.tenancy_workspace_pks,
                   this_visibility_record.visibility_change_set_pk,
                   this_visibility_record.visibility_deleted_at
                );
    RAISE DEBUG 'associate many to many: %', insert_query;
    EXECUTE insert_query;
END;
$$ LANGUAGE plpgsql VOLATILE;

CREATE OR REPLACE FUNCTION disassociate_many_to_many_v1(this_table_name text,
                                                        this_tenancy jsonb,
                                                        this_visibility jsonb,
                                                        this_left_object_id ident,
                                                        this_right_object_id ident
) RETURNS VOID AS
$$
DECLARE
    update_query text;
BEGIN

    update_query :=
            format('UPDATE %1$I SET visibility_deleted_at = clock_timestamp() '
                   '  WHERE left_object_id = %2$L '
                   '    AND right_object_id = %3$L '
                   '    AND in_tenancy_v1(%4$L, '
                   '                    %1$I.tenancy_billing_account_pks, '
                   '                    %1$I.tenancy_organization_pks, '
                   '                    %1$I.tenancy_workspace_pks) '
                   '    AND is_visible_v1(%5$L, '
                   '                    %1$I.visibility_change_set_pk, '
                   '                    %1$I.visibility_deleted_at)',
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

CREATE OR REPLACE FUNCTION disassociate_all_many_to_many_v1(this_table_name text,
                                                            this_tenancy jsonb,
                                                            this_visibility jsonb,
                                                            this_left_object_id ident
) RETURNS VOID AS
$$
DECLARE
    update_query text;
BEGIN

    update_query :=
            format('UPDATE %1$I SET visibility_deleted_at = clock_timestamp() '
                   '  WHERE left_object_id = %2$L '
                   '    AND in_tenancy_v1(%3$L, '
                   '                    %1$I.tenancy_billing_account_pks, '
                   '                    %1$I.tenancy_organization_pks, '
                   '                    %1$I.tenancy_workspace_pks) '
                   '    AND is_visible_v1(%4$L, '
                   '                    %1$I.visibility_change_set_pk, '
                   '                    %1$I.visibility_deleted_at)',
                   this_table_name,
                   this_left_object_id,
                   this_tenancy,
                   this_visibility
                );
    RAISE DEBUG 'disassociate all many to many: %', update_query;
    EXECUTE update_query;
END;
$$ LANGUAGE plpgsql VOLATILE;

CREATE OR REPLACE FUNCTION set_belongs_to_v1(
    this_table_name text,
    this_read_tenancy jsonb,
    this_write_tenancy jsonb,
    this_visibility jsonb,
    this_object_id ident,
    this_belongs_to_id ident
) RETURNS VOID AS
$$
DECLARE
    insert_query              text;
    this_write_tenancy_record tenancy_record_v1;
    this_visibility_record    visibility_record_v1;
    this_existing_record_id   ident;
BEGIN
    this_write_tenancy_record := tenancy_json_to_columns_v1(this_write_tenancy);
    this_visibility_record := visibility_json_to_columns_v1(this_visibility);

    SELECT id
    INTO this_existing_record_id
    FROM find_by_attr_v1(
        this_table_name,
        this_read_tenancy,
        this_visibility,
        'object_id',
        this_object_id::text
    );

    IF this_existing_record_id IS NOT NULL THEN
        -- Since there is an existing relation record for the object we're interested in,
        -- we need to update that record, rather than create a new relation record.
        PERFORM update_by_id_v1(
            this_table_name,
            'belongs_to_id',
            this_read_tenancy,
            this_write_tenancy,
            this_visibility,
            this_existing_record_id,
            this_belongs_to_id
        );
        RETURN;
    END IF;

    insert_query :=
            format(' INSERT INTO %1$I (object_id, belongs_to_id, '
                   '                  tenancy_billing_account_pks, tenancy_organization_pks, '
                   '                  tenancy_workspace_pks, visibility_change_set_pk, '
                   '                  visibility_deleted_at) '
                   ' VALUES (%2$L, '
                   '         %3$L, '
                   '         %4$L, '
                   '         %5$L, '
                   '         %6$L, '
                   '         %7$L, '
                   '         %8$L)',
                   this_table_name,
                   this_object_id,
                   this_belongs_to_id,
                   this_write_tenancy_record.tenancy_billing_account_pks,
                   this_write_tenancy_record.tenancy_organization_pks,
                   this_write_tenancy_record.tenancy_workspace_pks,
                   this_visibility_record.visibility_change_set_pk,
                   this_visibility_record.visibility_deleted_at
                );
    RAISE DEBUG 'set belongs to: %', insert_query;
    EXECUTE insert_query;
END;
$$ LANGUAGE plpgsql VOLATILE;

CREATE OR REPLACE FUNCTION unset_belongs_to_v1(this_table_name text,
                                               this_tenancy jsonb,
                                               this_visibility jsonb,
                                               this_object_id ident
) RETURNS VOID AS
$$
DECLARE
    update_query text;
BEGIN

    update_query :=
            format('UPDATE %1$I SET visibility_deleted_at = clock_timestamp() '
                   '  WHERE object_id = %2$L '
                   '    AND in_tenancy_v1(%3$L, '
                   '                    %1$I.tenancy_billing_account_pks, '
                   '                    %1$I.tenancy_organization_pks, '
                   '                    %1$I.tenancy_workspace_pks) '
                   '    AND is_visible_v1(%4$L, '
                   '                    %1$I.visibility_change_set_pk, '
                   '                    %1$I.visibility_deleted_at)',
                   this_table_name,
                   this_object_id,
                   this_tenancy,
                   this_visibility
                );
    RAISE DEBUG 'unset belongs to: %', update_query;
    EXECUTE update_query;
END;
$$ LANGUAGE plpgsql VOLATILE;

CREATE OR REPLACE FUNCTION hard_unset_belongs_to_in_change_set_v1(this_table_name text,
                                                                  this_tenancy jsonb,
                                                                  this_visibility jsonb,
                                                                  this_object_id ident
) RETURNS VOID AS
$$
DECLARE
    update_query text;
BEGIN

    update_query :=
            format('DELETE FROM %1$I '
                   '  WHERE object_id = %2$L '
                   '        AND visibility_change_set_pk = (%4$L ->> visibility_change_set_pk)::ident '
                   '        AND in_tenancy_v1(%3$L, '
                   '                        %1$I.tenancy_billing_account_pks, '
                   '                        %1$I.tenancy_organization_pks, '
                   '                        %1$I.tenancy_workspace_pks)',
                   this_table_name,
                   this_object_id,
                   this_tenancy,
                   this_visibility
                );
    RAISE DEBUG 'unset belongs to: %', update_query;
    EXECUTE update_query;
END;
$$ LANGUAGE plpgsql VOLATILE;

CREATE OR REPLACE FUNCTION unset_all_belongs_to_v1(this_table_name text,
                                                   this_tenancy jsonb,
                                                   this_visibility jsonb,
                                                   this_belongs_to_id ident
) RETURNS VOID AS
$$
DECLARE
    update_query text;
BEGIN

    update_query :=
            format('UPDATE %1$I SET visibility_deleted_at = clock_timestamp() '
                   '  WHERE belongs_to_id = %2$L '
                   '    AND in_tenancy_v1(%3$L, '
                   '                      %1$I.tenancy_billing_account_pks, '
                   '                      %1$I.tenancy_organization_pks, '
                   '                      %1$I.tenancy_workspace_pks) '
                   '    AND is_visible_v1(%4$L, '
                   '                      %1$I.visibility_change_set_pk, '
                   '                      %1$I.visibility_deleted_at)',
                   this_table_name,
                   this_belongs_to_id,
                   this_tenancy,
                   this_visibility
                );
    RAISE DEBUG 'unset belongs to: %', update_query;
    EXECUTE update_query;
END;
$$ LANGUAGE plpgsql VOLATILE;

CREATE OR REPLACE FUNCTION hard_unset_all_belongs_to_in_change_set_v1(this_table_name text,
                                                                      this_tenancy jsonb,
                                                                      this_visibility jsonb,
                                                                      this_belongs_to_id ident
) RETURNS VOID AS
$$
DECLARE
    update_query text;
BEGIN

    update_query :=
            format('DELETE FROM %1$I '
                   '  WHERE belongs_to_id = %2$L '
                   '        AND visibility_change_set_pk = (%4$L ->> visibility_change_set_pk)::ident '
                   '        AND in_tenancy_v1(%3$L, '
                   '                        %1$I.tenancy_billing_account_pks, '
                   '                        %1$I.tenancy_organization_pks, '
                   '                        %1$I.tenancy_workspace_pks)',
                   this_table_name,
                   this_belongs_to_id,
                   this_tenancy,
                   this_visibility
                );
    RAISE DEBUG 'unset belongs to: %', update_query;
    EXECUTE update_query;
END;
$$ LANGUAGE plpgsql VOLATILE;

CREATE OR REPLACE FUNCTION import_builtins_v1(destination_tenancy jsonb)
RETURNS VOID AS
$$
DECLARE
    standard_model            standard_models%ROWTYPE;
    this_table_name           regclass;
    insert_column_names       text;
    source_billing_account_pk ident;
BEGIN
    FOR standard_model IN SELECT * FROM standard_models
        LOOP
            this_table_name := standard_model.table_name::regclass;

            SELECT string_agg(information_schema.columns.column_name::text, ',')
            FROM information_schema.columns
            WHERE information_schema.columns.table_name = standard_model.table_name
              AND information_schema.columns.column_name NOT IN ('pk', 'created_at', 'tenancy_billing_account_pks', 'tenancy_organization_pks', 'tenancy_workspace_pks', 'visibility_change_set_pk')
              AND information_schema.columns.is_generated = 'NEVER'
            INTO insert_column_names;

            SELECT (object ->> 'pk')::ident
            INTO source_billing_account_pk
            FROM billing_account_upsert_builtin_v1();

            -- No history events for this update
            EXECUTE format('INSERT INTO %1$I (tenancy_billing_account_pks,
                                              tenancy_organization_pks,
                                              tenancy_workspace_pks,
                                              visibility_change_set_pk,
                                              %2$s)
                           SELECT %3$L::ident[], %4$L::ident[], %5$L::ident[], ident_nil_v1(), %2$s
                           FROM %1$I
                           WHERE in_tenancy_v1(%6$L, %1$I.tenancy_billing_account_pks,
                                                     %1$I.tenancy_organization_pks,
                                                     %1$I.tenancy_workspace_pks)',
                           this_table_name,
                           insert_column_names,
                           ARRAY(SELECT json_array_elements_text((destination_tenancy -> 'tenancy_billing_account_pks')::json)),
                           ARRAY(SELECT json_array_elements_text((destination_tenancy -> 'tenancy_organization_pks')::json)),
                           ARRAY(SELECT json_array_elements_text((destination_tenancy -> 'tenancy_workspace_pks')::json)),
                           jsonb_build_object(
                               'tenancy_billing_account_pks', array[source_billing_account_pk],
                               'tenancy_organization_pks', '{}'::json,
                               'tenancy_workspace_pks', '{}'::json
                           ));
        END LOOP;
END;
$$ LANGUAGE PLPGSQL VOLATILE;
