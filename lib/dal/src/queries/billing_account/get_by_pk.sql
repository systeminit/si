SELECT row_to_json(ba.*) AS object
FROM billing_accounts AS ba
WHERE pk = $1 AND visibility_deleted_at is NULL
