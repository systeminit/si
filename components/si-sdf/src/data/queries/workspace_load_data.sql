--- You're welcome!
(SELECT obj AS object, epoch, update_count
 FROM change_sets
 WHERE workspace_id = si_id_to_primary_key_v1($1)
   AND epoch >= $2
   AND update_count >= $3
 UNION ALL
 SELECT obj AS object, epoch, update_count
 FROM change_set_participants
 WHERE workspace_id = si_id_to_primary_key_v1($1)
   AND epoch >= $2
   AND update_count >= $3
 UNION ALL
 SELECT obj AS object, epoch, update_count
 FROM edges
 WHERE workspace_id = si_id_to_primary_key_v1($1)
   AND epoch >= $2
   AND update_count >= $3
 UNION ALL
 SELECT obj AS object, epoch, update_count
 FROM edit_sessions
 WHERE workspace_id = si_id_to_primary_key_v1($1)
   AND epoch >= $2
   AND update_count >= $3
 UNION ALL
 SELECT entities_head.obj AS object, entities_head.epoch, entities_head.update_count
 FROM entities
          INNER JOIN entities_head ON (entities_head.id = entities.id AND epoch >= $2 AND update_count >= $3)
 WHERE workspace_id = si_id_to_primary_key_v1($1)
 UNION ALL
 SELECT entities_projection.obj AS object, entities_projection.epoch, entities_projection.update_count
 FROM entities
          INNER JOIN entities_projection
                     ON (entities_projection.id = entities.id AND epoch >= $2 AND update_count >= $3)
 WHERE workspace_id = si_id_to_primary_key_v1($1)
 UNION ALL
 SELECT systems_head.obj AS object, systems_head.epoch, systems_head.update_count
 FROM systems
          INNER JOIN systems_head ON (systems_head.id = systems.id AND epoch >= $2 AND update_count >= $3)
 WHERE workspace_id = si_id_to_primary_key_v1($1)
 UNION ALL
 SELECT systems_projection.obj AS object, systems_projection.epoch, systems_projection.update_count
 FROM systems
          INNER JOIN systems_projection ON (systems_projection.id = systems.id AND epoch >= $2 AND update_count >= $3)
 WHERE workspace_id = si_id_to_primary_key_v1($1)
 UNION ALL
 SELECT obj AS object, epoch, update_count
 FROM nodes
 WHERE workspace_id = si_id_to_primary_key_v1($1)
   AND epoch >= $2
   AND update_count >= $3
 UNION ALL
 SELECT obj AS object, epoch, update_count
 FROM ops
 WHERE workspace_id = si_id_to_primary_key_v1($1)
   AND epoch >= $2
   AND update_count >= $3
 UNION ALL
 SELECT resources_head.obj AS object, resources_head.epoch, resources_head.update_count
 FROM resources
          INNER JOIN resources_head ON (resources_head.id = resources.id AND epoch >= $2 AND update_count >= $3)
 WHERE workspace_id = si_id_to_primary_key_v1($1)
 UNION ALL
 SELECT resources_projection.obj AS object, resources_projection.epoch, resources_projection.update_count
 FROM resources
          INNER JOIN resources_projection
                     ON (resources_projection.id = resources.id AND epoch >= $2 AND update_count >= $3)
 WHERE workspace_id = si_id_to_primary_key_v1($1)
 UNION ALL
 SELECT obj AS object, epoch, update_count
 FROM secrets
 WHERE workspace_id = si_id_to_primary_key_v1($1)
   AND epoch >= $2
   AND update_count >= $3
 UNION ALL
 SELECT billing_accounts.obj AS object, 0 as epoch, 0 as update_count
 FROM workspaces
          INNER JOIN billing_accounts ON (billing_accounts.id = workspaces.billing_account_id)
 WHERE workspaces.id = si_id_to_primary_key_v1($1)
 UNION ALL
 SELECT key_pairs.obj AS object, 0 as epoch, 0 as update_count
 FROM workspaces
          INNER JOIN key_pairs ON (key_pairs.billing_account_id = workspaces.billing_account_id)
 WHERE workspaces.id = si_id_to_primary_key_v1($1)
)
    ORDER BY epoch, update_count;