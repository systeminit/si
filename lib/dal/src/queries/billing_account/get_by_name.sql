SELECT
    id,
    visibility_change_set_pk,

    row_to_json(ba.*) AS object
FROM billing_accounts_v1($2, $3) AS ba
WHERE name = $1
