SELECT
    pk,
    row_to_json(ba.*) AS object
FROM billing_accounts AS ba
WHERE name = $1 AND visibility_deleted_at is NULL
