(SELECT e.obj AS object, e.change_set_epoch, e.change_set_update_count
 FROM entities_base AS e
 WHERE e.change_set_id = si_id_to_primary_key_v1($1)
 UNION
 SELECT s.obj AS object, s.change_set_epoch, s.change_set_update_count
 FROM systems_base AS s
 WHERE s.change_set_id = si_id_to_primary_key_v1($1)
 UNION
 SELECT o.obj AS object, o.change_set_epoch, o.change_set_update_count
 FROM ops AS o
 WHERE o.change_set_id = si_id_to_primary_key_v1($1))
    ORDER BY change_set_epoch, change_set_update_count;