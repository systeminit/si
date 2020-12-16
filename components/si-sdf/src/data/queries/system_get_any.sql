SELECT COALESCE(systems_head.obj, systems_base.obj) AS object
FROM systems
         LEFT OUTER JOIN systems_head ON (systems_head.id = systems.id)
         LEFT OUTER JOIN systems_base ON (systems_base.id = systems.id)
WHERE systems.id = si_id_to_primary_key_v1($1);