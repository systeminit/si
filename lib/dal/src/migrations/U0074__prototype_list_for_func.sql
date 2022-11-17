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
                object                   json
            )
AS
$$
DECLARE
    this_table regclass;
BEGIN
    this_table := this_table_text::regclass;
    RETURN QUERY EXECUTE format('SELECT row_to_json(%1$I.*) AS object'
                                ' FROM %1$I_v1(%2$L, %3$L) as %1$I'
                                '  WHERE %1$I.func_id = %4$L'
                                ' ORDER BY id, '
                                '          component_id DESC,'
                                '          func_id DESC,'
                                '          schema_variant_id DESC,'
                                '          schema_id DESC'
        , this_table, this_tenancy, this_visibility, this_func_id);
END ;

$$ LANGUAGE PLPGSQL;
