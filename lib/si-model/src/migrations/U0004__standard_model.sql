CREATE TABLE standard_models
(
    pk bigserial PRIMARY KEY,
    table_name text NOT NULL,
    history_event_label_base text NOT NULL,
    history_event_message_name text NOT NULL
);

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
                                '                    %1$I.tenancy_billing_account_pks,' ||
                                '                    %1$I.tenancy_organization_pks,' ||
                                '                    %1$I.tenancy_workspace_pks)' ||
                                '  AND is_visible_v1(%4$L,' ||
                                '                    %1$I.visibility_change_set_pk,' ||
                                '                    %1$I.visibility_edit_session_pk,' ||
                                '                    %1$I.visibility_deleted)' ||
                                ' ORDER BY id, visibility_change_set_pk ASC, visibility_edit_session_pk ASC' ||
                                ' LIMIT 1'
        , this_table, this_id, this_tenancy, this_visibility);
END ;
$$ LANGUAGE PLPGSQL STABLE;

CREATE OR REPLACE FUNCTION update_by_pk_v1(this_table_text text,
                                           this_column text,
                                           this_tenancy jsonb,
                                           this_pk bigint,
                                           this_value text,
                                           OUT updated_at timestamp with time zone)
AS
$$
DECLARE
    this_table  regclass;
BEGIN
    this_table := this_table_text::regclass;
    EXECUTE format('UPDATE %1$I SET %2$I = %5$L, updated_at = now() WHERE pk = %4$L ' ||
                   '  AND in_tenancy_v1(%3$L, ' ||
                   '                    %1$I.tenancy_universal, ' ||
                   '                    %1$I.tenancy_billing_account_pks,' ||
                   '                    %1$I.tenancy_organization_pks,' ||
                   '                    %1$I.tenancy_workspace_pks)' ||
                   ' RETURNING updated_at',
                   this_table, this_column, this_tenancy, this_pk, this_value) INTO updated_at;
END ;
$$ LANGUAGE PLPGSQL VOLATILE;

CREATE OR REPLACE FUNCTION delete_by_pk_v1(this_table_text text,
                                           this_tenancy jsonb,
                                           this_pk bigint,
                                           OUT updated_at timestamp with time zone)
AS
$$
DECLARE
    this_table  regclass;
BEGIN
    this_table := this_table_text::regclass;
    EXECUTE format('UPDATE %1$I SET visibility_deleted = true, updated_at = now() ' ||
                   'WHERE pk = %3$L ' ||
                   '  AND in_tenancy_v1(%2$L, ' ||
                   '                    %1$I.tenancy_universal, ' ||
                   '                    %1$I.tenancy_billing_account_pks,' ||
                   '                    %1$I.tenancy_organization_pks,' ||
                   '                    %1$I.tenancy_workspace_pks)' ||
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
    this_table  regclass;
BEGIN
    this_table := this_table_text::regclass;
    EXECUTE format('UPDATE %1$I SET visibility_deleted = false, updated_at = now() ' ||
                   'WHERE pk = %3$L ' ||
                   '  AND in_tenancy_v1(%2$L, ' ||
                   '                    %1$I.tenancy_universal, ' ||
                   '                    %1$I.tenancy_billing_account_pks,' ||
                   '                    %1$I.tenancy_organization_pks,' ||
                   '                    %1$I.tenancy_workspace_pks)' ||
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
                                '                    %1$I.tenancy_billing_account_pks,' ||
                                '                    %1$I.tenancy_organization_pks,' ||
                                '                    %1$I.tenancy_workspace_pks)' ||
                                '  AND is_visible_v1(%3$L,' ||
                                '                    %1$I.visibility_change_set_pk,' ||
                                '                    %1$I.visibility_edit_session_pk,' ||
                                '                    %1$I.visibility_deleted)' ||
                                ' ORDER BY id, visibility_change_set_pk ASC, visibility_edit_session_pk ASC'
        , this_table, this_tenancy, this_visibility);
END ;
$$ LANGUAGE PLPGSQL STABLE;
