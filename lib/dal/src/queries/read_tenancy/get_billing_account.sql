SELECT DISTINCT ON (billing_accounts.id) billing_accounts.id,
                                         row_to_json(billing_accounts.*) AS object
FROM billing_accounts
         INNER JOIN organization_belongs_to_billing_account
                    ON organization_belongs_to_billing_account.belongs_to_id = billing_accounts.id
WHERE organization_belongs_to_billing_account.object_id = $1
ORDER BY billing_accounts.id DESC;
