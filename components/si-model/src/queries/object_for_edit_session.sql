SELECT COALESCE({root_table_name}_edit_session_projection.obj, {root_table_name}_change_set_projection.obj, {root_table_name}_head.obj) AS object
FROM {root_table_name}
         LEFT JOIN {root_table_name}_edit_session_projection ON {root_table_name}_edit_session_projection.id = {root_table_name}.id
    AND {root_table_name}_edit_session_projection.change_set_id = si_id_to_primary_key_v1($2)
    AND {root_table_name}_edit_session_projection.edit_session_id = si_id_to_primary_key_v1($3)
         LEFT JOIN {root_table_name}_change_set_projection ON {root_table_name}_change_set_projection.id = {root_table_name}.id
    AND {root_table_name}_change_set_projection.change_set_id = si_id_to_primary_key_v1($2)
         LEFT JOIN {root_table_name}_head ON {root_table_name}_head.id = {root_table_name}.id
WHERE {root_table_name}.id = si_id_to_primary_key_v1($1);
