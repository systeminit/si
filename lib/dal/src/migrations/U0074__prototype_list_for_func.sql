/*
    list all prototypes for a given function.
    currently works only for prototypes that have component ids as their most specific context id
 */
CREATE OR REPLACE FUNCTION prototype_list_for_func_v1(this_table_text text,
                                                      this_tenancy jsonb,
                                                      this_visibility jsonb,
                                                      this_func_id bigint)
    RETURNS TABLE
            (
                id                       bigint,
                component_id             bigint,
                schema_id                bigint,
                schema_variant_id        bigint,
                system_id                bigint,
                visibility_change_set_pk bigint,
                object                   json
            )
AS
$$
DECLARE
    this_table regclass;
BEGIN
    this_table := this_table_text::regclass;
    RETURN QUERY EXECUTE format('SELECT DISTINCT ON (%1$I.id)' ||
                                '  %1$I.id,' ||
                                '  %1$I.component_id,' ||
                                '  %1$I.schema_id,' ||
                                '  %1$I.schema_variant_id,' ||
                                '  %1$I.system_id,' ||
                                '  %1$I.visibility_change_set_pk,' ||
                                '  row_to_json(%1$I.*) AS object' ||
                                ' FROM %1$I' ||
                                ' WHERE in_tenancy_and_visible_v1(%2$L, %3$L, %1$I)' ||
                                '  AND %1$I.func_id = %4$L' ||
                                ' ORDER BY id,' ||
                                '         visibility_change_set_pk DESC,' ||
                                '         visibility_deleted_at DESC NULLS FIRST,' ||
                                '         component_id DESC,' ||
                                '         func_id DESC,' ||
                                '         system_id DESC,' ||
                                '         schema_variant_id DESC,' ||
                                '         schema_id DESC'
        , this_table, this_tenancy, this_visibility, this_func_id);
END ;

$$ LANGUAGE PLPGSQL;