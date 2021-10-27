SELECT {root_table_name}_head.obj AS object
FROM {root_table_name}
         LEFT JOIN {root_table_name}_head ON {root_table_name}_head.id = {root_table_name}.id
WHERE {root_table_name}.id = si_id_to_primary_key_v1($1);
