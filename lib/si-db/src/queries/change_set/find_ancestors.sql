WITH RECURSIVE change_set_recursive AS (
    SELECT id, base_change_set_id FROM change_set_pointers WHERE id=$1
    UNION ALL
    SELECT change_set_pointers.id, change_set_pointers.base_change_set_id FROM change_set_pointers 
        INNER JOIN change_set_recursive ON change_set_recursive.base_change_set_id = change_set_pointers.id 
)
SELECT id FROM change_set_recursive;