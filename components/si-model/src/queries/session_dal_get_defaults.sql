SELECT workspaces.obj AS workspace, organizations.obj AS organization, entities_head.obj AS system
FROM billing_accounts
         LEFT JOIN workspaces ON workspaces.billing_account_id = billing_accounts.id AND workspaces.name = 'default'
         LEFT JOIN organizations
                   ON organizations.billing_account_id = billing_accounts.id AND organizations.name = 'default'
         LEFT JOIN entities ON entities.workspace_id = workspaces.id AND entities.entity_type = 'system'
         LEFT JOIN entities_head ON entities.id = entities_head.id AND entities_head.obj ->> 'name' = 'production'
WHERE billing_accounts.id = si_id_to_primary_key_v1($1)