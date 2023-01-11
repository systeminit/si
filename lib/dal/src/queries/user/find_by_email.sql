SELECT row_to_json(users.*) AS object
FROM users_v1($2, $3) AS users
WHERE users.email = $1
