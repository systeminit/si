SELECT row_to_json(change_set_pointers.*) AS object
FROM change_set_pointers
WHERE change_set_pointers.id = $1
