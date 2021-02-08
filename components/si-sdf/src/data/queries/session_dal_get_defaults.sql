SELECT workspaces.obj AS workspace, organizations.obj AS organization, systems_head.obj AS system
FROM billing_accounts
         LEFT JOIN workspaces ON workspaces.billing_account_id = billing_accounts.id AND workspaces.name = 'default'
         LEFT JOIN organizations
                   ON organizations.billing_account_id = billing_accounts.id AND organizations.name = 'default'
         LEFT JOIN systems_head ON systems_head.id = (SELECT systems.id
                                                      FROM systems
                                                      WHERE systems.billing_account_id = billing_accounts.id
                                                        AND systems.workspace_id = workspaces.id) AND
                                   systems_head.obj ->> 'name' = 'default'
WHERE billing_accounts.id = si_id_to_primary_key_v1($1)