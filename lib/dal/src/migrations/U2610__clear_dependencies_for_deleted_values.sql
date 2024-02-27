CREATE OR REPLACE FUNCTION clear_dependencies_for_deleted_values_v1(
    this_tenancy jsonb,
    this_visibility jsonb
)
RETURNS VOID
AS
$$
DECLARE
    dependency_row_id ident;
BEGIN
    FOR dependency_row_id IN
        SELECT avd.id
        FROM attribute_value_dependencies_v1(this_tenancy, this_visibility) AS avd
        LEFT JOIN attribute_values_v1(this_tenancy, this_visibility) AS attribute_values_source_join 
            ON avd.source_attribute_value_id = attribute_values_source_join.id
        LEFT JOIN attribute_values_v1(this_tenancy, this_visibility) AS attribute_values_dest_join 
            ON avd.destination_attribute_value_id = attribute_values_dest_join.id
        WHERE attribute_values_source_join.id IS NULL
            OR attribute_values_dest_join.id IS NULL
    LOOP
        PERFORM delete_by_id_v1('attribute_value_dependencies', this_tenancy, this_visibility, dependency_row_id);
    END LOOP;
END
$$ LANGUAGE PLPGSQL;